/// Module for interpreting parsed LoGo-lang AST (Abstract Syntax Tree) nodes and executing them.
///
/// This module provides an interpreter for LoGo-lang programs represented as AST nodes.
///
/// # Examples
///
/// ```
/// use logolang_interpreter::{Interpreter, AstNode, Value};
///
/// let ast = vec![
///     AstNode::MakeStmnt {
///         var: String::from("x"),
///         expr: Box::new(AstNode::Num(10.0)),
///         line: 1,
///     }
/// ];
///
/// let mut interpreter = Interpreter::new();
/// let result = interpreter.run(&ast);
/// assert!(result.is_ok());
/// ```
///

use crate::logolang_errors::InterpreterError;
use crate::parser::{ArithOp, AstNode, BoolOp, CompOp, Direction, NodeType, PenPos, QueryKind};
use anyhow::{Context, Result};
use core::panic;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::mem::discriminant;
use std::rc::Rc;
use unsvg::{get_end_coordinates, Image, COLORS};

/// Describes to turtles position
#[derive(Debug)]
pub struct Position {
    x_coordinate: f32,
    y_coordinate: f32,
    direction: f32,
}

/// The terminal values for which an expression can evaluate to
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Float(f32),
    Bool(bool),
    Word(String),
}

/// Implementation for addition assignment of type Value::Float
impl std::ops::AddAssign for Value {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Value::Float(left), Value::Float(right)) => *left += right,
            _ => panic!("Unsupported addition-assignment: Must be Value::Float()"),
        }
    }
}

// Implement displays for the purpose of reporting errors
impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::FORWARD => write!(f, "FORWARD"),
            Direction::BACK => write!(f, "BACK"),
            Direction::RIGHT => write!(f, "RIGHT"),
            Direction::LEFT => write!(f, "LEFT"),
        }
    }
}
impl std::fmt::Display for PenPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PenPos::SETX => write!(f, "SETX"),
            PenPos::SETY => write!(f, "SETY"),
            PenPos::SETHEADING => write!(f, "SETHEADING"),
            PenPos::TURN => write!(f, "TURN"),
        }
    }
}
impl std::fmt::Display for ArithOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithOp::ADD => write!(f, "+"),
            ArithOp::SUB => write!(f, "-"),
            ArithOp::MUL => write!(f, "*"),
            ArithOp::DIV => write!(f, "/"),
        }
    }
}
impl std::fmt::Display for CompOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompOp::EQ => write!(f, "EQ"),
            CompOp::NE => write!(f, "NE"),
            CompOp::LT => write!(f, "LT"),
            CompOp::GT => write!(f, "GT"),
        }
    }
}
impl std::fmt::Display for BoolOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolOp::AND => write!(f, "AND"),
            BoolOp::OR => write!(f, "OR"),
        }
    }
}


/// Interpreter for the RSLOGO language.
/// Performs top-down descent over the AST.
pub struct Interpreter<'a> {
    /// Image to write
    image: &'a mut Image,
    /// Variable environment
    environment: HashMap<String, Value>,
    /// Function environment
    func_environment: HashMap<String, Rc<Vec<AstNode>>>, // Map each proc name to a list of its param names and a pointer to its executable body
    /// Turtle position
    current_position: Position,
    /// Pen color
    current_color: usize,
    /// Drawing status
    currently_drawing: bool,
}

impl<'a> Interpreter<'a> {
    /// Constructor
    pub fn new(image: &'a mut Image) -> Self {
        let (width, height) = image.get_dimensions();
        Self {
            image,
            environment: HashMap::new(),
            func_environment: HashMap::new(),
            current_position: Position {
                x_coordinate: width as f32 / 2.0,
                y_coordinate: height as f32 / 2.0,
                direction: 0.0,
            },
            current_color: 7,         // Starts default white
            currently_drawing: false, // Starts default penup (not drawing)
        }
    }

    /// Runs the evaluator to traverse the AST.
    /// Returns the edited image on success, else returns an InterpreterError.
    pub fn run(&mut self, ast: &Vec<AstNode>) -> Result<&Image, InterpreterError> {
        self.evaluate(ast)
            .with_context(|| "Failed to evaluate program".to_string())?;
        // Return image on success
        Ok(self.image)
    }

