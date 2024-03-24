#![allow(unused_imports)]
#![allow(dead_code)]

use std::io::{self, BufRead, BufReader, ErrorKind};
use std::fs::File;
use std::fmt;
use std::error::Error;
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
    //VarBind { var_name: String, expr: Box<AstNode> },
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
    Procedure { name: String, args: Vec<String>, body: Vec<AstNode> },
}

///////////////// PARSER FUNCS /////////////////////////////////

pub fn parse(tokens: VecDeque<Token>) -> Result<Vec<AstNode>, String> {
    let mut tokens = tokens;
    let mut ast = Vec::new();
    
    while let Some(_) = tokens.front() {
        ast.push(expr(&mut tokens)?);
    }
    
    Ok(ast)
}

fn while_statement(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    let if_token = tokens.pop_front().ok_or("Expected 'WHILE' token")?;
    if if_token.kind != TokenKind::WHILESTMNT {
        return Err(format!("Expected 'WHILE' token, found {:?}", if_token.kind));
    }
    
    
    // Parse the following condition (some expression which returns a bool)
    let condition = expr(tokens)?;
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
        body_tokens.push(expr(tokens)?);
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

fn if_statement(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    let if_token = tokens.pop_front().ok_or("Expected 'IF' token")?;
    if if_token.kind != TokenKind::IFSTMNT {
        return Err(format!("Expected 'IF' token, found {:?}", if_token.kind));
    }
    
    
    // Parse the following condition (some expression which returns a bool)
    let condition = expr(tokens)?;
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
        body_tokens.push(expr(tokens)?);
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

fn make_op(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
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
    let expr = expr(tokens)?;
    
    Ok(AstNode::MakeOp {
        var: ident_token.value,
        expr: Box::new(expr),
    })
}

fn binary_op(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    // Consume the operator token
    let operator_token = tokens.pop_front().ok_or("Expected binary operator token")?;
    if operator_token.kind != TokenKind::BINOP {
        return Err(format!("Expected binary operator token, found {:?}", operator_token.kind));
    }
    
    // Parse the left And right operands
    let left = expr(tokens)?;
    let right = expr(tokens)?;
    
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

fn add_assign(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
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

    let val = expr(tokens)?;
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

fn comparison_op(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    // Consume the operator token
    let operator_token = tokens.pop_front().ok_or("Expected comparison operator token")?;
    if operator_token.kind != TokenKind::COMPOP {
        return Err(format!("Expected binary operator token, found {:?}", operator_token.kind));
    }
    
    // Parse the left And right operAnds
    let left = expr(tokens)?;
    let right = expr(tokens)?;
    
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

fn bool_op(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    // Consume the operator token
    let operator_token = tokens.pop_front().ok_or("Expected boolean operator token")?;
    if operator_token.kind != TokenKind::BOOLOP {
        return Err(format!("Expected boolean operator token, found {:?}", operator_token.kind));
    }
    
    // Parse the left And right operAnds
    let left = expr(tokens)?;
    let right = expr(tokens)?;
    
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


fn num(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    let num_token = tokens.pop_front().ok_or("Expected number token")?;

    
    let num_value = num_token.value.parse::<f32>().map_err(|_| format!("Invalid number token: {}", num_token.value))?;
    Ok(AstNode::Num(num_value))
}

fn ident_ref(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    let ident_token = tokens.pop_front().ok_or("Expected ident token")?;

    
    let ident_value = ident_token.value;
    Ok(AstNode::IdentRef(ident_value))
}


fn pen_position_update(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    let pos_token = tokens.pop_front().ok_or("Expected setPosition token")?;
    
    // Parse the arg to the position setter
    let parsed_value = expr(tokens)?;

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

fn pen_status_update(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    let status_token = tokens.pop_front().ok_or("Expected setStatus token")?;
    
    Ok(AstNode::PenStatusUpdate(
         match status_token.value.as_str() {
            "PENUP" => false,
            "PENDOWN" => true,
            _ => return Err(format!("Unknown position update: {}", status_token.value)),
         }
    ))
}

fn pen_color_update(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    tokens.pop_front().ok_or("Expected setColor token")?;
    
    // Parse the arg to the position setter
    let parsed_value = expr(tokens)?;

    Ok(AstNode::PenColorUpdate(
         Box::new(parsed_value)
    ))
}

fn query(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
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


fn expr(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    // Peek at current token
    if let Some(token) = tokens.front() {
        match &token.kind {
            TokenKind::MAKEOP => make_op(tokens),
            TokenKind::BINOP => binary_op(tokens),
            TokenKind::COMPOP => comparison_op(tokens),
            TokenKind::NUM => num(tokens),
            TokenKind::IDENTREF => ident_ref(tokens),
            TokenKind::BOOLOP => bool_op(tokens),
            TokenKind::PENPOS => pen_position_update(tokens),
            TokenKind::PENSTATUS => pen_status_update(tokens),
            TokenKind::PENCOLOR => pen_color_update(tokens),
            TokenKind::QUERY => query(tokens),
            TokenKind::IFSTMNT => if_statement(tokens),
            TokenKind::WHILESTMNT => while_statement(tokens),
            TokenKind::ADDASSIGN => add_assign(tokens),
            _ => Err(format!("Unexpected token: {:?}", token)),
        }
    } else {
        Err("Unexpected end of tokens".to_string())
    }
}

#[allow(dead_code)]
fn main() {

}



















































