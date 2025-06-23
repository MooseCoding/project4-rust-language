#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_parens, unused_variables, unused_mut, dead_code)]

mod lexer;
mod token;
mod parser; 
mod scope;
mod ast; 
mod visitor; 
use lexer::Lexer; 
use parser::Parser; 
use scope::Scope; 
use visitor::Visitor; 

pub fn main() {
    let source = "int x = 42; println(x);";
    let mut lexer = Lexer::new(source);
    let mut global_scope = Scope::new(); 
    let mut parser: Parser = Parser::new(&mut lexer, &mut global_scope);
    let mut ast = parser.parse();
    let mut visitor = Visitor::new(); 
    visitor.visit( &mut ast);
}