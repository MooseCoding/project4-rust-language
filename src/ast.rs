use crate::scope::{SharedScope};
use crate::token::{Types};

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
    AST_BINARY,
    AST_RETURN,
    AST_IF,
    AST_WHILE,
    AST_REASSIGN,
    AST_FOR,
    AST_INCREMENT, 
    AST_DECREMENT, 
    AST_UNARY,
    AST_ARRAY_ACCESS,
    AST_ARRAY_DEF, 
    AST_ARRAY_ASSIGN, 
    AST_IMPORT,
    AST_DOT, 
}
#[derive(Clone, PartialEq, Debug)]
pub enum Data_Type {
    STR,
    INT, // uses 32-bit memory
    FLOAT, //uses 64-bit memory so same as a double
    VOID,
    CHAR, 
    BOOL, 
    ARRAY(Box<Data_Type>), 
}

#[derive(Clone, PartialEq, Debug)]
pub struct AST {
    pub ast_type: Ast_Type,
    pub data_type: Data_Type, 

    // Add in the scope
    pub scope: Option<SharedScope>, 

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

    pub left: Option<Box<AST>>,
    pub right: Option<Box<AST>>,
    pub operator: Option<Types>,

    pub return_value: Option<Box<AST>>,

    // new stuff that doesnt need to be implemented rn
    
    pub if_condition: Option<Box<AST>>,
    pub if_body: Option<Box<AST>>,
    pub else_body: Option<Box<AST>>,

    pub while_condition: Option<Box<AST>>,
    pub while_body: Option<Box<AST>>,

    pub reassign_name: Option<String>,
    pub reassign_value: Option<Box<AST>>,

    pub for_init: Option<Box<AST>>,
    pub for_condition: Option<Box<AST>>,
    pub for_increment: Option<Box<AST>>,
    pub for_body: Option<Box<AST>>, 

    pub array_elements: Option<Vec<AST>>,
    pub array_name: Option<String>,
    pub array_index: Option<Box<AST>>,
    pub array_assign_value: Option<Box<AST>>,

    pub dot_left: Option<Box<AST>>,
    pub dot_right: Option<Box<AST>>, 

    pub is_builtin: Option<bool>, 
    pub imported_ast: Option<Box<AST>>, 
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

            left: None,
            right:None,
            operator:None::<Types>,

            return_value: None,

            if_condition: None,
            if_body: None, 
            else_body: None, 

            while_condition: None,
            while_body: None, 

            reassign_name: None,
            reassign_value: None,

            for_init:None,
            for_condition:None,
            for_increment:None,
            for_body:None,

            array_elements:None,
            array_assign_value:None,
            array_index:None,
            array_name:None, 

            dot_left:None,
            dot_right:None, 

            is_builtin:None, 
            imported_ast:None, 
        }
    }

    pub fn from_bool(b: bool) -> AST {
        let mut node = AST::new(Ast_Type::AST_BOOL);

        node.bool_init = Some(true);
        node.bool_value = Some(b);

        node.data_type = Data_Type::BOOL;

        node
    }

    pub fn print(&self) {
        match self.ast_type {
            Ast_Type::AST_STRING => print!("{}", self.string_value.as_ref().unwrap()),
            Ast_Type::AST_INT => print!("{}", self.int_value.unwrap()),
            Ast_Type::AST_FLOAT => print!("{:.precision$}", self.float_value.unwrap(), precision = self.past_decimal.unwrap_or(2) as usize),
            Ast_Type::AST_BOOL => print!("{}", if self.bool_value.unwrap() { "true" } else { "false" }),
            Ast_Type::AST_VARIABLE_DEF => {
                for ast in self.variable_definition_value.as_ref() {
                    ast.print();
                }
            } 
            Ast_Type::AST_VARIABLE => {
                let t = self.scope.as_ref().unwrap().borrow().get_variable_definition(self.variable_name.as_ref().unwrap());
                t.unwrap().print(); 
            }
            Ast_Type::AST_RETURN => {
                self.return_value.as_ref().unwrap().print();
            }
            Ast_Type::AST_ARRAY_DEF => {
                print!("[");
                if let Some(elements) = &self.array_elements {
                    for (i, element) in elements.iter().enumerate() {
                        element.print();
                        if i < elements.len() -1 {
                            print!(", ");
                        }
                    }
                }
                print!("]");
            }
            _ => println!("<unhandled type>, {:#?}", self.ast_type),
        }
    }
}
