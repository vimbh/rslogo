use crate::lexer::{Token, TokenKind};
use crate::logolang_errors::ParserError;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;


/// Represents arithmetic operations
#[derive(Debug)]
pub enum ArithOp {
    ADD,
    SUB,
    MUL,
    DIV,
}

/// Represents comparison operations
#[derive(Debug)]
pub enum CompOp {
    EQ,
    NE,
    LT,
    GT,
}

/// Represents boolean operations
#[derive(Debug)]
pub enum BoolOp {
    AND,
    OR,
}

/// Represents drawing directions
#[derive(Debug)]
pub enum Direction {
    FORWARD,
    BACK,
    RIGHT,
    LEFT,
}

/// Represents pen position
#[derive(Debug)]
pub enum PenPos {
    SETX,
    SETY,
    SETHEADING,
    TURN,
}

/// Represents types of queries
#[derive(Debug)]
pub enum QueryKind {
    XCOR,
    YCOR,
    HEADING,
    COLOR,
}

/// Represents abstract syntax tree nodes
// Line corresponds to line number at the start of the expression/statement
#[derive(Debug)]
pub enum AstNode {
    /// Make statements
    MakeStmnt {
        var: String,
        expr: Box<AstNode>,
        line: i32,
    },
    /// Arithmetic expressions
    ArithExpr {
        operator: ArithOp,
        left: Box<AstNode>,
        right: Box<AstNode>,
        line: i32,
    },
    /// Comparison expressions
    CompExpr {
        operator: CompOp,
        left: Box<AstNode>,
        right: Box<AstNode>,
        line: i32,
    },
    /// Boolean expressions
    BoolExpr {
        operator: BoolOp,
        left: Box<AstNode>,
        right: Box<AstNode>,
        line: i32,
    },
    /// Reference to identifier
    IdentRef(String),
    /// Addition assignment
    AddAssign {
        var_name: String,
        expr: Box<AstNode>,
        line: i32,
    },
    /// Identifier
    Ident {
        var_name: String,
        line: i32,
    },
    /// Number
    Num(f32),
    /// If statement
    IfStmnt {
        condition: Box<AstNode>,
        body: Box<Vec<AstNode>>,
        line: i32,
    },
    /// While statement
    WhileStmnt {
        condition: Box<AstNode>,
        body: Box<Vec<AstNode>>,
        line: i32,
    },
    /// Pen status (penup/pendown)
    PenStatusUpdate(bool),
    PenColorUpdate {
        color: Box<AstNode>,
        line: i32,
    },
    /// Pen position 
    PenPosUpdate {
        update_type: PenPos,
        value: Box<AstNode>,
        line: i32,
    },
    /// Type of query
    Query(QueryKind),
    /// Procedure definition
    Procedure {
        name: String,
        body: Rc<Vec<AstNode>>,
    },
    /// Reference to procedure
    ProcedureRef {
        name_ref: String,
        args: Rc<Vec<AstNode>>,
        line: i32,
    },
    /// Draw instruction (direction instructions)
    DrawInstruction {
        direction: Direction,
        num_pixels: Box<AstNode>,
        line: i32,
    },
    /// String literals
    Word(String),
}

/// A trait implementation that defines the operations inherited by the node
pub trait NodeType {
    fn is_numeric(&self) -> bool {
        false
    }
    fn is_boolean(&self) -> bool {
        false
    }
    fn is_word(&self) -> bool {
        false
    }
}

impl NodeType for AstNode {
    fn is_numeric(&self) -> bool {
        matches!(
            self,
            AstNode::Num(_) | AstNode::ArithExpr { .. } | AstNode::Query(_) | AstNode::IdentRef(_)
        )
    }
    fn is_boolean(&self) -> bool {
        matches!(
            &self,
            AstNode::CompExpr { .. } | AstNode::BoolExpr { .. } | AstNode::IdentRef(_)
        )
    }
    fn is_word(&self) -> bool {
        matches!(&self, AstNode::Word(_) | AstNode::IdentRef(_))
    }
}


