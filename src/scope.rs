use crate::ast::{Ast_Type, AST};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

pub type SharedScope = Rc<RefCell<Scope>>;

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub function_definitions: Vec<AST>,
    pub variable_definitions: Vec<AST>,
    pub imports: Vec<AST>, 
    pub parent: Option<SharedScope>
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scope")
            .field("variable_definitions", &self.variable_definitions)
            .field("function_definitions", &self.function_definitions)
            .field("parent", &if self.parent.is_some() {
                &"Some(...)"
            }
            else {
                &"None"
            })
            .finish()
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            function_definitions: Vec::new(),
            variable_definitions: Vec::new(),
            imports: Vec::new(), 
            parent: None::<SharedScope>,
        }
    }

    pub fn new_with_parent(parent: SharedScope) -> Self {
        Scope {
            function_definitions: Vec::new(),
            variable_definitions: Vec::new(),
            imports: Vec::new(), 
            parent: Some(parent),
        }
    }

    pub fn get_variable_definition(&self, name: &str) -> Option<AST> {
        for def in &self.variable_definitions {
            if let Some(def_name) = &def.variable_definition_variable_name {
                if def_name == name {
                    return Some(def.clone());
                }
            }
            else if let Some(def_name) = &def.array_name {
                if def_name == name {
                    return Some(def.clone()); 
                }
            }
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get_variable_definition(name);
        }
        None
    }

    pub fn add_variable_definition(&mut self, def: AST) {
        if def.ast_type == Ast_Type::AST_VARIABLE_DEF {
            let name = def
                .variable_definition_variable_name
                .as_ref()
                .expect("Var def has no name");
            for existing_def in self.variable_definitions.iter_mut() {
                if let Some(existing_name) = &existing_def.variable_definition_variable_name {
                    if existing_name == name {
                        *existing_def = def; 
                        return;
                    }
                }
            }
        }
        else if def.ast_type == Ast_Type::AST_ARRAY_DEF {
            let name = def
                .array_name 
                .as_ref()
                .expect("Array def has no name");
            for existing_def in self.variable_definitions.iter_mut() {
                if let Some(existing_name) = &existing_def.variable_definition_variable_name {
                    if existing_name == name {
                        *existing_def = def; 
                        return;
                    }
                }
            }
        }

        self.variable_definitions.push(def);
    }

    pub fn get_function_definition(&self, name: &str) -> Option<AST> {
        for def in &self.function_definitions {
            if let Some(def_name) = &def.function_definition_name {
                if def_name == name {
                    return Some(def.clone());
                }
            }
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get_function_definition(name);
        }
        None
    }

    pub fn add_function_definition(&mut self, def: AST) {
        self.function_definitions.push(def);
    }

    pub fn update_variable_definition(&mut self, name: String, new_def: AST) {
        for v in &mut self.variable_definitions {
            if v.variable_definition_variable_name == Some(name.clone()) {
                *v = new_def;
                return;
            }
            if v.array_name == Some(name.clone()) {
                *v = new_def; 
                return; 
            }
        }

        if let Some(ref parent) = self.parent {
            parent.borrow_mut().update_variable_definition(name, new_def);
        }
        else {
            panic!("Variable {} not found in any scope", name);
        }
    }

    pub fn add_import(&mut self, import: &AST) {
        self.imports.push(import.clone());
    }

    pub fn get_import(&self, import_name: &str) -> Option<AST>  {
        for i in &self.imports {
            if let Some(name) = &i.variable_name {
                if name == import_name {
                    return Some(i.clone()) 
                }
            }
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get_import(import_name);
        }
        None
    }

   pub fn update_import(&mut self, mut import: AST) {
        for i in &mut self.imports {
            if import.variable_name.as_ref().unwrap() == i.variable_name.as_ref().unwrap() {
                *i = import.clone();
                return;
            }
        }
        self.imports.push(import);
    }

    pub fn print_all_imports(&self) {
        for import in &self.imports {
            let name = import.variable_name.as_ref().unwrap();
            println!("Import {}", name); 
        }
    }

    pub fn print_all(&self) {
        for import in &self.imports {
            let name = import.variable_name.as_ref().unwrap();
            println!("Import {}", name); 
        }
        for func in &self.function_definitions {
            let f_name = func.function_definition_name.as_ref().unwrap();
            println!("Func name {}", f_name);
        }
        for var in &self.variable_definitions {
            let name = var.variable_definition_variable_name.as_ref().unwrap();
            println!("Var {}", name); 
        }
    }
}