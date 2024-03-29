use std::io;
use thiserror::Error;

// MAIN errors: File extension errors
#[derive(Debug, Error)]
pub enum ImgFileError {
    #[error("Provided image file extension is not supported, could not save image. Please use .svg or .png")]
    UnsupportedFileExtension,
}

// LEXER errors: File read errors, unsupported tokens
#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Failed to lex input file: '{0}' is not a valid token")]
    InvalidTokenError(String),

    #[error("Error while trying to read file")]
    IoError(#[from] io::Error),
}

// PARSER errors: syntactic errors
#[derive(Debug, Error)]
pub enum ParserError {
    // All <anyhow::Error> are cast to ParseError to catch the context chain
    #[error("{0}")]
    ParseError(String),

    #[error("Unexpected ending  while parsing program.\n")]
    UnexpectedEnding,

    #[error("\t[Line {0}]: Arguments to '{1}' will not return a float. You must provide arguments which return a number\n")]
    NonNumericExpr(String, String),

    #[error("\t[Line {0}]: Arguments to '{1}' will not return a boolean. You must provide arguments which return TRUE or FALSE\n")]
    NonBooleanExpr(String, String),

    #[error("[Line {0}]: {1}\n")]
    IncorrectArgType(String, String),

    #[error("[Line {0}]: {1}\n")]
    InvalidToken(String, String),

    #[error("[Line {0}]: {1} statement is missing parenthesis: expected {2}, received {3}.\n")]
    MissingParenthesis(String, String, String, String),

    #[error("[Line{0}]: Invalid ADDASSIGN operation. Expected identifier, received {1}.\n")]
    InvalidAddAssign(String, String),

    #[error("[Line {0}]: Invalid procedure name: Keywords and variables must not be used as procedure names, received: {1}.\n")]
    InvalidProcName(String, String),

    #[error("[Line {0}]: Invalid procedure: Expected END, received: {1}.\n")]
    MissingProcEnd(String, String),

    #[error("[Line{0}]: Invalid procedure reference: {1} does not exist.\n")]
    InvalidProcReference(String, String),
}

// Error propogation
impl From<anyhow::Error> for ParserError {
    fn from(error: anyhow::Error) -> Self {
        ParserError::ParseError(format!("{:?}", error))
    }
}

// INTERPRETER errors: semantic errors
#[derive(Debug, Error)]
pub enum InterpreterError {
    // All <anyhow::Error> are cast to InterpError to catch the context chain
    #[error("{0}")]
    InterpError(String),

    #[error("[{0}")]
    TypeError(String),

    #[error("{0}")]
    InvalidVariableRef(String),

    #[error("{0} {1}")]
    DrawLineError(String, String),

    #[error("{0} is not a valid color. Enter an integer between 0 and 15.")]
    InvalidPenColor(String),

    #[error("{0}")]
    InvalidProcedureRef(String),
}

// Error propogation
impl From<anyhow::Error> for InterpreterError {
    fn from(error: anyhow::Error) -> Self {
        InterpreterError::InterpError(format!("{:?}", error))
    }
}
