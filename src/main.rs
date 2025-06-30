#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_parens, unused_variables, unused_mut, dead_code)]

mod scope;
mod lexer;
mod ast; 
mod token; 
mod visitor; 

use std::env;
use std::fs;

use std::cell::RefCell;
use std::rc::Rc;

use lexer::Lexer;
use visitor::Visitor; 
use scope::Scope; 


mod parser;
use parser::Parser;

pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Unexpected input");
    }

    let n = &args[1];
    let source = fs::read_to_string(n)
        .unwrap_or_else(|_| panic!("Could not read the file {}", n));

    let mut lexer = Lexer::new(&source);
    let mut global_scope = Rc::new(RefCell::new(Scope::new())); 
    let mut parser: Parser = Parser::new(&mut lexer, global_scope);
    let mut ast = parser.parse();
    let mut visitor = Visitor::new(); 
    visitor.visit( &mut ast);
}