    /// Traverses AST by matching on each parent node, and recursively stepping
    /// until leaf nodes are reached. The results are then propogated back up to
    /// the parent node.
    fn evaluate(&mut self, ast: &Vec<AstNode>) -> Result<(), InterpreterError> {
        for node in ast {
            match node {
                // Statement evaluation
                AstNode::MakeStmnt { var, expr, line } => {
                    self.make(String::from(var), expr, *line)?
                }
                AstNode::AddAssign {
                    var_name,
                    expr,
                    line,
                } => self.add_assign(var_name, expr, *line)?,
                AstNode::DrawInstruction {
                    direction,
                    num_pixels,
                    line,
                } => self.draw_line(direction, num_pixels, *line)?,
                AstNode::IfStmnt {
                    condition,
                    body,
                    line,
                } => self.if_statement(condition, body, *line)?,
                AstNode::WhileStmnt {
                    condition,
                    body,
                    line,
                } => self.while_statement(condition, body, *line)?,
                AstNode::PenStatusUpdate(new_drawing_status) => {
                    self.set_drawing_status(*new_drawing_status);
                }
                AstNode::PenColorUpdate { color, line } => self.set_pen_color(color, *line)?,
                AstNode::PenPosUpdate {
                    update_type,
                    value,
                    line,
                } => self.set_position(update_type, value, *line)?,
                AstNode::Procedure { name, body } => {
                    self.create_procedure(String::from(name), Rc::clone(body));
                }
                AstNode::ProcedureRef {
                    name_ref,
                    args,
                    line,
                } => self.eval_procedure(name_ref, args, *line)?,
                // Expressions that are evaluated here are stand alone expressions; that is,
                // their results are not used in any operations. We evaluate non-terminal
                // expressions for correctness, and return nothing for terminal expressions.
                AstNode::ArithExpr {
                    operator,
                    left,
                    right,
                    line,
                } => {
                    self.arith_expr(operator, left, right, *line)?;
                }
                AstNode::Query(_) => (),
                AstNode::IdentRef(_) => (),
                AstNode::Num { .. } => (),
                AstNode::CompExpr {
                    operator,
                    left,
                    right,
                    line,
                } => {
                    self.comp_expr(operator, left, right, *line)?;
                }
                AstNode::BoolExpr {
                    operator,
                    left,
                    right,
                    line,
                } => {
                    self.bool_expr(operator, left, right, *line)?;
                }
                AstNode::Ident { .. } => (),
                // If an ident it received here, it is not bound: treat it as an unbound word
                AstNode::Word(word) => self.word(word),
            }
        }
        Ok(())
    }

    /// Evaluation of MAKE statment
    fn make(&mut self, var: String, expr: &AstNode, line: i32) -> Result<(), InterpreterError> {
        let bound_val = match expr {
            // Numeric expressions
            AstNode::ArithExpr {
                operator,
                left,
                right,
                line
            } => Value::Float(self.arith_expr(operator, left, right, *line)
                              .with_context(|| format!("[Line {}]: interp Invalid MAKE statement: Failed to evaluate expression passed to {}",line, var))?),
            AstNode::Query(query_kind) => Value::Float(self.query(query_kind)),
            AstNode::IdentRef(var) => self.eval_ident_ref_as_val(var)
                    .with_context(|| format!("[Line {}]: Invalid MAKE statement: Failed to evaluate expression passed to {}",line, var))?,
            AstNode::Num(val) => Value::Float(*val),
            // Logic expressions
            AstNode::CompExpr {
                operator,
                left,
                right,
                line
            } => Value::Bool(self.comp_expr(operator, left, right, *line)
                             .with_context(|| format!("[Line {}]: Failed to evaluate expression provided to {}", line, operator))?),
            AstNode::BoolExpr {
                operator,
                left,
                right,
                line
            } => Value::Bool(self.bool_expr(operator, left, right, *line)
                           .with_context(|| format!("[Line {}]: Failed to evaluate expression provided to {}", line, operator))?),                            
            // Word expressions
            AstNode::Word(word) => Value::Word(word.to_string()),
            _ => unreachable!("fn make_op in parser checks that expressions passed to MAKE implement is_boolean() or is_numeric()."),
        };

        // Add binding to map
        self.environment.insert(var, bound_val);
        Ok(())
    }


