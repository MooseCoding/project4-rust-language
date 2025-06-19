use crate::scope::Scope; 

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

pub enum Data_Type {
    STR,
    INT,
    FLOAT, //uses 64-bit memory so same as a double
    VOID,
    CHAR, 
    BOOL,
}

pub struct AST {
    pub ast_type: Ast_Type,
    pub data_type: Data_Type, 

    // Add in the scope

    pub variable_definition_variable_name: Option<String>,
    pub variable_definition_value: Option<Box<AST>>,
    pub variable_type: Option<Data_Type>,
    pub is_const: bool,

    pub variable_name: Option<String>,
    
    
}

