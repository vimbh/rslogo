use std::collections::HashMap;
use crate::parse_test;
use parse_test::{AstNode, Binop, Boolop, Compop};

#[allow(unused)]
pub struct Position {
    x_coordinate: f32,
    y_coordinate: f32,
    color: f32,
    heading: f32,
}

#[derive(Debug)]
pub enum Value {
    Float(f32),
    Bool(bool),
}

#[allow(unused)]
pub struct Evaluator {
    environment: HashMap<String, Value>,
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

    // Root Eval function
    pub fn evaluate(&mut self, ast: Vec<AstNode>) {
        
        for node in ast {
            match node {
                AstNode::MakeOp { var, expr } => self.make_eval(var, &expr), 
                _ => todo!(),
            }
        }
    }


    // Helper fn: evaluates any expr that could return a float (Num, Variable ref)
    fn eval_numeric_expression(&mut self, node: &AstNode) -> f32 {
        
        match node {
            AstNode::Num(val) => *val,
            AstNode::IdentRef(var) => {
                match self.eval_ref(&var) {
                    &Value::Float(num) => num,
                    _ => panic!("Variable {} is bound to a Boolean value, not a Float.", var),
                }
            }
            _ => panic!("Value not recognised"),
        }
    }

    // Helper fn: evaluates any expr that could return a float (Num, Variable ref)
    fn eval_bool_expression(&mut self, node: &AstNode) -> bool {
        
        match node {
            AstNode::IdentRef(var) => {
                match self.eval_ref(&var) {
                    &Value::Bool(value) => value,
                    _ => panic!("Variable {} is bound to a Float value, not a Boolean.", var),
                }
            }
            _ => panic!("Value not recognised"),
        }
    }

    // Eval Binary Operations (+, -, *, /)
    // Binop args must return a num
    fn eval_binary_op(&mut self, operator: &Binop, left: &AstNode, right: &AstNode) -> f32 {
         
        let left_val = match left {
            AstNode::BinaryOp { operator, left, right } => self.eval_binary_op(&operator, &left, &right),
            _ => self.eval_numeric_expression(left),
        };

        let right_val = match right {
            AstNode::BinaryOp { operator, left, right } => self.eval_binary_op(&operator, &left, &right),
            _ => self.eval_numeric_expression(right),
        };

        match operator {
            Binop::Add => left_val + right_val,
            Binop::Sub => left_val - right_val,
            Binop::Mul => left_val * right_val,
            Binop::Div => left_val / right_val,
        }
    }
   
    // Eval Comparison Operations (EQ, NE, GT, LT)
    // Comps args must return a num
    fn eval_comp_op(&mut self, operator: &Compop, left: &AstNode, right: &AstNode) -> bool {
          
        let left_val = match left {
            AstNode::BinaryOp { operator, left, right } => self.eval_binary_op(&operator, &left, &right),
            _ => self.eval_numeric_expression(left),
        };

        let right_val = match right {
            AstNode::BinaryOp { operator, left, right } => self.eval_binary_op(&operator, &left, &right),
            _ => self.eval_numeric_expression(right),
        };
        
        match operator {
            Compop::EQ => left_val == right_val,
            Compop::NE => left_val != right_val,
            Compop::LT => left_val < right_val,
            Compop::GT => left_val > right_val,
        }
    }
   
    // Eval Boolean Operations (AND, OR)
    // Bool args must return a bool
    fn eval_bool_op(&mut self, operator: &Boolop, left: &AstNode, right: &AstNode) -> bool {
       
        let left_val = match left {
            AstNode::BooleanOp { operator, left, right } => self.eval_bool_op(&operator, &left, &right),
            AstNode::ComparisonOp { operator, left, right } => self.eval_comp_op(&operator, &left, &right),
            _ => self.eval_bool_expression(left),
        };

        let right_val = match right {
            AstNode::BooleanOp { operator, left, right } => self.eval_bool_op(&operator, &left, &right),
            AstNode::ComparisonOp { operator, left, right } => self.eval_comp_op(&operator, &left, &right),
            _ => self.eval_bool_expression(right),
        };
        
        match operator {
            Boolop::AND => left_val & right_val,
            Boolop::OR => left_val || right_val,
        }
    }
    
    // Retrieve a variables value
    fn eval_ref(&mut self, var: &String) -> &Value {

        match self.environment.get(var) {
            Some(value) => value,
            _ => panic!("This variable has not been instantiated: {}", &var),
        }
    }

    fn make_eval(&mut self, var: String, expr: &AstNode ) {
        
        let assign_val = match expr {
            AstNode::Num(val) => Value::Float( *val ),
            AstNode::BinaryOp { operator, left, right } => Value::Float( self.eval_binary_op(&operator, &left, &right) ),
            AstNode::ComparisonOp { operator, left, right } => Value::Bool( self.eval_comp_op(&operator, &left, &right) ),
            AstNode::BooleanOp { operator, left, right } => Value::Bool( self.eval_bool_op(&operator, &left, &right) ),
            //AstNode::VarRef
            _ => todo!(),
        };
        
        // Add binding to map
        self.environment.insert(var, assign_val);
        println!("{:?}", self.environment);
    }


}

#[allow(dead_code)]
fn main() {}
