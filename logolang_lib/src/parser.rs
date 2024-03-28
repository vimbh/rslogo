use crate::lexer::{Token, TokenKind};
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::string::ParseError;
use thiserror::Error;

////////////////////// ENUMS FOR PARSE /////////////////////////////////

#[derive(Debug)]
pub enum Binop {
    ADD,
    SUB,
    MUL,
    DIV,
}

#[derive(Debug)]
pub enum Compop {
    EQ,
    NE,
    LT,
    GT,
}

#[derive(Debug)]
pub enum Boolop {
    AND,
    OR,
}

#[derive(Debug)]
pub enum Direction {
    FORWARD,
    BACK,
    RIGHT,
    LEFT,
}

#[derive(Debug)]
pub enum PenPos {
    SETX,
    SETY,
    SETHEADING,
    TURN,
}

#[derive(Debug)]
pub enum QueryKind {
    XCOR,
    YCOR,
    HEADING,
    COLOR,
}

#[derive(Debug)]
pub enum AstNode {
    MakeOp {
        var: String,
        expr: Box<AstNode>,
    },
    BinaryOp {
        operator: Binop,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    ComparisonOp {
        operator: Compop,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    BooleanOp {
        operator: Boolop,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    DirecOp {
        direction: Direction,
        expr: Box<AstNode>,
    },
    IdentRef(String),
    AddAssign {
        var_name: String,
        expr: Box<AstNode>,
    },
    Ident(String),
    Num(f32),
    IfStatement {
        condition: Box<AstNode>,
        body: Box<Vec<AstNode>>,
    },
    WhileStatement {
        condition: Box<AstNode>,
        body: Box<Vec<AstNode>>,
    },
    PenStatusUpdate(bool),
    PenColorUpdate(Box<AstNode>),
    PenPosUpdate {
        update_type: PenPos,
        value: Box<AstNode>,
    },
    Query(QueryKind),
    Procedure {
        name: String,
        body: Rc<Vec<AstNode>>,
    },
    ProcedureReference {
        name_ref: String,
        args: Rc<Vec<AstNode>>,
    },
    DrawInstruction {
        direction: Direction,
        num_pixels: Box<AstNode>,
    },
}


/// Defines the return type of an expression
// is_boolean might seem redundant, but is needed to differentiate between
// Nodes which implements neither, vs those which implement is_numeric.
pub trait NodeType {
    fn is_numeric(&self) -> bool {
        false
    }
    fn is_boolean(&self) -> bool {
        false
    }
}

impl NodeType for AstNode {
    fn is_numeric(&self) -> bool {
        matches!(self, AstNode::Num(_) | AstNode::BinaryOp {..} | AstNode::Query(_)| AstNode::IdentRef(_))
    }

    fn is_boolean(&self) -> bool {
        matches!(&self, AstNode::ComparisonOp {..} | AstNode::BooleanOp {..} | AstNode::IdentRef(_))
    }
}



// Parser errors are SYNTACTIC errors, as opposed to SEMANTIC errors which are caught in the
// interpreter.
#[derive(Debug, Error)]
pub enum ParserError {
    
    // All <anyhow::Error> are cast to ParseError to catch the context chain
    #[error("{0}")]
    ParseError(String),

    #[error("Unexpected ending  while parsing program.")]
    UnexpectedEnding,


    #[error("\t[Line {0}]: Provided expression '{1}' will not return a float.\n")]
    NonNumericExpr(String, String),

    #[error("\t[Line {0}]: Provided expression '{1}' will not return a boolean.\n")]
    NonBooleanExpr(String, String),

    #[error("[Line {0}]: {1}")]   
    IncorrectArgType(String, String),
    
    #[error("[Line {0}]: {1}")]
    InvalidToken(String, String),

    #[error("{0} expression is missing parenthesis: expected {1}, received {2}")]
    MissingParenthesis(String, String, String),

    #[error("Referenced procedure {0} does not exist.")]
    InvalidProcReference(String),

}


impl From<anyhow::Error> for ParserError {
    fn from(error: anyhow::Error) -> Self {
        ParserError::ParseError(format!("{:?}", error))
    }
}

///////////////// PARSER METHODS /////////////////////////////////

#[allow(unused)]
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
    pub fn new() -> Self {
        Self {
            proc_arg_map: HashMap::new(),
        }
    }

    pub fn parse(&mut self, tokens: VecDeque<Token>) -> Result<Vec<AstNode>, ParserError> {
        let mut tokens = tokens;
        let mut ast = Vec::new();

        while tokens.front().is_some() {
            ast.push(self.expr(&mut tokens)?);
        }

        Ok(ast)
    }

    // Parses the tokens which form expressions in RSLOGO.
    fn expr(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        if let Some(token) = tokens.front() {
            //dbg!(&token);
            match &token.kind {
                // num_expressions
                TokenKind::BINOP => self.binary_op(tokens),
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
                TokenKind::IFSTMNT => self.if_statement(tokens),
                TokenKind::WHILESTMNT => self.while_statement(tokens),
                TokenKind::PENSTATUS => self.pen_status_update(tokens),
                TokenKind::PENCOLOR => self.pen_color_update(tokens),
                TokenKind::PENPOS => self.pen_position_update(tokens),
                TokenKind::PROCSTART => self.procedure(tokens),
                TokenKind::PROCNAME => self.procedure_reference(tokens),
                // Terminal                
                TokenKind::NUM => self.num(tokens),
                _ => Err(ParserError::InvalidToken(token.line.to_string(), format!("Variable '{0}' cannot be passed to this operation. Perhaps you meant :{0}?", token.value))),
            }
        } else {
            Err(ParserError::UnexpectedEnding)
        }
    }


    fn make_op(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        // Consume 'Make' token
        let make_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Consume next token
        let ident_token = tokens
            .pop_front()
            .ok_or_else(|| { ParserError::UnexpectedEnding })?;
    
        // Verify identifier token
        if let TokenKind::IDENT = ident_token.kind {
            // Continue
        } else {
            return Err(ParserError::IncorrectArgType(
                make_token.line.to_string(),
                format!("Invalid MAKE expression. MAKE did not receive a variable, instead receieved: {}.", ident_token.value).to_string(),
            )
            .into());
        }
    
        // Parse the expression which is bound to the identifier
        let expr = self.expr(tokens)
            .with_context(|| format!("\t[Line {}]: Invalid MAKE operation: Failed to parse expression provided to '{}'",
                                     ident_token.line, 
                                     ident_token.value))?;

        Ok(AstNode::MakeOp {
            var: ident_token.value,
            expr: Box::new(expr),
        })
    }


    // A binary expression either returns a float or a bool.
    fn binary_op(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        // Consume the operator token
        let operator_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        let left = self.expr(tokens)
            .with_context(|| format!("[Line{}]: The first argument to binary operator '{}' is invalid.",
                                     operator_token.line, 
                                     operator_token.value))?;

        let right = self.expr(tokens)
            .with_context(|| format!("[Line{}]: The second argument to binary operator '{}' is invalid",
                                     operator_token.line, 
                                     operator_token.value))?;
        
        // Check the validity of the provided expressions
        match operator_token.kind {
            TokenKind::BINOP => 
            {
                if !left.is_numeric() || !right.is_numeric() {
                    return Err(ParserError::NonNumericExpr(operator_token.line.to_string(), operator_token.value.to_string()));
                }   
            },
            // Note: if left and right are both IDENTREF's whos underlying values types are mismatched, they will not return an 
            // error here, as they return true for both is_boolean() and is_numeric(). This is intended, as the parser only checks 
            // for syntactic errors, while the interpreter will check for semantic errors.
            TokenKind::COMPOP =>
            {
                if !(left.is_boolean() && right.is_boolean()) && !(left.is_numeric() && right.is_numeric()) {
                    return Err(ParserError::NonBooleanExpr(operator_token.line.to_string(), operator_token.value.to_string()));                       }
            },
            TokenKind::BOOLOP => 
            {
                if !left.is_boolean() || !right.is_boolean() {
                    return Err(ParserError::NonBooleanExpr(operator_token.line.to_string(), operator_token.value.to_string()));
                }
            
            },
            _ => unreachable!("These are the only token kinds passed to the binary_op function"),
        }

        
        match operator_token.kind {
            TokenKind::BINOP => 
                Ok(AstNode::BinaryOp {
                    operator: match operator_token.value.as_str() {
                        "+" => Binop::ADD,
                        "-" => Binop::SUB,
                        "*" => Binop::MUL,
                        "/" => Binop::DIV,
                        _ => unreachable!("Lexer only produces these binary operators"),
                    },
                    left: Box::new(left),
                    right: Box::new(right),
            }),
            TokenKind::COMPOP =>
                Ok(AstNode::ComparisonOp {
                    operator: match operator_token.value.as_str() {
                        "EQ" => Compop::EQ,
                        "NE" => Compop::NE,
                        "LT" => Compop::LT,
                        "GT" => Compop::GT,
                        _ => unreachable!("Lexer only produces these binary operators"),
                    },
                    left: Box::new(left),
                    right: Box::new(right),
            }),
            TokenKind::BOOLOP => 
                Ok(AstNode::BooleanOp {
                    operator: match operator_token.value.as_str() {
                        "AND" => Boolop::AND,
                        "OR" => Boolop::OR,
                        _ => unreachable!("Lexer only produces these binary operators"),
                    },
                    left: Box::new(left),
                    right: Box::new(right),
                }),
            _ => unreachable!("fn binary_op only retrieves arguments of these types")

        }        
   
    }

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

    fn ident_ref(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let ident_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        let ident_value = ident_token.value;
        Ok(AstNode::IdentRef(ident_value))
    }

    fn pen_position_update(
        &mut self,
        tokens: &mut VecDeque<Token>,
    ) -> Result<AstNode, ParserError> {
        let pos_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the arg which was provided to the position setter
        let parsed_value = self.expr(tokens)?;

        Ok(AstNode::PenPosUpdate {
            update_type: match pos_token.value.as_str() {
                "SETX" => PenPos::SETX,
                "SETY" => PenPos::SETY,
                "TURN" => PenPos::TURN,
                "SETHEADING" => PenPos::SETHEADING,
                _ => unreachable!("Lexer only produces these binary operators"),
            },
            value: Box::new(parsed_value),
        })
    }

    fn pen_status_update(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let status_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        Ok(AstNode::PenStatusUpdate(
            match status_token.value.as_str() {
                "PENUP" => false,
                "PENDOWN" => true,
                _ => unreachable!("Lexer only produces these binary operators"),
            },
        ))
    }

    fn pen_color_update(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the arg to the position setter
        let parsed_value = self.expr(tokens)?;

        Ok(AstNode::PenColorUpdate(Box::new(parsed_value)))
    }

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

    fn if_statement(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the following condition (some expression which returns a bool)
        let condition = self.expr(tokens)?;
        // Check returned val was bool
        match condition {
            AstNode::BooleanOp { .. } | AstNode::ComparisonOp { .. } | AstNode::IdentRef(_) => {} // Continue
            _ => {
                todo!();
                //return Err(ParserError::IncorrectArgType(
                //    "IF".to_string(),
                //    "Bool".to_string(),
                //    "Float".to_string(),
                //))
            }
        }

        // Parse body opening parenthesis
        let l_paren_token = tokens
            .pop_front()
            .ok_or_else(|| ParserError::UnexpectedEnding);

        if l_paren_token.expect("Already verified above").kind != TokenKind::LPAREN {
            return Err(ParserError::MissingParenthesis(
                "IF".to_string(),
                "Expr".to_string(),
                "[".to_string(),
            ));
        };

        let mut body_tokens = Vec::<AstNode>::new();

        // Parse body until closing parenthesis is seen
        while let Some(token) = tokens.front() {
            if token.kind == TokenKind::RPAREN {
                break;
            }
            body_tokens.push(self.expr(tokens)?);
        }

        tokens.pop_front().expect("Already verified above");

        Ok(AstNode::IfStatement {
            condition: Box::new(condition),
            body: Box::new(body_tokens),
        })
    }

    fn while_statement(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the following condition (some expression which returns a bool)
        let condition = self.expr(tokens)?;

        match condition {
            AstNode::BooleanOp { .. } | AstNode::ComparisonOp { .. } | AstNode::IdentRef(_) => {} // Continue
            _ => {
                todo!();
                //return Err(ParserError::IncorrectArgType(
                //    "WHILE".to_string(),
                //    "Bool".to_string(),
                //    "Float".to_string(),
                //))
            }
        }

        // Parse body opening parenthesis
        let l_paren_token = tokens
            .pop_front()
            .ok_or_else(|| ParserError::UnexpectedEnding)
            .expect("Already verified");

        if l_paren_token.kind != TokenKind::LPAREN {
            return Err(ParserError::MissingParenthesis(
                "IF".to_string(),
                l_paren_token.value,
                "[".to_string(),
            ));
        };

        let mut body_tokens = Vec::<AstNode>::new();

        // Parse body until closing parenthesis is seen
        while let Some(token) = tokens.front() {
            if token.kind == TokenKind::RPAREN {
                break;
            }
            body_tokens.push(self.expr(tokens)?);
        }

        tokens.pop_front().expect("Already verified above");

        Ok(AstNode::WhileStatement {
            condition: Box::new(condition),
            body: Box::new(body_tokens),
        })
    }

    fn add_assign(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        // Consume the operator token
        tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the next token
        let var_token = tokens
            .pop_front()
            .ok_or_else(|| ParserError::UnexpectedEnding)
            .expect("Already verified");

        if var_token.kind != TokenKind::IDENT {
            todo!();
            //return Err(ParserError::IncorrectArgType(
            //    "ADDASSIGN".to_string(),
            //    "Variable reference".to_string(),
            //    "Expr".to_string(),
            //));
        }

        let val = self.expr(tokens)?;

        // Val must return a float
        match val {
            AstNode::Num(_)
            | AstNode::ComparisonOp { .. }
            | AstNode::IdentRef(_)
            | AstNode::Query(_) => {}
            _ => {
                todo!();
                //return Err(ParserError::IncorrectArgType(
                //    "ASSASSIGN".to_string(),
                //    "Float".to_string(),
                //    "Bool/Non-returning operator".to_string(),
                //));
            }
        }

        Ok(AstNode::AddAssign {
            var_name: var_token.value,
            expr: Box::new(val),
        })
    }

    pub fn procedure(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse Proc Name
        let proc_name_token = tokens
            .pop_front()
            .ok_or_else(|| ParserError::UnexpectedEnding)
            .expect("Verified above");

        if proc_name_token.kind != TokenKind::PROCNAME {
            //return Err(ParserError::IncorrectArgType(
            //    "PROCEDURE".to_string(),
            //    "<Procedure Name>".to_string(),
            //    proc_name_token.value,
            //));
        }

        let mut arg_tokens = Vec::<String>::new();

        // The following token(s) must be: (>=1 IDENTS) XOR (Proc Body)
        // Parse Proc args until a non-IDENT token is seen
        while let Some(token) = tokens.front() {
            if token.kind != TokenKind::IDENT {
                break;
            }
            arg_tokens.push(
                tokens
                    .pop_front()
                    .ok_or_else(|| ParserError::UnexpectedEnding)
                    .expect("Verified above")
                    .value,
            );
        }

        // tokens.front() is now the first token in the body
        let mut body_tokens = Vec::<AstNode>::new();

        // Parse body until END token is seen
        while let Some(token) = tokens.front() {
            if token.kind == TokenKind::PROCEND {
                break;
            }
            body_tokens.push(self.expr(tokens)?);
        }

        let end_token = tokens
            .pop_front()
            .ok_or_else(|| ParserError::UnexpectedEnding)
            .expect("Verified Above");

        if end_token.kind != TokenKind::PROCEND {
            todo!();
            //return Err(ParserError::IncorrectArgType(
            //    "PROCEDURE".to_string(),
            //    "Float or variable reference".to_string(),
            //    end_token.value,
            //));
        }

        // ADD parameter list to proc_arg_map
        self.proc_arg_map
            .insert(proc_name_token.value.clone(), Rc::new(arg_tokens));

        Ok(AstNode::Procedure {
            name: proc_name_token.value,
            body: Rc::new(body_tokens),
        })
    }

    // When a procedure reference is made, directly bind the provided arguments to the functions
    // parameters.
    pub fn procedure_reference(
        &mut self,
        tokens: &mut VecDeque<Token>,
    ) -> Result<AstNode, ParserError> {
        let proc_name = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        let mut arg_list = Vec::<AstNode>::new();

        // The type of args to procedures cannot be verified until the evaluation stage
        // as their use cases in the body are yet to be verified. Handle errors in evaluator
        let param_list = match self.proc_arg_map.get(&proc_name.value) {
            Some(value) => value,
            None => return Err(ParserError::InvalidProcReference(proc_name.value)),
        };

        let param_list_rc = Rc::clone(param_list);

        for i in 0..param_list_rc.len() {
            let arg_value = self.expr(tokens)?;
            arg_list.push({
                AstNode::MakeOp {
                    var: param_list_rc
                        .get(i)
                        .expect("Looping within the bounds of arg_rc by definition")
                        .to_string(),
                    expr: Box::new(arg_value),
                }
            });
        }

        Ok(AstNode::ProcedureReference {
            name_ref: proc_name.value,
            args: Rc::new(arg_list),
        })
    }

    fn draw_line(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, ParserError> {
        let direction_token = tokens
            .pop_front()
            .expect("Token must have been verified to be passed to fn");

        // Parse the following condition (some expression which returns a float)
        let num_pixels_token = self.expr(tokens)?;

        match num_pixels_token {
            AstNode::Num(_)
            | AstNode::IdentRef(_)
            | AstNode::Query(_)
            | AstNode::BinaryOp { .. } => {}
            _ => {
                todo!();
                //return Err(ParserError::IncorrectArgType(
                //    "FORWARD/BACK/LEFT/RIGHT".to_string(),
                //    "expression which evaluates to a float".to_string(),
                //    "Non-float value".to_string(),
                //))
            }
        }

        Ok(AstNode::DrawInstruction {
            direction: match direction_token.value.as_str() {
                "FORWARD" => Direction::FORWARD,
                "BACK" => Direction::BACK,
                "LEFT" => Direction::LEFT,
                "RIGHT" => Direction::RIGHT,
                _ => unreachable!("Lexer only produces these directions"),
            },
            num_pixels: Box::new(num_pixels_token),
        })
    }


}
