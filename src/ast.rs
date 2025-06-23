use crate::scope::Scope; 

#[derive(Clone, PartialEq, Debug)]
pub enum Ast_Type {
    AST_FUNCTION_CALL,
    AST_VARIABLE_DEF,
    AST_VARIABLE,
    AST_STRING,
    AST_COMPOUND,
    AST_FUNCTION_DEF,
    AST_FLOAT,
    AST_INT,
    AST_BOOL,
    AST_NOOP,
}
#[derive(Clone, PartialEq, Debug)]
pub enum Data_Type {
    STR,
    INT,
    FLOAT, //uses 64-bit memory so same as a double
    VOID,
    CHAR, 
    BOOL, 
}

#[derive(Clone, PartialEq, Debug)]
pub struct AST {
    pub ast_type: Ast_Type,
    pub data_type: Data_Type, 

    // Add in the scope
    pub scope: Option<Box<Scope>>, 

    pub variable_definition_variable_name: Option<String>,
    pub variable_definition_value: Option<Box<AST>>,
    pub variable_type: Option<Data_Type>,
    pub is_const: Option<bool>,

    pub variable_name: Option<String>,
    
    pub function_call_name: Option<String>,
    pub function_call_args: Option<Vec<AST>>,

    pub string_value: Option<String>,
    pub concatenated: Option<bool>,

    pub float_value: Option<f64>,
    pub past_decimal: Option<i32>,
    pub float_init: Option<bool>, 

    pub int_value: Option<i32>,
    pub int_init: Option<bool>,

    pub bool_value: Option<bool>,
    pub bool_init: Option<bool>,

    pub compound_value: Option<Vec<AST>>,

    pub function_definition_body: Option<Box<AST>>,
    pub function_definition_name: Option<String>,
    pub function_definition_args: Option<Vec<AST>>,

    pub class_definition_body: Option<Box<AST>>,
    pub class_definition_name: Option<String>,
    pub class_defintiion_args: Option<Vec<AST>>,

    pub class_call_name: Option<String>,
    pub class_call_args: Option<Vec<AST>>,
}

impl AST {
    pub fn new(ast_type: Ast_Type) -> Self {
        AST {
            ast_type,
            data_type: Data_Type::VOID,
            
            scope: None, 

            variable_definition_variable_name: None,
            variable_definition_value: None,
            variable_type: None,
            is_const: None,

            variable_name:None,

            function_call_name:None,
            function_call_args:None,
            
            string_value:None,
            concatenated:None,

            float_value:None,
            past_decimal:None,
            float_init:None,

            int_value:None,
            int_init:None,

            bool_value:None,
            bool_init:None,

            compound_value:None,

            function_definition_body:None,
            function_definition_name:None,
            function_definition_args:None,

            class_definition_body:None,
            class_definition_name:None,
            class_defintiion_args:None,
    
            class_call_name:None,
            class_call_args:None,
        }
    }

    pub fn print(&self) {
        match self.ast_type {
            Ast_Type::AST_STRING => print!("{}", self.string_value.as_ref().unwrap()),
            Ast_Type::AST_INT => print!("{}", self.int_value.unwrap()),
            Ast_Type::AST_FLOAT => print!("{:.precision$}", self.float_value.unwrap(), precision = self.past_decimal.unwrap_or(2) as usize),
            Ast_Type::AST_BOOL => print!("{}", if self.bool_value.unwrap() { "true" } else { "false" }),
            _ => println!("<unhandled type>"),
        }
    }
}