/// Parser for the RSLOGO language
pub struct Parser {
    // Keep track of the parameter names for each procedure
    proc_arg_map: HashMap<String, Rc<Vec<String>>>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    /// Returns a new Parser instance
    pub fn new() -> Self {
        Self {
            proc_arg_map: HashMap::new(),
        }
    }

    /// Parses a given sequence of tokens into an abstract syntax tree (AST), as a collection of
    /// AST nodes.
    /// Returns a `ParserError` if any syntactic errors are encountered.
    pub fn parse(&mut self, tokens: VecDeque<Token>) -> Result<Vec<AstNode>, ParserError> {
        let mut tokens = tokens;
        let mut ast = Vec::new();

        while tokens.front().is_some() {
            ast.push(self.expr(&mut tokens)?);
        }

        Ok(ast)
    }

    /// Parses tokens recursively to return valid AST nodes.
    fn expr(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        if let Some(token) = tokens.front() {
            match &token.kind {
                // num_expressions
                TokenKind::ARITHOP => self.binary_op(tokens),
                TokenKind::QUERY => self.query(tokens),
                // bool_expressions
                TokenKind::COMPOP => self.binary_op(tokens),
                TokenKind::BOOLOP => self.binary_op(tokens),
                // num or bool expression
                TokenKind::IDENTREF => self.ident_ref(tokens),
                // statements
                TokenKind::MAKEOP => self.make_op(tokens),
                TokenKind::ADDASSIGN => self.add_assign(tokens),
                TokenKind::DIRECTION => self.draw_line(tokens),
                TokenKind::IFSTMNT => self.if_while_statement(tokens),
                TokenKind::WHILESTMNT => self.if_while_statement(tokens),
                TokenKind::PENSTATUS => self.pen_status_update(tokens),
                TokenKind::PENCOLOR => self.pen_color_update(tokens),
                TokenKind::PENPOS => self.pen_position_update(tokens),
                TokenKind::PROCSTART => self.procedure(tokens),
                TokenKind::PROCNAME => self.procedure_reference(tokens),
                // Terminal
                TokenKind::NUM => self.num(tokens),
                // If an ident it received here, it is not bound: treat it as a raw string
                TokenKind::IDENT => self.raw_string(tokens),
                _ => unreachable!("LPAREN, RPAREN & PROCEND are handled within PROCSTART match"),
            }
        } else {
            Err(ParserError::UnexpectedEnding)
        }
    }

    /// Parses tokens into a MAKE statement node
    fn make_op(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        // Consume 'Make' token
        let make_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Consume next token
        let ident_token = tokens.pop_front().ok_or(ParserError::UnexpectedEnding)?;

        // Verify identifier token
        if TokenKind::IDENT != ident_token.kind {
            return Err(ParserError::IncorrectArgType(
                make_token.line.to_string(),
                format!("Invalid MAKE expression. MAKE did not receive a variable, instead receieved: {}.", ident_token.value).to_string(),
            ));
        }

        // Parse the expression which is bound to the identifier
        let expr = self.expr(tokens).with_context(|| {
            format!(
                "\t[Line {}]: Invalid MAKE operation: Failed to parse expression provided to '{}'",
                ident_token.line, ident_token.value
            )
        })?;

        // The value for which a identifier is bound must be an expression (returns a bool or float)
        if !expr.is_numeric() && !expr.is_boolean() && !expr.is_word() {
            return Err(ParserError::IncorrectArgType(
                    ident_token.line.to_string(),
                    format!("Invalid MAKE statement. {} received an argument which does not return a float value or a boolean value."
                            ,ident_token.value)));
        }

        Ok(AstNode::MakeStmnt {
            var: ident_token.value,
            expr: Box::new(expr),
            line: ident_token.line,
        })
    }

