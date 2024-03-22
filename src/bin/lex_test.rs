use std::io::{self, BufReader, BufRead, ErrorKind};
use std::fs::File;
use std::collections::VecDeque;

mod lex_test {}
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
    PENANGLE,
    QUERY,
    PROCSTART,
    PROCEND,
    PROCNAME,
}

#[derive(Debug)]
#[allow(unused)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
}

// Return a Token object
fn to_token(input: &str) -> Token { 
    
    match input {
        // Variable Binding
        "MAKE" => Token { kind: TokenKind::MAKEOP, value: input.to_string() },
        // Arith Binary Operations
        "+" => Token { kind: TokenKind::BINOP, value: input.to_string() },
        "-" => Token { kind: TokenKind::BINOP, value: input.to_string() },
        "*" => Token { kind: TokenKind::BINOP, value: input.to_string() },
        "/" => Token { kind: TokenKind::BINOP, value: input.to_string() },
        // Comparitive Operators
        "EQ" => Token { kind: TokenKind::COMPOP, value: input.to_string() },
        "NE" => Token { kind: TokenKind::COMPOP, value: input.to_string() },
        "GT" => Token { kind: TokenKind::COMPOP, value: input.to_string() },
        "LT" => Token { kind: TokenKind::COMPOP, value: input.to_string() },
        // Boolean Operators
        "AND" => Token { kind: TokenKind::BOOLOP, value: input.to_string() },
        "OR" => Token { kind: TokenKind::BOOLOP, value: input.to_string() },
        // Directional Movement
        "FORWARD" => Token { kind: TokenKind::DIRECTION, value: input.to_string() },
        "BACK" => Token { kind: TokenKind::DIRECTION, value: input.to_string() },
        "RIGHT" => Token { kind: TokenKind::DIRECTION, value: input.to_string() },
        "LEFT" => Token { kind: TokenKind::DIRECTION, value: input.to_string() },
        // Pen Status
        "PENUP" => Token { kind: TokenKind::PENSTATUS, value: input.to_string() },
        "PENDOWN" => Token { kind: TokenKind::PENSTATUS, value: input.to_string() },
        "SETPENCOLOR" => Token { kind: TokenKind::PENCOLOR, value: input.to_string() },
        // Pen Position / Orientation
        "SETX" => Token { kind: TokenKind::PENPOS, value: input.to_string() },
        "SETY" => Token { kind: TokenKind::PENPOS, value: input.to_string() },
        "TURN" => Token { kind: TokenKind::PENANGLE, value: input.to_string() },
        "SETHEADING" => Token { kind: TokenKind::PENANGLE, value: input.to_string() },
        // Queries
        "XCOR" => Token { kind: TokenKind::QUERY, value: input.to_string() },
        "YCOR" => Token { kind: TokenKind::QUERY, value: input.to_string() },
        "HEADING" => Token { kind: TokenKind::QUERY, value: input.to_string() },
        "COLOR" => Token { kind: TokenKind::PENANGLE, value: input.to_string() },
        // If Statements
        "IF" => Token { kind: TokenKind::IFSTMNT, value: input.to_string() },
        // While statements
        "WHILE" => Token { kind: TokenKind::WHILESTMNT, value: input.to_string() },
        // Brackets (For If / While statement blocks)
        "[" => Token { kind: TokenKind::LPAREN, value: input.to_string() },
        "]" => Token { kind: TokenKind::RPAREN, value: input.to_string() },
        // Procedures
        "TO" => Token { kind: TokenKind::PROCSTART, value: input.to_string() },
        "END" => Token { kind: TokenKind::PROCEND, value: input.to_string() },
        s if s.chars().all(|c| c.is_alphabetic()) => Token { kind: TokenKind::PROCNAME, value: s.to_string()},
        // Variables and Numbers
        s if s.starts_with('"') => {
            if s[1..].chars().all(|c| c.is_ascii_digit()) {
                Token { kind: TokenKind::NUM, value: s[1..].to_string() }
            } else if s[1..].chars().all(|c| c.is_alphanumeric()) {
                Token { kind: TokenKind::IDENT, value: s[1..].to_string() }
            } else {
                Token { kind: TokenKind::UNKNOWN, value: input.to_string() }
            }
        },
        // Variable Reference
        s if s.starts_with(':') && s[1..].chars().all(|c| c.is_alphanumeric()) => Token { kind: TokenKind::IDENTREF, value: s[1..].to_string() },
        // Addition Assignment
        "ADDASSIGN" => Token { kind: TokenKind::ADDASSIGN, value: input.to_string() },
    _ => Token { kind: TokenKind::UNKNOWN, value: input.to_string() },
    }


}

 pub fn lexer(file_path: &str) -> io::Result<VecDeque<Token>> {
  
    let file = BufReader::new(
                        File::open(file_path)?
                    );
         
    let mut tokens = VecDeque::<Token>::new();

    for buf_line in file.lines() {
        let mut line = buf_line?
                        .split_whitespace()
                        .map(|word| to_token(word))
                        .collect::<VecDeque<Token>>();
        

        tokens.append(&mut line);
    }
 
                
    Ok(tokens)  
} 

fn main() {}

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
