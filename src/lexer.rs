use std::io::{self, BufReader, BufRead, ErrorKind};
use std::fs::File;

#[derive(Debug)]
pub enum TokenKind {
    MAKE_OP,
    BIN_OP,
    DIRECTION,
    IDENT,
    NUM,
    UNKNOWN,
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    value: String,
}

// Return a Token object
fn to_token(input: &str) -> Token {
    
    match input {
        "MAKE" => Token { kind: TokenKind::MAKE_OP, value: input.to_string() },
        "+" => Token { kind: TokenKind::BIN_OP, value: input.to_string() },
        "-" => Token { kind: TokenKind::BIN_OP, value: input.to_string() },
        "*" => Token { kind: TokenKind::BIN_OP, value: input.to_string() },
        "/" => Token { kind: TokenKind::BIN_OP, value: input.to_string() },
        "FORWARD" => Token { kind: TokenKind::DIRECTION, value: input.to_string() },
        "BACK" => Token { kind: TokenKind::DIRECTION, value: input.to_string() },
        "RIGHT" => Token { kind: TokenKind::DIRECTION, value: input.to_string() },
        s if s.starts_with('"') => {
            if s[1..].chars().all(|c| c.is_ascii_alphabetic()) {
                Token { kind: TokenKind::IDENT, value: s.to_string() }
            } else if s[1..].chars().all(|c| c.is_ascii_digit()) {
                Token { kind: TokenKind::NUM, value: s[1..].to_string() }
            } else {
                Token { kind: TokenKind::UNKNOWN, value: input.to_string() }
            }
        },
    _ => Token { kind: TokenKind::UNKNOWN, value: input.to_string() },
    }


}

pub fn lexer(file_path: std::path::PathBuf) -> io::Result<Vec<Token>> {
  
    let file = BufReader::new(
                        File::open(file_path)?
                    );
         
    let mut tokens = Vec::<Token>::new();

    for buf_line in file.lines() {
        let mut line = buf_line?
                        .split_whitespace()
                        .map(|word| to_token(word))
                        .collect::<Vec<Token>>();
        

        tokens.append(&mut line);
    }
 
                
    Ok(tokens)  
} 


//fn main() {
//
//    let file_path = "./src/test.lg";
//   
//    let tokens = match lexer(file_path) {
//        Ok(tokens) => tokens,
//        Err(e) => {
//            match e.kind() {
//                ErrorKind::NotFound => panic!("Error: File not found"),
//                ErrorKind::PermissionDenied => panic!("Error: Permission to file denied"),
//                ErrorKind::InvalidData => panic!("Nnvalid (non utf-8) character encountered file"),
//                // Generic handling of other IO errors
//                _ => panic!("Error: {}", e),
//            }
//        }
//    };
//   
//    println!("{:?}", tokens); 
//   
//    
//}
