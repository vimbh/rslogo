use clap::Parser as clapParser;
use unsvg::Image;
use logolang_lib::{interpreter, lexer, parser};
use logolang_lib::logolang_errors::ImgFileError;
use lexer::tokenize;
use parser::Parser;
use interpreter::Interpreter;
use anyhow::Result;

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


fn main() -> Result<()> {
    let args: Args = Args::parse();
    // Access the parsed arguments
    let file_path = args.file_path;
    let image_path = args.image_path;
    let image_width = args.width;
    let image_height = args.height;
    
    // Generate Tokens, manage errors
    let tokens = match tokenize(file_path) {
        Ok(tokens) => tokens,
        Err(e) => {
  
            return Err(e.into());
        },

    };
    println!("{:?}",&tokens);
    
    // Parse & generate AST
    let mut parser = Parser::new();
    let mut ast = match parser.parse(tokens) {
        Ok(ast) => ast,
        Err(e) => panic!("Error: {}", e),
    };

    println!("{:?}", &ast);

    let mut empty_image = Image::new(image_width, image_height);

    // Loop nodes and evaluate
    let mut interpreter = Interpreter::new(&mut empty_image);
    let drawing = interpreter.run(&mut ast); 
    
    if let Ok(image) = drawing {

        match image_path.extension().map(|s| s.to_str()).flatten() {
            Some("svg") => {
                let res = image.save_svg(&image_path);
                if let Err(e) = res {
                    eprintln!("Error saving svg: {e}");
                    return Err(e.into());
                }
            }
            Some("png") => {
                let res = image.save_png(&image_path);
                if let Err(e) = res {
                    eprintln!("Error saving png: {e}");
                    return Err(e.into());
                }
            }
            _ => {
                eprintln!("File extension not supported");
                return Err(ImgFileError::UnsupportedFileExtension.into());
            }
        }

   }

    Ok(())
}