    /// Evaluation of numeric expressions tp their terminal float value.
    /// Numeric expression include arith_expr, query_expr, ident_ref and num.
    fn eval_numeric_expression(
        &mut self,
        node: &AstNode,
        line: i32,
    ) -> Result<f32, InterpreterError> {
        match node {
            AstNode::ArithExpr {
                operator,
                left,
                right,
                line,
            } => Ok(self.arith_expr(operator, left, right, *line)
                        .with_context(|| format!("[Line {}]: Failed to evaluate expression passed to {}"
                                                 ,line
                                                 ,operator))?
                    ),
            AstNode::Query(query_kind) => Ok(self.query(query_kind)),
            AstNode::IdentRef(var) => {
                let ident_value = self.eval_ident_ref(var)?;
                match ident_value {
                    Value::Float(num) => Ok(*num),
                    Value::Bool(val) => Err(InterpreterError::TypeError(format!("[Line {}]: variable '{}' is assigned to the boolean value {}, not a float.",
                                                                                 line,String::from(var), val))),
                    Value::Word(word) => Err(InterpreterError::TypeError(format!("[Line {}]: variable '{}' is assigned to the String value {}, not a float."
                                                                             ,line,String::from(var), word))),
                }
            }
            AstNode::Num(val) => Ok(*val),
            _ => unreachable!("This fn is only called by functions which expect numeric expressions, which has already been verified by the parser."),
        }
    }
    

    /// Evalutes logic expressions to their terminal bool value.
    /// Logic expressions include comparison_expr, boolean_expr and ident_ref
    fn eval_logic_expression(
        &mut self,
        node: &AstNode,
        line: i32,
    ) -> Result<bool, InterpreterError> {
        match node {
            AstNode::CompExpr {
                operator,
                left,
                right,
                line,
            } => Ok(self
                .comp_expr(operator, left, right, *line)
                .with_context(|| {
                    format!(
                        "[Line {}]: Failed to evaluate expression passed to {}",
                        line, operator
                    )
                })?),
            AstNode::BoolExpr {
                operator,
                left,
                right,
                line,
            } => Ok(self
                .bool_expr(operator, left, right, *line)
                .with_context(|| {
                    format!(
                        "[Line {}]: Failed to evaluate expression passed to {}",
                        line, operator
                    )
                })?),
            AstNode::IdentRef(var) => {
                let ident_value = self
                    .eval_ident_ref(var)
                    .with_context(|| String::from("Error evaluating numeric expression"))?;
                match ident_value {
                    Value::Bool(value) => Ok(*value),
                    Value::Float(num) => Err(InterpreterError::TypeError(format!(
                        "[Line {}]: variable '{}' is assigned to the float value {}, not a bool.",
                        line,
                        String::from(var),
                        num
                    ))),
                    Value::Word(word) => Err(InterpreterError::TypeError(format!(
                        "[Line {}]: variable '{}' is assigned to the String value {}, not a bool.",
                        line,
                        String::from(var),
                        word
                    ))),
                }
            }
            _ => panic!("All cases for which this function is called were expected to be handled"),
        }
    }

    /// Evaluates Addition Assignment operation
    fn add_assign(
        &mut self,
        var_name: &String,
        expr: &AstNode,
        line: i32,
    ) -> Result<(), InterpreterError> {
        let bound_value =
            Value::Float(self.eval_numeric_expression(expr, line).with_context(|| {
                format!(
                    "Invalid ADDASSIGN: Failed to add number to '{}'\n",
                    var_name
                )
            })?);
        if let Some(var) = self.environment.get_mut(var_name) {
            *var += bound_value;
        } else {
            return Err(InterpreterError::InvalidVariableRef(format!(
                "Variable {} does not exist.",
                var_name
            )));
        }
        Ok(())
    }

    /// Helper fn: Provides adjusted directions relative to current direction
    fn get_relative_direction(&mut self, direction: &Direction) -> i32 {
        match direction {
            Direction::FORWARD => self.current_position.direction as i32,
            Direction::BACK => self.current_position.direction as i32 + 180,
            Direction::LEFT => self.current_position.direction as i32 + 270,
            Direction::RIGHT => self.current_position.direction as i32 + 90,
        }
    }

