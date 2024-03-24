use core::{f32, panic};
use std::collections::HashMap;
use crate::parse_test;
use nom::number::complete::float;
use parse_test::{AstNode, Binop, Boolop, Compop, PenPos, QueryKind};

#[allow(unused)]
#[derive(Debug)]
pub struct Position {
    x_coordinate: f32,
    y_coordinate: f32,
    direction: f32,
}

#[derive(Debug)]
pub enum Value {
    Float(f32),
    Bool(bool),
}

#[allow(unused)]
pub struct Evaluator {
    environment: HashMap<String, Value>,
    current_position : Position,
    current_color: u8,
    currently_drawing: bool,
}



impl Evaluator {

    // Constructor
    pub fn new() -> Self {
        Self {
            environment: HashMap::new(),
            current_position: Position {
                x_coordinate: 0.0,
                y_coordinate: 0.0,
                direction: 0.0,
            },
            current_color: 7, // Starts default w_hite
            currently_drawing: false, // Starts default penup (not drawing)  
        }
    }

    // Root Eval function
    pub fn evaluate(&mut self, ast: Vec<AstNode>) {
        
        for node in ast {
            match node {
                AstNode::MakeOp { var, expr } => self.make_eval(var, &expr),
                AstNode::PenPosUpdate { update_type, value } => self.set_position(&update_type, &value),
                AstNode::PenStatusUpdate(new_drawing_status) => self.set_drawing_status(new_drawing_status),
                AstNode::PenColorUpdate(new_pen_color) => self.set_pen_color(&new_pen_color),
                _ => todo!(),
            }
        }

        println!("{:?}", self.environment);
        println!("{:?}", self.current_position);
        println!("{:?}", self.currently_drawing);
        println!("{:?}", self.current_color);

    }


    // Helper fn: evaluates any expr that could return a float (Num, Variable ref, Query)
    fn eval_numeric_expression(&mut self, node: &AstNode) -> f32 {
        
        match node {
            AstNode::Num(val) => *val,
            AstNode::IdentRef(var) => {
                match self.eval_ref(&var) {
                    &Value::Float(num) => num,
                    _ => panic!("Variable {} is bound to a Boolean value, not a Float.", var),
                }
            },
            AstNode::Query(query_kind) => self.query(&query_kind),
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

    // Drawing status Setter
    fn set_drawing_status(&mut self, new_drawing_status: bool) {
        
        self.currently_drawing = new_drawing_status;
    }
 
    // Pen color setter
    fn set_pen_color(&mut self, value: &AstNode) {
        
        let float_val = self.eval_numeric_expression(value);
        
        // Check precision & bounds before casting to an int color
        if float_val == (float_val as u8) as f32 && float_val >= 0.0 && float_val <= 15.0 {
            self.current_color = float_val as u8;
        } else {
            panic!("SETPENCOLOR requires an integer from 0..15 as an argument");
        };

    }   
    
    // Position Setter
    fn set_position(&mut self, update_type: &PenPos, value: &AstNode ) {
       
        let val = self.eval_numeric_expression(value);
        
        match update_type {
            PenPos::SETX => self.current_position.x_coordinate = val,
            PenPos::SETY => self.current_position.y_coordinate = val,
            PenPos::SETHEADING => self.current_position.direction = val,
            PenPos::TURN => self.current_position.direction += val,
        }
    
    }

    fn query(&mut self, query_kind: &QueryKind) -> f32 {
        
        match query_kind {
            QueryKind::XCOR => self.current_position.x_coordinate,
            QueryKind::YCOR => self.current_position.y_coordinate,
            QueryKind::HEADING => self.current_position.direction,
            QueryKind::COLOR => self.current_color as f32, 
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

    // Bind a variable to a value
    // args must return a float or bool
    fn make_eval(&mut self, var: String, expr: &AstNode ) {
        
        let assign_val = match expr {
            AstNode::Num(val) => Value::Float( *val ),
            AstNode::BinaryOp { operator, left, right } => Value::Float( self.eval_binary_op(&operator, &left, &right) ),
            AstNode::ComparisonOp { operator, left, right } => Value::Bool( self.eval_comp_op(&operator, &left, &right) ),
            AstNode::BooleanOp { operator, left, right } => Value::Bool( self.eval_bool_op(&operator, &left, &right) ),
            AstNode::Query(query_kind) => Value::Float( self.query(&query_kind) ),

            //AstNode::VarRef
            _ => todo!(),
        };
        
        // Add binding to map
        self.environment.insert(var, assign_val);
    
    }


}

#[allow(dead_code)]
fn main() {}
