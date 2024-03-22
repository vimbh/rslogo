use std::io::{self, BufRead, BufReader};
use std::fs::File;
pub use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::map_res,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};


#[derive(Debug)] 
enum Expr {
    Term(Box<Term>),
    BinExp(BinOp, Box<Expr>, Box<Term>),
}

#[derive(Debug)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
enum Term {
    Int(i32),
    Id(String),
}
// Returns each word in the file in a Vec<String>, else returns io::error
fn read_file(file_path: std::path::PathBuf) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut file_contents: Vec<String> = Vec::new();

    for read_line in reader.lines() {
        let line = read_line?;
        file_contents.extend(line.split_whitespace().map(String::from));    
    }
    
    Ok(file_contents) 

}

fn expr(input: &str) -> IResult<&str, Expr> {
    alt((bin_exp, term))(input)
}


fn bin_exp(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0(input)?;
    let (input, bin_op) = binop(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, expr1) = expr(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(")")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, term) = term(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, Expr::BinExp(bin_op, Box::new(expr1), Box::new(term))))
}

fn binop(input: &str) -> IResult<&str, BinOp> {
    alt((
        map_res(tag("+"), |_| Ok(BinOp::Add)),
        map_res(tag("-"), |_| Ok(BinOp::Sub)),
        map_res(tag("*"), |_| Ok(BinOp::Mul)),
        map_res(tag("/"), |_| Ok(BinOp::Div)),
    ))(input)
}

fn term(input: &str) -> IResult<&str, Term> {
    alt((int_term, id_term))(input)
}

fn int_term(input: &str) -> IResult<&str, Term> {
    let (input, int_val) = map_res(digit1, |s: &str| s.parse::<i32>())(input)?;
    Ok((input, Term::Int(int_val)))
}

fn id_term(input: &str) -> IResult<&str, Term> {
    let (input, id_val) = map_res(digit1, |s: &str| s.parse::<i32>())(input)?;
    Ok((input, Term::Id(id_val.to_string())))
}



pub fn lexer(file_path: std::path::PathBuf) {
    
    let file_contents = match read_file(file_path) {
        Ok(contents) => contents,
        Err(error) => panic!("Error: {:?}", error),
    };
    
    dbg!(file_contents);
    todo!();
}


fn main() {
    let input = "1 + 2 * 3";
    match expr(input) {
        Ok(("", ast)) => println!("AST: {:?}", ast),
        _ => println!("Parsing failed"),
    }
}
