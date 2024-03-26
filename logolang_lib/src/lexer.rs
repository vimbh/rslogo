use std::io::{self, BufReader, BufRead};
use std::fs::File;
use std::collections::VecDeque;
//use thiserror::Error;
use anyhow::Result;
use crate::logolang_errors::LexerError; 

//#[derive(Debug, Error)]
//pub enum LexerError {
//    #[error("Failed to lex input file: '{0}' is not a valid token")]
//    InvalidTokenError(String),
//
//    #[error("Error while trying to read file")]
//    IoError(#[from] io::Error),
//}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    MAKEOP,
    BINOP,
    COMPOP,
    BOOLOP,
    DIRECTION,
    IDENT,
    IDENTREF,
    ADDASSIGN,
    NUM,
    UNKNOWN,
    IFSTMNT,
    WHILESTMNT,
    LPAREN,
    RPAREN,
    PENSTATUS,
    PENCOLOR,
    PENPOS,
    QUERY,
    PROCSTART,
    PROCEND,
    PROCNAME,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
}

// Return a Token object
fn to_token(input: &str) -> Result<Token, LexerError> { 
    
    match input {
        // Variable Binding
        "MAKE" => Ok(Token { kind: TokenKind::MAKEOP, value: input.to_string() }),
        // Arith Binary Operations
        "+" => Ok(Token { kind: TokenKind::BINOP, value: input.to_string() }),
        "-" => Ok(Token { kind: TokenKind::BINOP, value: input.to_string() }),
        "*" => Ok(Token { kind: TokenKind::BINOP, value: input.to_string() }),
        "/" => Ok(Token { kind: TokenKind::BINOP, value: input.to_string() }),
        // Comparitive Operators
        "EQ" => Ok(Token { kind: TokenKind::COMPOP, value: input.to_string() }),
        "NE" => Ok(Token { kind: TokenKind::COMPOP, value: input.to_string() }),
        "GT" => Ok(Token { kind: TokenKind::COMPOP, value: input.to_string() }),
        "LT" => Ok(Token { kind: TokenKind::COMPOP, value: input.to_string() }),
        // Boolean Operators
        "AND" => Ok(Token { kind: TokenKind::BOOLOP, value: input.to_string() }),
        "OR" => Ok(Token { kind: TokenKind::BOOLOP, value: input.to_string() }),
        // Addition Assignment
        "ADDASSIGN" => Ok(Token { kind: TokenKind::ADDASSIGN, value: input.to_string() }),
        // Directional Movement
        "FORWARD" => Ok(Token { kind: TokenKind::DIRECTION, value: input.to_string() }),
        "BACK" => Ok(Token { kind: TokenKind::DIRECTION, value: input.to_string() }),
        "RIGHT" => Ok(Token { kind: TokenKind::DIRECTION, value: input.to_string() }),
        "LEFT" => Ok(Token { kind: TokenKind::DIRECTION, value: input.to_string() }),
        // Pen Status
        "PENUP" => Ok(Token { kind: TokenKind::PENSTATUS, value: input.to_string() }),
        "PENDOWN" => Ok(Token { kind: TokenKind::PENSTATUS, value: input.to_string() }),
        "SETPENCOLOR" => Ok(Token { kind: TokenKind::PENCOLOR, value: input.to_string() }),
        // Pen Position / Orientation
        "SETX" => Ok(Token { kind: TokenKind::PENPOS, value: input.to_string() }),
        "SETY" => Ok(Token { kind: TokenKind::PENPOS, value: input.to_string() }),
        "TURN" => Ok(Token { kind: TokenKind::PENPOS, value: input.to_string() }),
        "SETHEADING" => Ok(Token { kind: TokenKind::PENPOS, value: input.to_string() }),
        // Queries
        "XCOR" => Ok(Token { kind: TokenKind::QUERY, value: input.to_string() }),
        "YCOR" => Ok(Token { kind: TokenKind::QUERY, value: input.to_string() }),
        "HEADING" => Ok(Token { kind: TokenKind::QUERY, value: input.to_string() }),
        "COLOR" => Ok(Token { kind: TokenKind::QUERY, value: input.to_string() }),
        // If Statements
        "IF" => Ok(Token { kind: TokenKind::IFSTMNT, value: input.to_string() }),
        // While statements
        "WHILE" => Ok(Token { kind: TokenKind::WHILESTMNT, value: input.to_string() }),
        // Brackets (For If / While statement blocks)
        "[" => Ok(Token { kind: TokenKind::LPAREN, value: input.to_string() }),
        "]" => Ok(Token { kind: TokenKind::RPAREN, value: input.to_string() }),
        // Variables and Numbers
        s if s.starts_with('"') => {
            if let Ok(_) = s[1..].parse::<f32>() {
                Ok(Token { kind: TokenKind::NUM, value: s[1..].to_string() })
            } else if s[1..].chars().all(|c| c.is_alphanumeric()) {
                Ok(Token { kind: TokenKind::IDENT, value: s[1..].to_string() })
            } else {
                Ok(Token { kind: TokenKind::UNKNOWN, value: input.to_string() })
            }
        },
        // Variable Reference
        s if s.starts_with(':') && s[1..].chars().all(|c| c.is_alphanumeric()) => Ok(Token { kind: TokenKind::IDENTREF, value: s[1..].to_string() }),
        // Procedures
        "TO" => Ok(Token { kind: TokenKind::PROCSTART, value: input.to_string() }),
        "END" => Ok(Token { kind: TokenKind::PROCEND, value: input.to_string() }),
        s if s.chars().all(|c| c.is_alphabetic()) => Ok(Token { kind: TokenKind::PROCNAME, value: s.to_string()}),

        _ => Err(LexerError::InvalidTokenError(input.to_string())),
    }


}

 // Lex input stream into tokens
 pub fn tokenize(file_path: std::path::PathBuf) -> Result<VecDeque<Token>, LexerError> {
  
    let file = BufReader::new(
                        File::open(file_path)?
                    );
         
    let mut tokens = VecDeque::<Token>::new();

    for buf_line in file.lines() {
        let line = buf_line?;

        // Ignore comments
        if line.trim_start().starts_with("//") {
            continue;
        }

        // Tokenize stream
        let mut tokenized_lines = 
            line
                .split_whitespace()
                .map(|word| to_token(word))
                .collect::<Result<VecDeque<_>, _>>()?;

        tokens.append(&mut tokenized_lines);
    }
 
    Ok(tokens)  
} 