    /// Draws a line given a direction
    fn draw_line(
        &mut self,
        direction: &Direction,
        value: &AstNode,
        line: i32,
    ) -> Result<(), InterpreterError> {
        let num_pixels = self.eval_numeric_expression(value, line).with_context(|| {
            format!(
                "[Line {}]: Invalid ADDASSIGN: Failed to add number to '{}'\n",
                line, direction
            )
        })?;

        let adjusted_direction = self.get_relative_direction(direction);

        if self.currently_drawing {
            let res_pair = self.image.draw_simple_line(
                self.current_position.x_coordinate,
                self.current_position.y_coordinate,
                adjusted_direction,
                num_pixels,
                COLORS[self.current_color],
            );

            match res_pair {
                Ok(res_pair) => {
                    (
                        self.current_position.x_coordinate,
                        self.current_position.y_coordinate,
                    ) = res_pair;
                }
                Err(error) => {
                    return Err(InterpreterError::DrawLineError(
                        format!(
                            "[Line {}]: Failed to draw line for direction {} due to UNSVG error:",
                            line, direction
                        ),
                        error.to_string(),
                    ))
                }
            };
        } else {
            // Update coordinates without drawing
            let res_pair = get_end_coordinates(
                self.current_position.x_coordinate,
                self.current_position.y_coordinate,
                adjusted_direction,
                num_pixels,
            );

            (
                self.current_position.x_coordinate,
                self.current_position.y_coordinate,
            ) = res_pair;
        };

        Ok(())
    }

    /// Evaluates If statement
    fn if_statement(
        &mut self,
        condition: &AstNode,
        body: &Vec<AstNode>,
        line: i32,
    ) -> Result<(), InterpreterError> {
        let condition_is_true = self
            .eval_logic_expression(condition, line)
            .with_context(|| format!("[Line {}]: Invalid IF statement condition.\n", line))?;
        if condition_is_true {
            self.evaluate(body)
                .with_context(|| format!("[Line {}]: Invalid IF statement condition.\n", line))?;
        }
        Ok(())
    }

    /// Evaluates while statement
    fn while_statement(
        &mut self,
        condition: &AstNode,
        body: &Vec<AstNode>,
        line: i32,
    ) -> Result<(), InterpreterError> {
        let condition_is_true = self
            .eval_logic_expression(condition, line)
            .with_context(|| format!("[Line {}]: Invalid WHILE statement condition.\n", line))?;

        if condition_is_true {
            self.evaluate(body).with_context(|| {
                format!(
                    "[Line {}]: Invalid expression in the body of the WHILE statement.\n",
                    line
                )
            })?;
            self.while_statement(condition, body, line)
                  .with_context(|| format!("[Line {}]: Invalid expression in the body of the WHILE loop encountered while looping.\n",line))?;
        }
        Ok(())
    }

    /// Sets drawing state
    fn set_drawing_status(&mut self, new_drawing_status: bool) {
        self.currently_drawing = new_drawing_status;
    }

    /// Sets pen color
    fn set_pen_color(&mut self, value: &AstNode, line: i32) -> Result<(), InterpreterError> {
        let float_val = self
            .eval_numeric_expression(value, line)
            .with_context(|| format!("[Line {}]: Invalid argument to PENCOLOR.\n", line))?;

        // Check precision & bounds before casting to an int color
        if float_val == (float_val as usize) as f32 && (0.0..=15.0).contains(&float_val) {
            self.current_color = float_val as usize;
        } else {
            return Err(InterpreterError::InvalidPenColor(float_val.to_string()));
        };
        Ok(())
    }

    /// Sets the position/orientation of the pen
    fn set_position(
        &mut self,
        update_type: &PenPos,
        value: &AstNode,
        line: i32,
    ) -> Result<(), InterpreterError> {
        let val = self
            .eval_numeric_expression(value, line)
            .with_context(|| format!("[Line {}]: Invalid argument to {}.\n", line, update_type))?;
        match update_type {
            PenPos::SETX => self.current_position.x_coordinate = val,
            PenPos::SETY => self.current_position.y_coordinate = val,
            PenPos::SETHEADING => self.current_position.direction = val,
            PenPos::TURN => self.current_position.direction += val,
        }

        Ok(())
    }