    /// Parses tokens into a binary expression node: An arithmetic expression, 
    /// comparison expression or a boolean expression.
    /// All binary expressions return a terminal value: a float or a bool.
    fn binary_op(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        // Consume the operator token
        let operator_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        let left = self.expr(tokens).with_context(|| {
            format!(
                "[Line{}]: The first argument to binary operator '{}' is invalid.",
                operator_token.line, operator_token.value
            )
        })?;

        let right = self.expr(tokens).with_context(|| {
            format!(
                "[Line{}]: The second argument to binary operator '{}' is invalid",
                operator_token.line, operator_token.value
            )
        })?;

        // Check the validity of the provided expressions
        match operator_token.kind {
            TokenKind::ARITHOP => {
                if !left.is_numeric() || !right.is_numeric() {
                    return Err(ParserError::NonNumericExpr(
                        operator_token.line.to_string(),
                        operator_token.value.to_string(),
                    ));
                }
            }
            // Note: if left and right are both IDENTREF's whos underlying values types are mismatched, they will not return an
            // error here, as they return true for both is_boolean() and is_numeric(). This is intended, as the parser only checks
            // for syntactic errors, while the interpreter will check for semantic errors.
            TokenKind::COMPOP => {
                if !(left.is_boolean() && left.is_numeric())
                    && !(right.is_boolean() && right.is_numeric())
                    && (left.is_boolean() != right.is_boolean()
                        || left.is_numeric() != right.is_numeric()
                        || left.is_word() != right.is_word())
                {
                    return Err(ParserError::NonBooleanExpr(
                        operator_token.line.to_string(),
                        operator_token.value.to_string(),
                    ));
                }
            }
            TokenKind::BOOLOP => {
                if !left.is_boolean() || !right.is_boolean() {
                    return Err(ParserError::NonBooleanExpr(
                        operator_token.line.to_string(),
                        operator_token.value.to_string(),
                    ));
                }
            }
            _ => unreachable!("These are the only token kinds passed to the binary_op function"),
        }

        // Construct result depending on operator type
        match operator_token.kind {
            TokenKind::ARITHOP => Ok(AstNode::ArithExpr {
                operator: match operator_token.value.as_str() {
                    "+" => ArithOp::ADD,
                    "-" => ArithOp::SUB,
                    "*" => ArithOp::MUL,
                    "/" => ArithOp::DIV,
                    _ => unreachable!("Lexer only produces these binary operators"),
                },
                left: Box::new(left),
                right: Box::new(right),
                line: operator_token.line,
            }),
            TokenKind::COMPOP => Ok(AstNode::CompExpr {
                operator: match operator_token.value.as_str() {
                    "EQ" => CompOp::EQ,
                    "NE" => CompOp::NE,
                    "LT" => CompOp::LT,
                    "GT" => CompOp::GT,
                    _ => unreachable!("Lexer only produces these binary operators"),
                },
                left: Box::new(left),
                right: Box::new(right),
                line: operator_token.line,
            }),
            TokenKind::BOOLOP => Ok(AstNode::BoolExpr {
                operator: match operator_token.value.as_str() {
                    "AND" => BoolOp::AND,
                    "OR" => BoolOp::OR,
                    _ => unreachable!("Lexer only produces these binary operators"),
                },
                left: Box::new(left),
                right: Box::new(right),
                line: operator_token.line,
            }),
            _ => unreachable!("fn binary_op only retrieves arguments of these types"),
        }
    }

