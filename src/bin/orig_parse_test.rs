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

#[derive(Debug)] 
enum Expr {
    Term(Term),
    BinExp(BinOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
 enum Term {
     Num(i32),
     Id(String),
}

#[derive(Debug)]
enum AstNode {
    MakeOp(String, Box<AstNode>),
    BinaryOp(String, Box<AstNode>, Box<AstNode>),
    Identifier(String),
    Number(i32),
}


//impl fmt::Display for BinOp {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//          match self {
//              BinOp::Add => write!(f, "BinOp::Add"),
//              BinOp::Sub => write!(f, "BinOp::Sub"),
//              BinOp::Mul => write!(f, "BinOp::Mul"),
//              BinOp::Div => write!(f, "BinOp::Div"),
//          }
//    }
//}


//fn parse_bin_op(input: VecDeque<Token>) -> IResult<&str, BinOp> {
//    alt((
//        map(tag("+"), |_| BinOp::Add),
//        map(tag("-"), |_| BinOp::Sub),
//        map(tag("*"), |_| BinOp::Mul),
//        map(tag("/"), |_| BinOp::Div),
//    ))(input)
//
//}

//fn parse_bin_exp(input: &str) -> IResult<&str, Expr> {
//    let (input, _) = multispace0(input)?;
//    let (input, bin_op) = parse_bin_op(input)?;
//    let (input, _) = multispace0(input)?;
//    let (input, expr1) = parse_expr(input)?;
//    let (input, _) = multispace0(input)?;
//    let (input, term) = parse_term(input)?;
//    let (input, _) = multispace0(input)?;
//    Ok((input, Expr::BinExp(bin_op, Box::new(expr1), Box::new(term))))
//}

// Parse Expression
//fn parse_expr(input: &str) -> IResult<&str, Expr> {
//   alt((
//        parse_bin_exp,
//        parse_term
//        ))(input) 
//}


//fn parse_term(input: &str) -> IResult<&str, Expr> {
//    alt((
//        map(parse_num, |term| Expr::Term(term)),
//        map(parse_id, |term| Expr::Term(term))
//        ))(input)
//}
//
//fn parse_num(input: &str) -> IResult<&str, Term> {
//    let (input, num) = map_res(digit1, |s: &str| s.parse::<i32>())(input)?; 
//    Ok((input, Term::Num(num)))
//}
//
//fn parse_id(input: &str) -> IResult<&str, Term> {
//    let (input, id) = alpha1(input)?;
//    Ok((input, Term::Id(id.to_string())))
//}


fn main() { 

    let file_path = "./src/test.lg";
   
    let mut tokens = match lexer(file_path) {
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
  
    //println!("{:?}", tokens.pop_front());
    println!("{:?}", tokens); 
//    let mut token_iter = tokens.into_iter();
//    // Loop tokens
//    while let Some(token) = token_iter.next() {
//        dbg!(token);
//        match token.kind {
//            MAKOP => {
//                let next_token = token_iter.next().unwrap();
//                let Ok((left1, operator)) = parse_bin_op(&next_token.value);  
//            }
//        }
//    }


//    println!("left_inp: {}, out: {}", leftover_input, output);
//    assert_eq!(leftover_input, "World");
//    assert_eq!(output, "abc");
//
//    assert!(parse_bin_op("defWorld").is_err());
}

fn parse_ident(input: &mut VecDeque<Token>) -> IResult<&mut VecDeque<Token>, Token> {
    // Parse identifier
    let token = input.pop_front().ok_or(())?;
    match token {
        Token::IDENT(ident) => Ok(((), Token::IDENT(ident))),
        _ => Err(()),
    }
}

//fn parse_num(input: &mut VecDeque<Token>) -> IResult<(), Token> {
//    // Parse number
//    let token = input.pop_front().ok_or(())?;
//    match token {
//        Token::NUM(num) => Ok(((), Token::NUM(num))),
//        _ => Err(()),
//    }
//}
//
//fn parse_bin_exp(input: &mut VecDeque<Token>) -> IResult<(), Token> {
//    // Parse binary expression
//    let (input, (op, left, right)) = tuple((map(parse_ident, |t| t.1), parse_num, parse_num))(input)?;
//    Ok((input, Token::BINOP(op.to_string())))
//}
//
//
//
//
fn parse_make_op(input: &mut VecDeque<Token>) -> IResult<VecDeque<Token>, AstNode> {
    // Parse MAKEOP operation
    let (input, ident) = parse_ident(input)?;
    let (input, value) = alt((
                             parse_num,
                             parse_bin_exp
                             ))(input)?;
    
    Ok((input, (ident, value)))
}


fn parse(mut tokens: VecDeque<Token>) -> Result<(), Box<dyn Error>> {

    while let Some(token) = tokens.pop_front() {
        //dbg!(token);
        match token.kind {
            MAKEOP => {
                tokens.pop_front();
                let (left1, operator) = parse_make_op(&mut tokens)?;  
            }
        }
    }

    Ok(())
}












































//// Returns each word in the file in a Vec<String>, else returns io::error
//fn read_file(file_path: std::path::PathBuf) -> io::Result<Vec<String>> {
//    let file = File::open(file_path)?;
//    let reader = BufReader::new(file);
//    let mut file_contents: Vec<String> = Vec::new();
//
//    for read_line in reader.lines() {
//        let line = read_line?;
//        file_contents.extend(line.split_whitespace().map(String::from));    
//    }
//    
//    file_contents) 
//
//}
//
//
// fn lexer(file_path: std::path::PathBuf) {
//    
//    let file_contents = match read_file(file_path) {
//        contents) => contents,
//        Err(error) => panic!("Error: {:?}", error),
//    };
//    
//    dbg!(file_contents);
//    todo!();
//}