    /// Creates a new procedure binding in the function map
    fn create_procedure(&mut self, name: String, body: Rc<Vec<AstNode>>) {
        // Add the procedure name and body to the func environment
        self.func_environment.insert(name, body);
    }

    /// Evaluates a procedure that has been referenced
    // func_body has an exclusive borrow over the environment maps Vec<AstNode>. Below, we access
    // self.evaluate(), which itself may mutate the map. As we assume procedures are never defined
    // (but can be called) within another procedure, we can assure self.evaluate() will never
    // mutate the map, and will at most read from it, in the case another procedure is referenced.
    // As such, we take a Rc over the func_body to allow shared access to the map.
    fn eval_procedure(
        &mut self,
        name_ref: &String,
        args: &Vec<AstNode>,
        line: i32,
    ) -> Result<(), InterpreterError> {
        // Eval the args to bind the values
        self.evaluate(args).with_context(|| {
            format!(
                "[Line {}]: Failed to bind provided arguments to {}'s parameters.\n",
                line, name_ref
            )
        })?;

        // Evaluate body of procedure
        if let Some(func_body) = self.func_environment.get_mut(name_ref) {
            let mut func_body_rc = Rc::clone(func_body);
            self.evaluate(func_body_rc.borrow_mut()).with_context(|| {
                format!(
                    "[Line {}]: Failed to evaluate body of procedure {}.\n",
                    line, name_ref
                )
            })?;
        } else {
            return Err(InterpreterError::InvalidProcedureRef(format!(
                "[Line {}]: Referenced Procedure {} does not exist.",
                line, name_ref
            )));
        }

        Ok(())
    }

    /// Evaluates an arithmetic expression
    fn arith_expr(
        &mut self,
        operator: &ArithOp,
        left: &AstNode,
        right: &AstNode,
        line: i32,
    ) -> Result<f32, InterpreterError> {
        let left_val = self.eval_numeric_expression(left, line).with_context(|| {
            format!(
                "[Line {}]: Failed to evaluate first argument to operator '{}",
                line, operator
            )
        })?;

        let right_val = self.eval_numeric_expression(right, line).with_context(|| {
            format!(
                "[Line {}]: Failed to evaluate second argument to operator '{}",
                line, operator
            )
        })?;

        match operator {
            ArithOp::ADD => Ok(left_val + right_val),
            ArithOp::SUB => Ok(left_val - right_val),
            ArithOp::MUL => Ok(left_val * right_val),
            ArithOp::DIV => Ok(left_val / right_val),
        }
    }

