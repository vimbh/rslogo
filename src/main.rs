use clap::Parser as clapParser;
//use unsvg::Image;
use logolang_lib::{lexer, parser, interpreter};
use lexer::tokenize;
use parser::Parser;
use interpreter::Interpreter;
use std::io::ErrorKind;

/// A simple program to parse four arguments using clap.
#[derive(clapParser)]
struct Args {
    /// Path to a file
    file_path: std::path::PathBuf,

    /// Path to an svg or png image
    image_path: std::path::PathBuf,

    /// Height
    height: u32,

    /// Width
    width: u32,
}


fn main() -> Result<(), ()> {
    let args: Args = Args::parse();
    // Access the parsed arguments
    let file_path = args.file_path;
    
    // Generate Tokens, manage errors
    let tokens = match tokenize(file_path) {
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
    println!("{:?}",&tokens);
    
    // Parse & generate AST
    let mut parser = Parser::new();
    let mut ast = match parser.parse(tokens) {
        Ok(ast) => ast,
        Err(e) => panic!("Error: {}", e),
    };

    println!("{:?}", &ast);

    // Loop nodes and evaluate
    let mut interpreter = Interpreter::new();
    interpreter.evaluate(&mut ast); 


//    let image = Image::new(width, height);
//
//    match image_path.extension().map(|s| s.to_str()).flatten() {
//        Some("svg") => {
//            let res = image.save_svg(&image_path);
//            if let Err(e) = res {
//                eprintln!("Error saving svg: {e}");
//                return Err(());
//            }
//        }
//        Some("png") => {
//            let res = image.save_png(&image_path);
//            if let Err(e) = res {
//                eprintln!("Error saving png: {e}");
//                return Err(());
//            }
//        }
//        _ => {
//            eprintln!("File extension not supported");
//            return Err(());
//        }
//    }


    Ok(())
}

