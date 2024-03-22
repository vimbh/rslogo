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
mod lex_test;
use lex_test::{lexer, Token, TokenKind};
use std::collections::VecDeque;

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
}

#[derive(Debug)]
pub enum PenAngle {
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
    CompOp { operator: Compop, left: Box<AstNode>, right: Box<AstNode> },
    BoolOp { operator: Boolop, left: Box<AstNode>, right: Box<AstNode> },
    DirecOp { direction: Direction, expr: Box<AstNode> },
    VarBind { var_name: String, expr: Box<AstNode> },
    VarRef { var_name: String },
    AddAssign { var_name: String, expr: Box<AstNode> },
    Ident(String),
    Num(i32),
    IfStatement { operation: Box<AstNode>, body: Box<AstNode> },
    WhileStatement { operation: Box<AstNode>, body: Box<AstNode> },
    PenStatusUpdate { pen_down: bool },
    PenColorUpdate { pen_color: i32 },
    PenPosUpdate { coordinate: PenPos, value: i32 },
    PenAngleUpdate { update_kind: PenAngle, value: i32 },
    Query { query_kind: QueryKind, value: i32 },
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

fn make_op(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    // Consume 'Make' token
    let make_token = tokens.pop_front().ok_or("Expected 'Make' token")?;
    if make_token.kind != TokenKind::MAKEOP {
        return Err(format!("Expected 'Make' token, found {:?}", make_token.kind));
    }
    
    // Consume identifier token
    let ident_token = tokens.pop_front().ok_or("Expected identifier token after 'Make'")?;
    if ident_token.kind != TokenKind::IDENT {
        return Err(format!("Expected identifier token after 'Make', found {:?}", ident_token.kind));
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
    
    // Parse the left and right operands
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

fn num(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    let num_token = tokens.pop_front().ok_or("Expected number token")?;

    
    let num_value = num_token.value.parse::<i32>().map_err(|_| format!("Invalid number token: {}", num_token.value))?;
    Ok(AstNode::Num(num_value))
}


fn expr(tokens: &mut VecDeque<Token>) -> Result<AstNode, String> {
    // Peek at current token
    if let Some(token) = tokens.front() {
        match &token.kind {
            TokenKind::MAKEOP => make_op(tokens),
            TokenKind::BINOP => binary_op(tokens),
            TokenKind::NUM => num(tokens),
            _ => Err(format!("Unexpected token: {:?}", token.value)),
        }
    } else {
        Err("Unexpected end of tokens".to_string())
    }
}

fn main() { 

    let file_path = "./src/test.lg";
   
    // Generate Tokens, manage errors
    let tokens = match lexer(file_path) {
        Ok(tokens) => tokens,
        Err(e) => {
            match e.kind() {    
                ErrorKind::NotFound => panic!("Error: File not found"),
                ErrorKind::PermissionDenied => panic!("Error: Permission to file denied"),
                ErrorKind::InvalidData => panic!("Nnvalid (non utf-8) character encountered file"),
                // Generic handling of other IO errors
                _ => panic!("Error: {}", e),
            }
        }
    };
 
    // Parse & generate AST
    let ast = match parse(tokens) {
        Ok(ast) => ast,
        Err(e) => panic!("Error: {}", e),
    };

    println!("{:?}", &ast);

    // Loop nodes and evaluate
    evaluate(ast);
        
}

////////////// EVALUATORS /////////////


fn evaluate(ast: Vec<AstNode>) {
        
    for node in ast {

        match node {
            AstNode::MakeOp { var, expr } => todo!(),
            _ => panic!("no match"),
        }
    }

}














































