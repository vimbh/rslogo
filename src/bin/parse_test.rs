#![allow(unused_imports)]
#![allow(dead_code)]

use std::io::{self, BufRead, BufReader, ErrorKind};
use std::fs::File;
use std::fmt;
use std::error::Error;
use std::rc::Rc;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, alpha1, multispace0},
    combinator::{map,map_res},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};
use crate::lex_test::Token;
use crate::lex_test::TokenKind;
use std::collections::VecDeque;
use std::collections::HashMap;

////////////////////// ENUMS FOR PARSE /////////////////////////////////

#[derive(Debug)]
pub enum Binop {
    Add,
    Sub,
    Mul,    
    Div,
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
    MakeOp { var: String, expr: Box<AstNode> },
    BinaryOp { operator: Binop, left: Box<AstNode>, right: Box<AstNode> },
    ComparisonOp { operator: Compop, left: Box<AstNode>, right: Box<AstNode> },
    BooleanOp { operator: Boolop, left: Box<AstNode>, right: Box<AstNode> },
    DirecOp { direction: Direction, expr: Box<AstNode> },
    IdentRef(String),
    AddAssign { var_name: String, expr: Box<AstNode> },
    Ident(String),
    Num(f32),
    IfStatement { condition: Box<AstNode>, body: Box<Vec<AstNode>> },
    WhileStatement { condition: Box<AstNode>, body: Box<Vec<AstNode>> },
    PenStatusUpdate(bool),
    PenColorUpdate(Box<AstNode>),
    PenPosUpdate { update_type: PenPos, value: Box<AstNode> },
    Query(QueryKind),
    Procedure { name: String, args: Rc<Vec<String>>, body: Rc<Vec<AstNode>> },
    ProcedureReference{ name_ref: String, args: Rc<Vec<AstNode>> },
}

///////////////// PARSER FUNCS /////////////////////////////////


#[allow(unused)]
pub struct Parser {
    // Keep track of how many args an procedure has when defined
    proc_arg_map: HashMap<String, usize>,
}


impl Parser {

    // Constructor
    pub fn new() -> Self {
        Self {
            proc_arg_map: HashMap::new(),
        }
    }

    pub fn parse(&mut self, tokens: VecDeque<Token>) -> Result<Vec<AstNode>, String> {
        let mut tokens = tokens;
        let mut ast = Vec::new();
        
        while let Some(_) = tokens.front() {
            ast.push(self.expr(&mut tokens)?);
        }
        
        Ok(ast)
    }
    
    // Another possible approach: Create the arg expression, and insert the binding of the param on
    // the expression (requires keeping track of each procs argnames here, but saves evaluator)
    // E.g. To Box "arg1 "arg2 Make "X "arg1 && Box "5 + "2 "3 -> { proc_name: 'Box', {MakeOp {var: "arg1",
    // value: Num(5)} }, MakeOp { var: "arg2", value: BinOp { operator: '+', left: Num(2), right: Num(3)} } }
    // Procedure reference (any PROCNAME passed to fn expr() must be a reference to an existing procedure)
    pub fn procedure_reference(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        let proc_name = tokens.pop_front().ok_or("Expected 'PROCNAME' token")?;
        if proc_name.kind != TokenKind::PROCNAME {
            return Err(format!("Expected 'PROCNAME' token, found {:?}", proc_name.kind));
        } 

        let mut args = Vec::<AstNode>::new();

        // The type of args to procedures cannot be verified until the evaluation stage
        // as their use cases in the body are yet to be verified. Handle errors in evaluator
        if let Some(&num_args) = self.proc_arg_map.get(&proc_name.value) {
            for _ in 0..num_args {
                args.push(self.expr(tokens)?);
            }
        } else {
            panic!("The provided procedure name: {}, does not exist", proc_name.value);
        } 


        Ok(AstNode::ProcedureReference { 
            name_ref: proc_name.value,
            args: Rc::new(args),
        })

    }