    /// Parses a token into a number node.
    fn num(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let num_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        let num_value = num_token
            .value
            .parse::<f32>()
            .expect("Num tokens are already verified as parsing to f32 in lexer");
        Ok(AstNode::Num(num_value))
    }
    /// Parses a token into a identifier reference (the value bound a the identifier) node
    fn ident_ref(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let ident_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        let ident_value = ident_token.value;
        Ok(AstNode::IdentRef(ident_value))
    }
    /// Parses tokens into a pen position update node (setx, sety, turn, setheading)
    fn pen_position_update(
        &mut self,
        tokens: &mut VecDeque<Token>,
    ) -> Result<AstNode, ParserError> {
        let pos_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the arg which was provided to the position setter
        let parsed_value = self.expr(tokens)?;

        // Handle extra arguments
        check_extra_args(tokens, pos_token.line)
            .with_context(|| format!("Error parsing '{}' expression", pos_token.value))?;

        Ok(AstNode::PenPosUpdate {
            update_type: match pos_token.value.as_str() {
                "SETX" => PenPos::SETX,
                "SETY" => PenPos::SETY,
                "TURN" => PenPos::TURN,
                "SETHEADING" => PenPos::SETHEADING,
                _ => unreachable!("Lexer only produces these binary operators"),
            },
            value: Box::new(parsed_value),
            line: pos_token.line,
        })
    }
    /// Parses tokens into a pen status update node (penup / pendown)
    fn pen_status_update(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let status_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Handle extra arguments
        check_extra_args(tokens, status_token.line)
            .with_context(|| format!("Error parsing '{}' expression", status_token.value))?;

        Ok(AstNode::PenStatusUpdate(
            match status_token.value.as_str() {
                "PENUP" => false,
                "PENDOWN" => true,
                _ => unreachable!("Lexer only produces these binary operators"),
            },
        ))
    }
    /// Parses tokens into a pen colour update node
    fn pen_color_update(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let col_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the arg to the position setter
        let parsed_value = self.expr(tokens)?;

        // Handle extra arguments
        check_extra_args(tokens, col_token.line)
            .with_context(|| format!("Error parsing '{}' expression", col_token.value))?;

        Ok(AstNode::PenColorUpdate {
            color: Box::new(parsed_value),
            line: col_token.line,
        })
    }
    /// Parses tokens into a query node (xcor, ycor, heading, color)
    fn query(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let query_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        Ok(AstNode::Query(match query_token.value.as_str() {
            "XCOR" => QueryKind::XCOR,
            "YCOR" => QueryKind::YCOR,
            "HEADING" => QueryKind::HEADING,
            "COLOR" => QueryKind::COLOR,
            _ => unreachable!("Lexer only produces these binary operators"),
        }))
    }
    /// Parses tokens into an if / while statement node
    fn if_while_statement(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let if_while_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        let statement_type = if if_while_token.kind == TokenKind::IFSTMNT {
            "IF"
        } else {
            "WHILE"
        };
        // Parse the condition which if statement checks
        let condition_token = self.expr(tokens).with_context(|| {
            format!(
                "\t[Line {0}]: Invalid {1} statement: Failed to parse expression provided to {1}",
                if_while_token.line, statement_type
            )
        })?;

        // Check the validity of the provided expressions
        if !condition_token.is_boolean() {
            return Err(ParserError::NonBooleanExpr(
                if_while_token.line.to_string(),
                if_while_token.value.to_string(),
            ));
        }

        // Parse body opening parenthesis
        let l_paren_token = tokens
            .pop_front()
            .ok_or(ParserError::UnexpectedEnding)
            .expect("Checked validity in ok_or");

        if l_paren_token.kind != TokenKind::LPAREN {
            return Err(ParserError::MissingParenthesis(
                l_paren_token.line.to_string(),
                if_while_token.value.to_string(),
                "[".to_string(),
                l_paren_token.value.to_string(),
            ));
        };

        // Store the expressions/statements within the body of the if statement
        let mut body_tokens = Vec::<AstNode>::new();

        // Parse body until closing parenthesis is seen.
        while let Some(token) = tokens.front() {
            if token.kind == TokenKind::RPAREN {
                break;
            }
            let current_expr = self.expr(tokens).with_context(|| {
                format!(
                    "\t[Line {}]: Invalid expression found within {} statement body.",
                    l_paren_token.line, statement_type
                )
            })?;
            body_tokens.push(current_expr);
        }

        // Verify if we saw the closing parenthesis, or if we ran out of tokens
        let r_paren_token = tokens
            .pop_front()
            .ok_or(ParserError::UnexpectedEnding)
            .expect("Checked validity in ok_or");

        if r_paren_token.kind != TokenKind::RPAREN {
            return Err(ParserError::MissingParenthesis(
                r_paren_token.line.to_string(),
                if_while_token.value.to_string(),
                "]".to_string(),
                l_paren_token.value.to_string(),
            ));
        };

        // Return node based on token kind
        if if_while_token.kind == TokenKind::IFSTMNT {
            Ok(AstNode::IfStmnt {
                condition: Box::new(condition_token),
                body: Box::new(body_tokens),
                line: if_while_token.line,
            })
        } else {
            Ok(AstNode::WhileStmnt {
                condition: Box::new(condition_token),
                body: Box::new(body_tokens),
                line: if_while_token.line,
            })
        }
    }
    /// Parses tokens into an addition assignment node
    fn add_assign(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        // Consume the operator token
        tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the next token
        let var_token = tokens
            .pop_front()
            .ok_or(ParserError::UnexpectedEnding)
            .expect("Checked validity in ok_or");

        // Check valid identifier was provided to assign to
        if var_token.kind != TokenKind::IDENT {
            return Err(ParserError::InvalidAddAssign(
                var_token.line.to_string(),
                var_token.value.to_string(),
            ));
        }

        // Parse the expression which is bound to the identifier
        let value_token = self.expr(tokens)
            .with_context(|| format!("\t[Line {}]: Invalid ADDASSIGN operation: Failed to parse expression provided to '{}'",
                                     var_token.line,
                                     var_token.value))?;

        // Check the validity of the provided expression
        if !value_token.is_numeric() {
            return Err(ParserError::NonNumericExpr(
                var_token.line.to_string(),
                var_token.value.to_string(),
            ));
        }

        Ok(AstNode::AddAssign {
            var_name: var_token.value,
            expr: Box::new(value_token),
            line: var_token.line,
        })
    }
    /// Parses tokens into a procedure definition node
    pub fn procedure(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse Proc Name
        let proc_name_token = tokens
            .pop_front()
            .ok_or(ParserError::UnexpectedEnding)
            .expect("Checked validity in ok_or");

        if proc_name_token.kind != TokenKind::PROCNAME {
            return Err(ParserError::InvalidProcName(
                proc_name_token.line.to_string(),
                proc_name_token.value.to_string(),
            ));
        }

        // Store procedure parameters
        let mut arg_tokens = Vec::<String>::new();

        // The following token(s) must be: (>=1 IDENTS) XOR (Procedure Body)
        // Parse procedure parameters until a non-IDENT token is seen
        while let Some(token) = tokens.front() {
            if token.kind != TokenKind::IDENT {
                break;
            }
            arg_tokens.push(
                tokens
                    .pop_front()
                    .ok_or(ParserError::UnexpectedEnding)
                    .expect("Checked validity in ok_or")
                    .value,
            );
        }

        // Store procedure body
        let mut body_tokens = Vec::<AstNode>::new();

        // Parse body until END token is seen
        while let Some(token) = tokens.front() {
            if token.kind == TokenKind::PROCEND {
                break;
            }
            let current_expr = self.expr(tokens).with_context(|| {
                format!(
                    "\t[Line {}]: Invalid expression found within Procedure {}'s body.",
                    proc_name_token.line, proc_name_token.value
                )
            })?;
            body_tokens.push(current_expr);
        }

        // Verify if we saw the END token, or if we ran out of tokens
        tokens
            .pop_front()
            .ok_or(ParserError::UnexpectedEnding)
            .expect("Checked validity in ok_or");

        // Add to our procedure map: <procedure_name, Rc<<parameter_list>>
        // so we can bind arguments to each parameter if a procedure reference is seen later.
        // See procedure_reference for explanation of Rc usage
        self.proc_arg_map
            .insert(proc_name_token.value.clone(), Rc::new(arg_tokens));

        Ok(AstNode::Procedure {
            name: proc_name_token.value,
            body: Rc::new(body_tokens),
        })
    }