    /// Evaluates a comparison expression
    fn comp_expr(
        &mut self,
        operator: &CompOp,
        left: &AstNode,
        right: &AstNode,
        line: i32,
    ) -> Result<bool, InterpreterError> {
        // If we're dealing with LT or GT, check arguments are numeric
        match operator {
            CompOp::LT => {
                if !left.is_numeric() || !right.is_numeric() {
                    return Err(InterpreterError::TypeError(format!(
                        "[Line {}]: Arguments to LT operator must evaluate to numbers.\n",
                        line
                    )));
                }
            }
            CompOp::GT => {
                if !left.is_numeric() || !right.is_numeric() {
                    return Err(InterpreterError::TypeError(format!(
                        "[Line {}]: Arguments to LT operator must evaluate to numbers.\n",
                        line
                    )));
                }
            }
            _ => {},
        };

        // Choose evaluation path based on trait implementation
        let left_val = match left {
            _ if left.is_word() => match left {
                AstNode::Word(word) => Value::Word(word.to_string()),
                AstNode::IdentRef(word) => self.eval_ident_ref_as_val(word).with_context(|| {
                    format!(
                        "[Line {}]: Failed to evaluate first argument to {}",
                        line, operator
                    )
                })?,
                _ => unreachable!("These are the only nodes for which is_word() is true"),
            },
            _ if left.is_numeric() => {
                Value::Float(self.eval_numeric_expression(left, line).with_context(|| {
                    format!(
                        "[Line {}]: Failed to evaluate first argument to {}",
                        line, operator
                    )
                })?)
            }
            _ if left.is_boolean() => {
                Value::Bool(self.eval_logic_expression(left, line).with_context(|| {
                    format!(
                        "[Line {}]: Failed to evaluate first argument to {}",
                        line, operator
                    )
                })?)
            }
            _ => {
                return Err(InterpreterError::TypeError(format!(
                "First argument to {0} is a statement, and cannot be passed as an argument to {0}",
                operator
            )))
            }
        };

        let right_val = match right {
            _ if right.is_word() => match right {
                AstNode::Word(word) => Value::Word(word.to_string()),
                AstNode::IdentRef(word) => self.eval_ident_ref_as_val(word).with_context(|| {
                    format!(
                        "[Line {}]: Failed to evaluate first argument to {}",
                        line, operator
                    )
                })?,
                _ => panic!("{:?}", right),
            },
            _ if right.is_numeric() => {
                Value::Float(self.eval_numeric_expression(right, line).with_context(|| {
                    format!(
                        "[Line {}]: Failed to evaluate second argument to {}",
                        line, operator
                    )
                })?)
            }
            _ if right.is_boolean() => {
                Value::Bool(self.eval_logic_expression(right, line).with_context(|| {
                    format!(
                        "[Line {}]: Failed to evaluate second argument to {}",
                        line, operator
                    )
                })?)
            }
            _ => {
                return Err(InterpreterError::TypeError(format!(
                "First argument to {0} is a statement, and cannot be passed as an argument to {0}",
                operator
            )))
            }
        };

        // Check that both types match
        if discriminant(&left_val) != discriminant(&right_val) {
            return Err(InterpreterError::TypeError(format!(
                "[Line {}]: Arguments to {} do not have matching types",
                line, operator
            )));
        }

        match operator {
            CompOp::EQ => Ok(left_val == right_val),
            CompOp::NE => Ok(left_val != right_val),
            CompOp::LT => Ok(left_val < right_val),
            CompOp::GT => Ok(left_val > right_val),
        }
    }

    /// Evaluates a boolean expression
    fn bool_expr(
        &mut self,
        operator: &BoolOp,
        left: &AstNode,
        right: &AstNode,
        line: i32,
    ) -> Result<bool, InterpreterError> {
        let left_val = self.eval_logic_expression(left, line).with_context(|| {
            format!(
                "[Line {}]: Failed to evaluate first argument to {}",
                line, operator
            )
        })?;
        let right_val = self.eval_logic_expression(right, line).with_context(|| {
            format!(
                "[Line {}]: Failed to evaluate second argument to {}",
                line, operator
            )
        })?;

        match operator {
            BoolOp::AND => Ok(left_val & right_val),
            BoolOp::OR => Ok(left_val || right_val),
        }
    }

    fn query(&mut self, query_kind: &QueryKind) -> f32 {
        match query_kind {
            QueryKind::XCOR => self.current_position.x_coordinate,
            QueryKind::YCOR => self.current_position.y_coordinate,
            QueryKind::HEADING => self.current_position.direction,
            QueryKind::COLOR => self.current_color as f32,
        }
    }

    /// Stores a raw string in the map (bind to itself)
    fn word(&mut self, var: &String) {
        // A clone is necessary here as we access to the same value,
        // and a smart pointer is likely excessive
        let ident_clone = String::from(var);
        self.environment
            .insert(var.to_string(), Value::Word(ident_clone));
    }

    /// Returns a reference to an identifiers value
    fn eval_ident_ref(&mut self, var: &String) -> Result<&Value, InterpreterError> {
        match self.environment.get(var) {
            Some(value) => Ok(value),
            _ => Err(InterpreterError::InvalidVariableRef(var.to_string())),
        }
    }

    /// Returns a copy of the identifiers value
    fn eval_ident_ref_as_val(&mut self, var: &String) -> Result<Value, InterpreterError> {
        match self.environment.get(var) {
            Some(value) => Ok(value.clone()),
            _ => Err(InterpreterError::InvalidVariableRef(var.to_string())),
        }
    }
}
