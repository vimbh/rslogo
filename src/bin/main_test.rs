use std::io::ErrorKind;
mod lex_test;
mod parse_test;
mod evaluator_test;
use lex_test::lexer;
use parse_test::parse;
use evaluator_test::Evaluator;



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
    // println!("{:?}",&tokens);
    
    // Parse & generate AST
    let ast = match parse(tokens) {
        Ok(ast) => ast,
        Err(e) => panic!("Error: {}", e),
    };

    println!("{:?}", &ast);

    // Loop nodes and evaluate
    let mut evaluator = Evaluator::new();
    evaluator.evaluate(ast); 
}
