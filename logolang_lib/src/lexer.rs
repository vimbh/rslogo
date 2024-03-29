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
    ARITHOP,
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
    pub line: i32,
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
fn to_token(input: &str, line_no: i32) -> Result<Token, LexerError> {
    match input {
        // Variable Binding
        "MAKE" => Ok(Token {
            kind: TokenKind::MAKEOP,
            value: String::from(input),
            line: line_no,
        }),
        // Arith Binary Operations
        "+" => Ok(Token {
            kind: TokenKind::ARITHOP,
            value: String::from(input),
            line: line_no,
        }),
        "-" => Ok(Token {
            kind: TokenKind::ARITHOP,
            value: String::from(input),
            line: line_no,
        }),
        "*" => Ok(Token {
            kind: TokenKind::ARITHOP,
            value: String::from(input),
            line: line_no,
        }),
        "/" => Ok(Token {
            kind: TokenKind::ARITHOP,
            value: String::from(input),
            line: line_no,
        }),
        // Comparitive Operators
        "EQ" => Ok(Token {
            kind: TokenKind::COMPOP,
            value: String::from(input),
            line: line_no,
        }),
        "NE" => Ok(Token {
            kind: TokenKind::COMPOP,
            value: String::from(input),
            line: line_no,
        }),
        "GT" => Ok(Token {
            kind: TokenKind::COMPOP,
            value: String::from(input),
            line: line_no,
        }),
        "LT" => Ok(Token {
            kind: TokenKind::COMPOP,
            value: String::from(input),
            line: line_no,
        }),
        // Boolean Operators
        "AND" => Ok(Token {
            kind: TokenKind::BOOLOP,
            value: String::from(input),
            line: line_no,
        }),
        "OR" => Ok(Token {
            kind: TokenKind::BOOLOP,
            value: String::from(input),
            line: line_no,
        }),
        // Addition Assignment
        "ADDASSIGN" => Ok(Token {
            kind: TokenKind::ADDASSIGN,
            value: String::from(input),
            line: line_no,
        }),
        // Directional Movement
        "FORWARD" => Ok(Token {
            kind: TokenKind::DIRECTION,
            value: String::from(input),
            line: line_no,
        }),
        "BACK" => Ok(Token {
            kind: TokenKind::DIRECTION,
            value: String::from(input),
            line: line_no,
        }),
        "RIGHT" => Ok(Token {
            kind: TokenKind::DIRECTION,
            value: String::from(input),
            line: line_no,
        }),
        "LEFT" => Ok(Token {
            kind: TokenKind::DIRECTION,
            value: String::from(input),
            line: line_no,
        }),
        // Pen Status
        "PENUP" => Ok(Token {
            kind: TokenKind::PENSTATUS,
            value: String::from(input),
            line: line_no,
        }),
        "PENDOWN" => Ok(Token {
            kind: TokenKind::PENSTATUS,
            value: String::from(input),
            line: line_no,
        }),
        "SETPENCOLOR" => Ok(Token {
            kind: TokenKind::PENCOLOR,
            value: String::from(input),
            line: line_no,
        }),
        // Pen Position / Orientation
        "SETX" => Ok(Token {
            kind: TokenKind::PENPOS,
            value: String::from(input),
            line: line_no,
        }),
        "SETY" => Ok(Token {
            kind: TokenKind::PENPOS,
            value: String::from(input),
            line: line_no,
        }),
        "TURN" => Ok(Token {
            kind: TokenKind::PENPOS,
            value: String::from(input),
            line: line_no,
        }),
        "SETHEADING" => Ok(Token {
            kind: TokenKind::PENPOS,
            value: String::from(input),
            line: line_no,
        }),
        // Queries
        "XCOR" => Ok(Token {
            kind: TokenKind::QUERY,
            value: String::from(input),
            line: line_no,
        }),
        "YCOR" => Ok(Token {
            kind: TokenKind::QUERY,
            value: String::from(input),
            line: line_no,
        }),
        "HEADING" => Ok(Token {
            kind: TokenKind::QUERY,
            value: String::from(input),
            line: line_no,
        }),
        "COLOR" => Ok(Token {
            kind: TokenKind::QUERY,
            value: String::from(input),
            line: line_no,
        }),
        // If Statements
        "IF" => Ok(Token {
            kind: TokenKind::IFSTMNT,
            value: String::from(input),
            line: line_no,
        }),
        // While statements
        "WHILE" => Ok(Token {
            kind: TokenKind::WHILESTMNT,
            value: String::from(input),
            line: line_no,
        }),
        // Brackets (For If / While statement blocks)
        "[" => Ok(Token {
            kind: TokenKind::LPAREN,
            value: String::from(input),
            line: line_no,
        }),
        "]" => Ok(Token {
            kind: TokenKind::RPAREN,
            value: String::from(input),
            line: line_no,
        }),
        // Variables and Numbers
        s if s.starts_with('"') => {
            if s[1..].parse::<f32>().is_ok() {
                Ok(Token {
                    kind: TokenKind::NUM,
                    value: s[1..].to_string(),
                    line: line_no,
                })
            } else if s[1..].chars().all(|c| c.is_alphanumeric() || c == '_') {
                Ok(Token {
                    kind: TokenKind::IDENT,
                    value: s[1..].to_string(),
                    line: line_no,
                })
            } else {
                Err(LexerError::InvalidTokenError(String::from(input)))
            }
        }
        // Variable Reference
        s if s.starts_with(':') && s[1..].chars().all(|c| c.is_alphanumeric() || c == '_') => {
            Ok(Token {
                kind: TokenKind::IDENTREF,
                value: s[1..].to_string(),
                line: line_no,
            })
        }
        // Procedures
        "TO" => Ok(Token {
            kind: TokenKind::PROCSTART,
            value: String::from(input),
            line: line_no,
        }),
        "END" => Ok(Token {
            kind: TokenKind::PROCEND,
            value: String::from(input),
            line: line_no,
        }),
        s if s.chars().all(|c| c.is_alphabetic()) => Ok(Token {
            kind: TokenKind::PROCNAME,
            value: s.to_string(),
            line: line_no,
        }),

        _ => Err(LexerError::InvalidTokenError(String::from(input))),
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
    for (line_no, buf_line) in (0_i32..).zip(file.lines()) {
        let line = buf_line?;

        // Ignore comments
        if line.trim_start().starts_with("//") {
            continue;
        }

        // Tokenize stream
        let mut tokenized_lines = line
            .split_whitespace()
            .map(|word| to_token(word, line_no))
            .collect::<Result<VecDeque<_>, _>>()?;

        tokens.append(&mut tokenized_lines);
    }

    Ok(tokens)
}