    /// Parses tokens into a procedure reference node.
    // When a procedure reference is made, directly bind the provided arguments to the functions
    // parameters.
    pub fn procedure_reference(
        &mut self,
        tokens: &mut VecDeque<Token>,
    ) -> Result<AstNode, ParserError> {
        let proc_name = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // We bind each parameter in the procedure to each argument in the procedure reference
        // Store each required binding as a Make statement to be executed in the interpreter
        let mut binding_list = Vec::<AstNode>::new();

        // Get parameter list
        let param_list = match self.proc_arg_map.get(&proc_name.value) {
            Some(value) => value,
            None => {
                return Err(ParserError::InvalidProcReference(
                    proc_name.line.to_string(),
                    proc_name.value,
                ))
            }
        };

        let param_list_rc = Rc::clone(param_list);

        // param_list has an exclusive borrow over the maps Vec<String>. Below, we access
        // self.expr(), which itself may mutate the map. As we assume procedures are never defined
        // (but can be called) within another procedure, we can assure self.expr() will never
        // mutate the map, and will at most read from it, in the case another procedure is referenced.
        // As such, we take a Rc over the param_list to allow shared access to the map.
        for i in 0..param_list_rc.len() {
            let arg_value = self.expr(tokens).with_context(|| {
                format!(
                    "\t[Line {}]: Invalid argument provided to procedure '{}'\n",
                    proc_name.line, proc_name.value
                )
            })?;

            binding_list.push({
                AstNode::MakeStmnt {
                    var: param_list_rc
                        .get(i)
                        .expect("Looping within the bounds of arg_rc by definition")
                        .to_string(),
                    expr: Box::new(arg_value),
                    line: proc_name.line,
                }
            });
        }

        Ok(AstNode::ProcedureRef {
            name_ref: proc_name.value,
            args: Rc::new(binding_list),
            line: proc_name.line,
        })
    }