    // Procedure
    pub fn procedure(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        let start_token = tokens.pop_front().ok_or("Expected 'TO' token")?;
        if start_token.kind != TokenKind::PROCSTART {
            return Err(format!("Expected 'PROCSTART' token, found {:?}", start_token.kind));
        }
        
        // Parse Proc Name 
        let proc_name_token = tokens.pop_front().ok_or("Expected 'PROCNAME' token")?;
        if proc_name_token.kind != TokenKind::PROCNAME {
            return Err(format!("Expected 'PROCSTART' token, found {:?}", proc_name_token.kind));
        }
        
        let mut arg_tokens = Vec::<String>::new();

        // The following token(s) must be: (>=1 IDENTS) XOR (Proc Body)
        // Parse Proc args until a non-IDENT token is seen
        while let Some(token) = tokens.front() {
            if token.kind != TokenKind::IDENT { 
                break; 
            }
            arg_tokens.push(tokens.pop_front().ok_or("Expected an arg/body to PROCNAME")?.value);
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

        let end_token = tokens.pop_front().ok_or("Expected ']' token")?;
        if end_token.kind != TokenKind::PROCEND {
            return Err(format!("Expected PROCEND token, instead found {:?}", end_token.kind));
        }

        // Add arg_token count to proc_arg_map
        self.proc_arg_map.insert(proc_name_token.value.clone(), arg_tokens.len());

        Ok(AstNode::Procedure { 
            name: proc_name_token.value, 
            args: Rc::new(arg_tokens), 
            body: Rc::new(body_tokens),
        })
         
    }

    fn while_statement(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        let if_token = tokens.pop_front().ok_or("Expected 'WHILE' token")?;
        if if_token.kind != TokenKind::WHILESTMNT {
            return Err(format!("Expected 'WHILE' token, found {:?}", if_token.kind));
        }
        
        
        // Parse the following condition (some expression which returns a bool)
        let condition = self.expr(tokens)?;
        // SHOULD I BE HANDLING SYNTAX ERRORS HERE OR IN EVAL
        match condition {
            AstNode::BooleanOp { .. }
                | AstNode::ComparisonOp { .. }
                | AstNode::IdentRef(_) => {
            }, 
            _ => return Err(format!("<EXPR1> in WHILE <EXPR1> [<EXPR2>], must return a boolean")),
        }
        
        // Parse body opening parenthesis
        let l_paren_token = tokens.pop_front().ok_or("Expected '[' token")?;
        if l_paren_token.kind != TokenKind::LPAREN {
            return Err(format!("Expected '[' after 'IF' in IF <EXPR1> [<EXPR2>], found {:?}", l_paren_token.value));
        }
        
        let mut body_tokens = Vec::<AstNode>::new();
        
        // Parse body until closing parenthesis is seen
        while let Some(token) = tokens.front() {
            if token.kind == TokenKind::RPAREN { 
                break; 
            }
            body_tokens.push(self.expr(tokens)?);
        }

        let r_paren_token = tokens.pop_front().ok_or("Expected ']' token")?;
        if r_paren_token.kind != TokenKind::RPAREN {
            return Err(format!("Expected ']' after '<EXPR2>' in IF <EXPR1> [<EXPR2>], found {:?}", r_paren_token.kind));
        }

        Ok(AstNode::WhileStatement { 
            condition: Box::new(condition), 
            body: Box::new(body_tokens),  
        })
    }

    fn if_statement(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        let if_token = tokens.pop_front().ok_or("Expected 'IF' token")?;
        if if_token.kind != TokenKind::IFSTMNT {
            return Err(format!("Expected 'IF' token, found {:?}", if_token.kind));
        }
        
        
        // Parse the following condition (some expression which returns a bool)
        let condition = self.expr(tokens)?;
        // SHOULD I BE HANDLING SYNTAX ERRORS HERE OR IN EVAL
        match condition {
            AstNode::BooleanOp { .. }
                | AstNode::ComparisonOp { .. }
                | AstNode::IdentRef(_) => {
            }, 
            _ => return Err(format!("<EXPR1> in IF <EXPR1> [<EXPR2>], must return a boolean")),
        }
        
        // Parse body opening parenthesis
        let l_paren_token = tokens.pop_front().ok_or("Expected '[' token")?;
        if l_paren_token.kind != TokenKind::LPAREN {
            return Err(format!("Expected '[' after 'IF' in IF <EXPR1> [<EXPR2>], found {:?}", l_paren_token.value));
        }
        
        let mut body_tokens = Vec::<AstNode>::new();
        
        // Parse body until closing parenthesis is seen
        while let Some(token) = tokens.front() {
            if token.kind == TokenKind::RPAREN { 
                break; 
            }
            body_tokens.push(self.expr(tokens)?);
        }

        let r_paren_token = tokens.pop_front().ok_or("Expected ']' token")?;
        if r_paren_token.kind != TokenKind::RPAREN {
            return Err(format!("Expected ']' after '<EXPR2>' in IF <EXPR1> [<EXPR2>], found {:?}", r_paren_token.kind));
        }

        Ok(AstNode::IfStatement { 
            condition: Box::new(condition), 
            body: Box::new(body_tokens),  
        })
    }

    fn make_op(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        // Consume 'Make' token
        let make_token = tokens.pop_front().ok_or("Expected 'Make' token")?;
        if make_token.kind != TokenKind::MAKEOP {
            return Err(format!("Expected 'Make' token, found {:?}", make_token.kind));
        }
        
        // Consume identifier token
        let ident_token = tokens.pop_front().ok_or("Expected identifier token after 'Make'")?;
        match ident_token.kind {
            TokenKind::IDENT | TokenKind::IDENTREF => {} // Continue
            _ => return Err(format!("Expected identifier/reference to identifier token after 'Make', found {:?}", ident_token.kind))

        }
       
        // Parse the expression following the identifier
        let expr = self.expr(tokens)?;
        
        Ok(AstNode::MakeOp {
            var: ident_token.value,
            expr: Box::new(expr),
        })
    }

    fn binary_op(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        // Consume the operator token
        let operator_token = tokens.pop_front().ok_or("Expected binary operator token")?;
        if operator_token.kind != TokenKind::BINOP {
            return Err(format!("Expected binary operator token, found {:?}", operator_token.kind));
        }
        
        // Parse the left And right operands
        let left = self.expr(tokens)?;
        let right = self.expr(tokens)?;
        
        Ok(AstNode::BinaryOp {
            operator: match operator_token.value.as_str() {
                "+" => Binop::Add,
                "-" => Binop::Sub,
                "*" => Binop::Mul,
                "/" => Binop::Div,
                _ => return Err(format!("Unknown binary operator: {}", operator_token.value)),
            },
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn add_assign(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        // Consume the operator token
        let operator_token = tokens.pop_front().ok_or("Expected AddAssign token")?;
        if operator_token.kind != TokenKind::ADDASSIGN {
            return Err(format!("Expected AddAssign operator token, found {:?}", operator_token.kind));
        }
        
        // Parse the next token
        let var = tokens.pop_front().ok_or("Expected variable")?;
        if var.kind != TokenKind::IDENT {
            return Err(format!("Addasign op expected a variable name, instead received: {:?}, of type {:?}", var.value, var.kind));
        }

        let val = self.expr(tokens)?;
        // Val must return a float
        match val {
            AstNode::Num(_)
                | AstNode::ComparisonOp { .. }
                | AstNode::IdentRef(_)
                | AstNode::Query(_) => {
            }, 
            _ => return Err(format!("<EXPR1> in WHILE <EXPR1> [<EXPR2>], must return a boolean")),
        }   

        
        Ok(AstNode::AddAssign { 
            var_name: var.value, 
            expr: Box::new(val),
        })

    }

    fn comparison_op(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        // Consume the operator token
        let operator_token = tokens.pop_front().ok_or("Expected comparison operator token")?;
        if operator_token.kind != TokenKind::COMPOP {
            return Err(format!("Expected binary operator token, found {:?}", operator_token.kind));
        }
        
        // Parse the left And right operAnds
        let left = self.expr(tokens)?;
        let right = self.expr(tokens)?;
        
        Ok(AstNode::ComparisonOp {
            operator: match operator_token.value.as_str() {
                "EQ" => Compop::EQ,
                "NE" => Compop::NE,
                "LT" => Compop::LT,
                "GT" => Compop::GT,
                _ => return Err(format!("Unknown binary operator: {}", operator_token.value)),
            },
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn bool_op(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        // Consume the operator token
        let operator_token = tokens.pop_front().ok_or("Expected boolean operator token")?;
        if operator_token.kind != TokenKind::BOOLOP {
            return Err(format!("Expected boolean operator token, found {:?}", operator_token.kind));
        }
        
        // Parse the left And right operAnds
        let left = self.expr(tokens)?;
        let right = self.expr(tokens)?;
        
        Ok(AstNode::BooleanOp {
            operator: match operator_token.value.as_str() {
                "AND" => Boolop::AND,
                "OR" => Boolop::OR,
                _ => return Err(format!("Unknown binary operator: {}", operator_token.value)),
            },
            left: Box::new(left),
            right: Box::new(right),
        })
    }


    fn num(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        let num_token = tokens.pop_front().ok_or("Expected number token")?;

        
        let num_value = num_token.value.parse::<f32>().map_err(|_| format!("Invalid number token: {}", num_token.value))?;
        Ok(AstNode::Num(num_value))
    }

    fn ident_ref(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        let ident_token = tokens.pop_front().ok_or("Expected ident token")?;

        
        let ident_value = ident_token.value;
        Ok(AstNode::IdentRef(ident_value))
    }


    fn pen_position_update(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        let pos_token = tokens.pop_front().ok_or("Expected setPosition token")?;
        
        // Parse the arg to the position setter
        let parsed_value = self.expr(tokens)?;

        Ok(AstNode::PenPosUpdate {
             update_type: match pos_token.value.as_str() {
                "SETX" => PenPos::SETX,
                "SETY" => PenPos::SETY,
                "TURN" => PenPos::TURN,
                "SETHEADING" => PenPos::SETHEADING,
                _ => return Err(format!("Unknown position update: {}", pos_token.value)),
             }, 
             value: Box::new(parsed_value), 
        })
    }

    fn pen_status_update(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        let status_token = tokens.pop_front().ok_or("Expected setStatus token")?;
        
        Ok(AstNode::PenStatusUpdate(
             match status_token.value.as_str() {
                "PENUP" => false,
                "PENDOWN" => true,
                _ => return Err(format!("Unknown position update: {}", status_token.value)),
             }
        ))
    }

    fn pen_color_update(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        tokens.pop_front().ok_or("Expected setColor token")?;
        
        // Parse the arg to the position setter
        let parsed_value = self.expr(tokens)?;

        Ok(AstNode::PenColorUpdate(
             Box::new(parsed_value)
        ))
    }

    fn query(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        let query_token = tokens.pop_front().ok_or("Expected PosQueryKind token")?;

        Ok(AstNode::Query(
            match query_token.value.as_str() {
                "XCOR" => QueryKind::XCOR,
                "YCOR" => QueryKind::YCOR,
                "HEADING" => QueryKind::HEADING,
                "COLOR" => QueryKind::COLOR,
                _ => return Err(format!("Unknown query: {}", query_token.value)),
            // NTS: can i just _ => unreachable!()  for all of the matches in parse? bc doesnt lexer take care of this? 
             } 
        ))
    }


    fn expr(&mut self, tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
        // Peek at current token
        if let Some(token) = tokens.front() {
            match &token.kind {
                TokenKind::MAKEOP => self.make_op(tokens),
                TokenKind::BINOP => self.binary_op(tokens),
                TokenKind::COMPOP => self.comparison_op(tokens),
                TokenKind::NUM => self.num(tokens),
                TokenKind::IDENTREF => self.ident_ref(tokens),
                TokenKind::BOOLOP => self.bool_op(tokens),
                TokenKind::PENPOS => self.pen_position_update(tokens),
                TokenKind::PENSTATUS => self.pen_status_update(tokens),
                TokenKind::PENCOLOR => self.pen_color_update(tokens),
                TokenKind::QUERY => self.query(tokens),
                TokenKind::IFSTMNT => self.if_statement(tokens),
                TokenKind::WHILESTMNT => self.while_statement(tokens),
                TokenKind::ADDASSIGN => self.add_assign(tokens),
                TokenKind::PROCSTART => self.procedure(tokens),
                TokenKind::PROCNAME => self.procedure_reference(tokens),
                _ => Err(format!("Unexpected token: {:?}", token)),
            }
        } else {
            Err("Unexpected end of tokens".to_string())
        }
    }


}

#[allow(dead_code)]
fn main() {

}



















































