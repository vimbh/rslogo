use crate::parser::{AstNode, Binop, Boolop, Compop, Direction, PenPos, QueryKind};
use core::panic;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::rc::Rc;
use unsvg::{get_end_coordinates, Image, COLORS};

#[derive(Debug)]
pub struct Position {
    x_coordinate: f32,
    y_coordinate: f32,
    direction: f32,
}

#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
    Bool(bool),
}

impl std::ops::AddAssign for Value {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Value::Float(left), Value::Float(right)) => *left += right,
            _ => panic!("Unsupported addition-assignment: Must be Value::Float()"),
        }
    }
}

pub struct Interpreter<'a> {
    image: &'a mut Image,
    environment: HashMap<String, Value>,
    func_environment: HashMap<String, Rc<Vec<AstNode>>>, // Map each proc name to a list of its param names and a pointer to its executable body
    current_position: Position,
    current_color: usize,
    currently_drawing: bool,
}

impl<'a> Interpreter<'a> {
    // Constructor
    pub fn new(image: &'a mut Image) -> Self {
        Self {
            image,
            environment: HashMap::new(),
            func_environment: HashMap::new(),
            current_position: Position {
                x_coordinate: 0.0,
                y_coordinate: 0.0,
                direction: 0.0,
            },
            current_color: 7,         // Starts default w_hite
            currently_drawing: false, // Starts default penup (not drawing)
        }
    }

    pub fn run(&mut self, ast: &Vec<AstNode>) -> Result<&Image, String> {
        self.evaluate(ast);
        // Return image on success
        println!("Success!");
        Ok(self.image)
    }

    // Root Eval function over AST
    pub fn evaluate(&mut self, ast: &Vec<AstNode>) {
        for node in ast {
            match node {
                AstNode::MakeOp { var, expr } => self.make_eval(var.to_string(), expr),
                AstNode::PenPosUpdate { update_type, value } => {
                    self.set_position(update_type, value)
                }
                AstNode::PenStatusUpdate(new_drawing_status) => {
                    self.set_drawing_status(*new_drawing_status)
                }
                AstNode::PenColorUpdate(new_pen_color) => self.set_pen_color(new_pen_color),
                AstNode::IfStatement { condition, body } => self.eval_if_statement(condition, body),
                AstNode::WhileStatement { condition, body } => {
                    self.eval_while_statement(condition, body)
                }
                AstNode::AddAssign { var_name, expr } => self.eval_add_assign(var_name, expr),
                AstNode::Procedure { name, body } => {
                    self.create_procedure(name.to_string(), Rc::clone(body))
                }
                AstNode::ProcedureReference { name_ref, args } => {
                    self.eval_procedure(name_ref, args)
                }
                AstNode::DrawInstruction {
                    direction,
                    num_pixels,
                } => self.draw_line(direction, num_pixels),

                _ => panic!("Unexpected error while evaluating AST tree: {:?}", node),
            }
        }

        println!("{:?}\n", self.environment);
        //println!("{:?\n}", self.current_position);
        //println!("{:?\n}", self.currently_drawing);
        //println!("{:?\n}", self.current_color);
        println!("{:?\n}", self.func_environment);
    }

    fn get_relative_direction(&mut self, direction: &Direction) -> i32 {
        match direction {
            Direction::FORWARD => self.current_position.direction as i32,
            Direction::BACK => self.current_position.direction as i32 + 180,
            Direction::LEFT => self.current_position.direction as i32 - 90,
            Direction::RIGHT => self.current_position.direction as i32 + 90,
        }
    }

    fn draw_line(&mut self, direction: &Direction, value: &AstNode) {
        let num_pixels = self.eval_numeric_expression(value).unwrap();
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
                Err(error) => panic!("error drawing line, {:?}", error),
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

        dbg!(&self.current_position);
    }

