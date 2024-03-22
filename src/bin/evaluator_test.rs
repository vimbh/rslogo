use std::collections::HashMap;
use crate::parse_test;
use parse_test::{AstNode, Binop};

#[allow(unused)]
pub struct Position {
    x_coordinate: f32,
    y_coordinate: f32,
    color: f32,
    heading: f32,
}

#[allow(unused)]
pub struct Evaluator {
    environment: HashMap<String, f32>,
    turtle : Position,
}

impl Evaluator {

    // Constructor
    pub fn new() -> Self {
        Self {
            environment: HashMap::new(),
            turtle: Position {
                x_coordinate: 0.0,
                y_coordinate: 0.0,
                color: 0.0,
                heading: 0.0,
            } 
        }
    }

    // Master eval func
    pub fn evaluate(&mut self, ast: Vec<AstNode>) {
        
        for node in ast {
            //println!("{:?}", &node);
            
            match node {
                AstNode::MakeOp { var, expr } => self.make_eval(var, &expr), 
                _ => todo!(),
            }


        }
    }

    fn eval_num(&mut self, val: &f32) -> f32 {
        *val
    }

    fn eval_binary_op(&mut self, operator: &Binop, left: &AstNode, right: &AstNode) -> f32 {
            
        let left_val = match left {
            AstNode::Num(val) => self.eval_num(&val),
            AstNode::BinaryOp { operator, left, right } => self.eval_binary_op(&operator, &left, &right), 
            _ => panic!("Value not recognised"),
        };
        
        let right_val = match right {
            AstNode::Num(val) => self.eval_num(&val),
            AstNode::BinaryOp { operator, left, right } => self.eval_binary_op(&operator, &left, &right), 
            _ => panic!("Value not recognised"),
        };

        match operator {
            Binop::Add => left_val + right_val,
            Binop::Sub => left_val - right_val,
            Binop::Mul => left_val * right_val,
            Binop::Div => left_val / right_val,
        }
    }

    fn make_eval(&mut self, var: String, expr: &AstNode ) {
        
        let assign_val = match expr {
            AstNode::Num(val) => self.eval_num(&val),
            AstNode::BinaryOp { operator, left, right } => self.eval_binary_op(&operator, &left, &right), 
            //AstNode::CompOp
            //ASTNode::BoolOp
            //AstNode::VarRef
            _ => todo!(),
        };
        println!("{}, {}", var, assign_val);
        
        // Add binding to map
        self.environment.insert(var, assign_val);
        println!("{:?}", self.environment);
    }


}

fn main() {

}
