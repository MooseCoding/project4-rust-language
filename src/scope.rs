use crate::ast::AST;

#[derive(Clone, PartialEq, Debug)]
pub struct Scope {
    pub function_definition: Vec<AST>,
    pub variable_definition: Vec<AST>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            function_definition: Vec::new(),
            variable_definition: Vec::new(),
        }
    }

    pub fn add_function_definition(&mut self, func_def: AST) -> &AST {
        self.function_definition.push(func_def);
        self.function_definition.last().unwrap()
    }

    pub fn get_function_definition(&self, name: &str) -> Option<&AST> {
        self.function_definition.iter().find(
            |f| {
                f.function_definition_name.as_deref() == Some(name)
            }
        )
    }

    pub fn add_variable_definition(&mut self, var_def: AST) -> &AST {
        self.variable_definition.push(var_def);
        self.variable_definition.last().unwrap()
    }

    pub fn get_variable_definition(&self, name: &str) -> Option<&AST>{
        self.variable_definition.iter().find(
            |v| {
                v.variable_definition_variable_name.as_deref() == Some(name)
            }
        )
    }
}