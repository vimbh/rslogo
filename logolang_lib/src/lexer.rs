//! This module defines the representations and functions utilised for lexical analysis over the RSLOGO language; specifically, tokenizing
//! input strings.
//!
//! It provides the `TokenKind` enum which defines the valid kinds of tokens, and
//! the `Token` struct representing the binding of a tokens kind and value. As hinted by its name,
//! the 'tokenize' function is used for tokenizing input from a file.

use crate::logolang_errors::LexerError;
use anyhow::Result;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Represents the set of valid tokens in RSLOGO.
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

/// Representation of a single tokens kind and value.
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
}

/// Converts an input string to a token.
/// # Arguments
///
/// * `input` - The input string to convert to a token.
///
/// # Returns
///
/// A `Result` containing the converted token, or a `LexerError` if the input
/// is not a valid token.
fn to_token(input: &str) -> Result<Token, LexerError> {
    match input {
        // Variable Binding
        "MAKE" => Ok(Token {
            kind: TokenKind::MAKEOP,
            value: input.to_string(),
        }),
        // Arith Binary Operations
        "+" => Ok(Token {
            kind: TokenKind::BINOP,
            value: input.to_string(),
        }),
        "-" => Ok(Token {
            kind: TokenKind::BINOP,
            value: input.to_string(),
        }),
        "*" => Ok(Token {
            kind: TokenKind::BINOP,
            value: input.to_string(),
        }),
        "/" => Ok(Token {
            kind: TokenKind::BINOP,
            value: input.to_string(),
        }),
        // Comparitive Operators
        "EQ" => Ok(Token {
            kind: TokenKind::COMPOP,
            value: input.to_string(),
        }),
        "NE" => Ok(Token {
            kind: TokenKind::COMPOP,
            value: input.to_string(),
        }),
        "GT" => Ok(Token {
            kind: TokenKind::COMPOP,
            value: input.to_string(),
        }),
        "LT" => Ok(Token {
            kind: TokenKind::COMPOP,
            value: input.to_string(),
        }),
        // Boolean Operators
        "AND" => Ok(Token {
            kind: TokenKind::BOOLOP,
            value: input.to_string(),
        }),
        "OR" => Ok(Token {
            kind: TokenKind::BOOLOP,
            value: input.to_string(),
        }),
        // Addition Assignment
        "ADDASSIGN" => Ok(Token {
            kind: TokenKind::ADDASSIGN,
            value: input.to_string(),
        }),
        // Directional Movement
        "FORWARD" => Ok(Token {
            kind: TokenKind::DIRECTION,
            value: input.to_string(),
        }),
        "BACK" => Ok(Token {
            kind: TokenKind::DIRECTION,
            value: input.to_string(),
        }),
        "RIGHT" => Ok(Token {
            kind: TokenKind::DIRECTION,
            value: input.to_string(),
        }),
        "LEFT" => Ok(Token {
            kind: TokenKind::DIRECTION,
            value: input.to_string(),
        }),
        // Pen Status
        "PENUP" => Ok(Token {
            kind: TokenKind::PENSTATUS,
            value: input.to_string(),
        }),
        "PENDOWN" => Ok(Token {
            kind: TokenKind::PENSTATUS,
            value: input.to_string(),
        }),
        "SETPENCOLOR" => Ok(Token {
            kind: TokenKind::PENCOLOR,
            value: input.to_string(),
        }),
        // Pen Position / Orientation
        "SETX" => Ok(Token {
            kind: TokenKind::PENPOS,
            value: input.to_string(),
        }),
        "SETY" => Ok(Token {
            kind: TokenKind::PENPOS,
            value: input.to_string(),
        }),
        "TURN" => Ok(Token {
            kind: TokenKind::PENPOS,
            value: input.to_string(),
        }),
        "SETHEADING" => Ok(Token {
            kind: TokenKind::PENPOS,
            value: input.to_string(),
        }),
        // Queries
        "XCOR" => Ok(Token {
            kind: TokenKind::QUERY,
            value: input.to_string(),
        }),
        "YCOR" => Ok(Token {
            kind: TokenKind::QUERY,
            value: input.to_string(),
        }),
        "HEADING" => Ok(Token {
            kind: TokenKind::QUERY,
            value: input.to_string(),
        }),
        "COLOR" => Ok(Token {
            kind: TokenKind::QUERY,
            value: input.to_string(),
        }),
        // If Statements
        "IF" => Ok(Token {
            kind: TokenKind::IFSTMNT,
            value: input.to_string(),
        }),
        // While statements
        "WHILE" => Ok(Token {
            kind: TokenKind::WHILESTMNT,
            value: input.to_string(),
        }),
        // Brackets (For If / While statement blocks)
        "[" => Ok(Token {
            kind: TokenKind::LPAREN,
            value: input.to_string(),
        }),
        "]" => Ok(Token {
            kind: TokenKind::RPAREN,
            value: input.to_string(),
        }),
        // Variables and Numbers
        s if s.starts_with('"') => {
            if s[1..].parse::<f32>().is_ok() {
                Ok(Token {
                    kind: TokenKind::NUM,
                    value: s[1..].to_string(),
                })
            } else if s[1..].chars().all(|c| c.is_alphanumeric()) {
                Ok(Token {
                    kind: TokenKind::IDENT,
                    value: s[1..].to_string(),
                })
            } else {
                Err(LexerError::InvalidTokenError(input.to_string()))
            }
        }
        // Variable Reference
        s if s.starts_with(':') && s[1..].chars().all(|c| c.is_alphanumeric()) => Ok(Token {
            kind: TokenKind::IDENTREF,
            value: s[1..].to_string(),
        }),
        // Procedures
        "TO" => Ok(Token {
            kind: TokenKind::PROCSTART,
            value: input.to_string(),
        }),
        "END" => Ok(Token {
            kind: TokenKind::PROCEND,
            value: input.to_string(),
        }),
        s if s.chars().all(|c| c.is_alphabetic()) => Ok(Token {
            kind: TokenKind::PROCNAME,
            value: s.to_string(),
        }),

        _ => Err(LexerError::InvalidTokenError(input.to_string())),
    }
}

/// Tokenizes the input from the provided file.
///
/// # Arguments
///
/// * `file_path` - The path to the input file.
///
/// # Returns
///
/// A [`anyhow::Result`] containing a [`VecDeque`] of tokens if successful, or a `LexerError`
/// if an error occurs during tokenization.
pub fn tokenize(file_path: std::path::PathBuf) -> Result<VecDeque<Token>, LexerError> {
    let file = BufReader::new(File::open(file_path)?);

    let mut tokens = VecDeque::<Token>::new();

    for buf_line in file.lines() {
        let line = buf_line?;

        // Ignore comments
        if line.trim_start().starts_with("//") {
            continue;
        }

        // Tokenize stream
        let mut tokenized_lines = line
            .split_whitespace()
            .map(to_token)
            .collect::<Result<VecDeque<_>, _>>()?;

        tokens.append(&mut tokenized_lines);
    }

    Ok(tokens)
}