    /// Parses tokens into a drawing node (forward, back, left, right)
    fn draw_line(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let direction_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the value to the direction
        let num_pixels = self.expr(tokens).with_context(|| {
            format!(
                "\t[Line {}]: Invalid argument to {}\n",
                direction_token.line, direction_token.value
            )
        })?;

        // Check the validity of the provided expressions
        if !num_pixels.is_numeric() {
            return Err(ParserError::NonNumericExpr(
                direction_token.line.to_string(),
                direction_token.value.to_string(),
            ));
        }

        // Handle extra arguments
        check_extra_args(tokens, direction_token.line)
            .with_context(|| format!("Error parsing '{}' expression", direction_token.value))?;

        Ok(AstNode::DrawInstruction {
            direction: match direction_token.value.as_str() {
                "FORWARD" => Direction::FORWARD,
                "BACK" => Direction::BACK,
                "LEFT" => Direction::LEFT,
                "RIGHT" => Direction::RIGHT,
                _ => unreachable!("Lexer only produces these directions"),
            },
            num_pixels: Box::new(num_pixels),
            line: direction_token.line,
        })
    }

    /// Parses a token into a word node (string literal)
    // Unbound variables which are not nested within an expression/statement are treated as raw
    // strings ('words')
    fn raw_string(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let word = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        Ok(AstNode::Word(word.value.to_string()))
    }
}

/// Returns an error if statement receives more arguments than expected.
fn check_extra_args(tokens: &mut VecDeque<Token>, line_number: i32) -> Result<(), ParserError> {
    let mut extra_args = Vec::<String>::new();

    while let Some(token) = tokens.pop_front() {
        if token.line == line_number {
            extra_args.push(format!("\"{}\"", token.value));
        } else {
            tokens.push_front(token);
            break;
        }
    }

    if extra_args.is_empty() {
        Ok(())
    } else {
        Err(ParserError::ExtraArguments(
            line_number.to_string(),
            extra_args.join(", "),
        ))
    }
}

