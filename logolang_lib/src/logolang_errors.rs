use std::io;
use thiserror::Error;


// Error for incorrect file path extensions
#[derive(Debug, Error)]
pub enum ImgFileError {
    #[error("Provided image file extension is not supported, could not save image. Please use .svg or .png")]
    UnsupportedFileExtension,
}

// Lexer errors
#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Failed to lex input file: '{0}' is not a valid token")]
    InvalidTokenError(String),

    #[error("Error while trying to read file")]
    IoError(#[from] io::Error),
}