    fn create_procedure(&mut self, name: String, body: Rc<Vec<AstNode>>) {
        // Add the function, args and body to the func environment
        self.func_environment.insert(name, body);
    }

    // eval_procedure requires copies to avoid borrow conflicts between fetching values in the
    // func_environment map and passing it's values to methods in the same instance.
    // To reduce copy overhead, func_environment holds Rc's to the function params & body,
    // so we can take cheap clones of Rc to pass to our methods.
    fn eval_procedure(&mut self, name_ref: &String, args: &Vec<AstNode>) {
        // Eval the args to bind the values
        self.evaluate(args);

        // Run the func
        if let Some(func_body) = self.func_environment.get_mut(name_ref) {
            let mut func_body_rc = Rc::clone(func_body);
            self.evaluate(func_body_rc.borrow_mut());
        } else {
            panic!("proc named {} does not exist", name_ref);
        }
    }

    fn eval_if_statement(&mut self, condition: &AstNode, body: &Vec<AstNode>) {
        let condition_is_true = self.eval_bool_expression(condition).unwrap();

        if condition_is_true {
            self.evaluate(body);
        }
    }

    fn eval_while_statement(&mut self, condition: &AstNode, body: &Vec<AstNode>) {
        let condition_is_true = self.eval_bool_expression(condition).unwrap();

        if condition_is_true {
            self.evaluate(body);
            self.eval_while_statement(condition, body);
        }
    }

    fn eval_add_assign(&mut self, var_name: &String, expr: &AstNode) {
        let assign_value = Value::Float(self.eval_numeric_expression(expr).unwrap());

        self.environment
            .entry(var_name.to_string())
            .and_modify(|var| *var += assign_value);
    }

    // get_relative_direction fn: evaluates any expr that could return a float (Num, Variable ref, Query, BinOp)
    fn eval_numeric_expression(&mut self, node: &AstNode) -> Result<f32, String> {
        match node {
            AstNode::Num(val) => Ok(*val),
            AstNode::IdentRef(var) => match self.eval_ref(var) {
                &Value::Float(num) => Ok(num),
                _ => panic!("Variable {} is bound to a Boolean value, not a Float.", var),
            },
            AstNode::BinaryOp {
                operator,
                left,
                right,
            } => Ok(self.eval_binary_op(operator, left, right)),
            AstNode::Query(query_kind) => Ok(self.query(query_kind)),
            _ => panic!("Value not recognised"),
        }
    }

    // get_relative_direction fn: evaluates any expr that could return a bool (Variable ref, BoolOp, CompOp)
    fn eval_bool_expression(&mut self, node: &AstNode) -> Result<bool, String> {
        match node {
            AstNode::IdentRef(var) => match self.eval_ref(var) {
                &Value::Bool(value) => Ok(value),
                _ => panic!("Variable {} is bound to a Float value, not a Boolean.", var),
            },
            AstNode::BooleanOp {
                operator,
                left,
                right,
            } => Ok(self.eval_bool_op(operator, left, right)),
            AstNode::ComparisonOp {
                operator,
                left,
                right,
            } => Ok(self.eval_comp_op(operator, left, right)),
            _ => panic!("Expression passed does not evaluate to a bool"),
        }
    }

    fn eval_binary_op(&mut self, operator: &Binop, left: &AstNode, right: &AstNode) -> f32 {
        let left_val = match left {
            AstNode::BinaryOp {
                operator,
                left,
                right,
            } => self.eval_binary_op(operator, left, right),
            _ => self.eval_numeric_expression(left).unwrap(),
        };

        let right_val = match right {
            AstNode::BinaryOp {
                operator,
                left,
                right,
            } => self.eval_binary_op(operator, left, right),
            _ => self.eval_numeric_expression(right).unwrap(),
        };

        match operator {
            Binop::ADD => left_val + right_val,
            Binop::SUB => left_val - right_val,
            Binop::MUL => left_val * right_val,
            Binop::DIV => left_val / right_val,
        }
    }

    fn eval_comp_op(&mut self, operator: &Compop, left: &AstNode, right: &AstNode) -> bool {
        let left_val = match left {
            AstNode::BinaryOp {
                operator,
                left,
                right,
            } => self.eval_binary_op(operator, left, right),
            _ => self.eval_numeric_expression(left).unwrap(),
        };

        let right_val = match right {
            AstNode::BinaryOp {
                operator,
                left,
                right,
            } => self.eval_binary_op(operator, left, right),
            _ => self.eval_numeric_expression(right).unwrap(),
        };

        match operator {
            Compop::EQ => left_val == right_val,
            Compop::NE => left_val != right_val,
            Compop::LT => left_val < right_val,
            Compop::GT => left_val > right_val,
        }
    }

    fn set_drawing_status(&mut self, new_drawing_status: bool) {
        self.currently_drawing = new_drawing_status;
    }

    fn set_pen_color(&mut self, value: &AstNode) {
        let float_val = self.eval_numeric_expression(value).unwrap();

        // Check precision & bounds before casting to an int color
        if float_val == (float_val as usize) as f32 && (0.0..=15.0).contains(&float_val) {
            self.current_color = float_val as usize;
        } else {
            panic!("SETPENCOLOR requires an integer from 0..15 as an argument");
        };
    }

    fn set_position(&mut self, update_type: &PenPos, value: &AstNode) {
        let val = self.eval_numeric_expression(value).unwrap();

        match update_type {
            PenPos::SETX => self.current_position.x_coordinate = val,
            PenPos::SETY => self.current_position.y_coordinate = val,
            PenPos::SETHEADING => self.current_position.direction = val,
            PenPos::TURN => self.current_position.direction += val,
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

    fn eval_bool_op(&mut self, operator: &Boolop, left: &AstNode, right: &AstNode) -> bool {
        let left_val = match left {
            AstNode::BooleanOp {
                operator,
                left,
                right,
            } => self.eval_bool_op(operator, left, right),
            AstNode::ComparisonOp {
                operator,
                left,
                right,
            } => self.eval_comp_op(operator, left, right),
            _ => self.eval_bool_expression(left).unwrap(),
        };

        let right_val = match right {
            AstNode::BooleanOp {
                operator,
                left,
                right,
            } => self.eval_bool_op(operator, left, right),
            AstNode::ComparisonOp {
                operator,
                left,
                right,
            } => self.eval_comp_op(operator, left, right),
            _ => self.eval_bool_expression(right).unwrap(),
        };

        match operator {
            Boolop::AND => left_val & right_val,
            Boolop::OR => left_val || right_val,
        }
    }

    fn eval_ref(&mut self, var: &String) -> &Value {
        match self.environment.get(var) {
            Some(value) => value,
            _ => panic!("This variable has not been instantiated: {}", &var),
        }
    }

    fn eval_ref_as_val(&mut self, var: &String) -> Value {
        self.eval_ref(var).clone()
    }

    fn make_eval(&mut self, var: String, expr: &AstNode) {
        let assign_val = match expr {
            AstNode::Num(val) => Value::Float(*val),
            AstNode::BinaryOp {
                operator,
                left,
                right,
            } => Value::Float(self.eval_binary_op(operator, left, right)),
            AstNode::ComparisonOp {
                operator,
                left,
                right,
            } => Value::Bool(self.eval_comp_op(operator, left, right)),
            AstNode::BooleanOp {
                operator,
                left,
                right,
            } => Value::Bool(self.eval_bool_op(operator, left, right)),
            AstNode::Query(query_kind) => Value::Float(self.query(query_kind)),
            AstNode::IdentRef(var) => self.eval_ref_as_val(var),
            _ => todo!("make not imp"),
        };

        // Add binding to map
        self.environment.insert(var, assign_val);
    }
}

#[allow(dead_code)]
fn main() {}
