use crate::ast::{AST, Ast_Type};
use std::collections::HashMap;

pub struct Visitor {
    pub builtins: HashMap<String, Box<dyn Fn(&[AST]) -> AST>>,
}

impl Visitor {
    pub fn new() -> Self {
        let mut b: HashMap<String, Box<dyn Fn(&[AST]) -> AST>> = HashMap::new();

        b.insert("print".to_string(), Box::new(|args: &[AST]| {
            for arg in args {
                arg.print();
            }
            AST::new(Ast_Type::AST_NOOP)
        }));

        b.insert("println".to_string(), Box::new(|args: &[AST]| {
            for arg in args {
                arg.print();
            }
            println!();
            AST::new(Ast_Type::AST_NOOP)
        }));

        Visitor { builtins: b }
    }

    pub fn visit(&mut self, node: &mut AST) -> AST {
        match node.ast_type {
            Ast_Type::AST_VARIABLE => self.visit_variable(node),
            Ast_Type::AST_VARIABLE_DEF => self.visit_variable_definition(node),
            Ast_Type::AST_COMPOUND => self.visit_compound(node),
            Ast_Type::AST_FUNCTION_CALL => self.visit_function_call(node),
            Ast_Type::AST_FUNCTION_DEF => self.visit_function_definition(node),
            _ => node.clone(),
        }
    }

    pub fn visit_variable(&mut self, node: &mut AST) -> AST {
        let name = node.variable_name.as_ref().expect("Variable name is missing");
        let scope = node.scope.as_ref().expect("Scope is missing");

        if let Some(v) = scope.get_variable_definition(name) {
            let val = v.variable_definition_value.as_ref().expect("Variable has no value");
            return self.visit(&mut val.clone());
        }

        panic!("Undefined variable: {}", name);
    }

    pub fn visit_variable_definition(&mut self, node: &mut AST) -> AST {
        let name = node
            .variable_definition_variable_name
            .as_ref()
            .expect("Var def missing name");

        if let Some(scope) = node.scope.as_ref() {
            if scope.get_variable_definition(name).is_none() {
                let scope = node.scope.as_ref().unwrap();
                let mut new_scope = scope.clone();
                new_scope.add_variable_definition(node.clone());
            }
        }

        node.clone()
    }

    pub fn visit_function_call(&mut self, node: &mut AST) -> AST {
        let name = node.function_call_name.as_ref().expect("Missing function name");

        let empty_args = Vec::new(); // Fix for temporary drop issue
        let args_raw = node.function_call_args.as_ref().unwrap_or(&empty_args);

        let args: Vec<AST> = {
            let mut visited = Vec::new();
            for arg in args_raw {
                visited.push(self.visit(&mut arg.clone()));
            }
            visited
        };

        if let Some(f) = self.builtins.get(name) {
            return f(&args);
        }

        // User-defined
        let def_scope = node.scope.as_ref().expect("Function call missing scope");
        let def = def_scope.get_function_definition(name).expect("Function not defined");

        let empty_p = vec![];
        let empty_a = vec![]; 

        let params = {
            def.function_definition_args.as_ref().unwrap_or(&empty_p)
        };

        let args = {
            node.function_call_args.as_ref().unwrap_or(&empty_a)
        };

        if args.len() != params.len() {
            panic!(
                "Function {} expected {} arguments, got {}",
                name,
                params.len(),
                args.len()
            );
        }

        let f_scope = def.scope.as_ref().expect("Function def missing scope");

        let mut new_scope = f_scope.clone();

        for (arg, param) in args.iter().zip(params.iter()) {
            if param.variable_type.as_ref() != Some(&arg.data_type) {
                panic!(
                    "Function {} argument type mismatch: expected {:?}, got {:?}",
                    name, param.variable_type, arg.data_type
                );
            }

            let mut var_def = AST::new(Ast_Type::AST_VARIABLE_DEF);
            var_def.variable_definition_variable_name = param.variable_name.clone();
            var_def.variable_definition_value = Some(Box::new(arg.clone()));
            var_def.variable_type = param.variable_type.clone();

            new_scope.add_variable_definition(var_def);
        }

        let mut body = *def.function_definition_body.as_ref().expect("Missing body").clone();
        body.scope = Some(Box::new(*new_scope));

        self.visit(&mut body)
    }

    pub fn visit_compound(&mut self, node: &mut AST) -> AST {
        if let Some(compound) = node.compound_value.as_mut() {
            for s in compound.iter_mut() {
                self.visit(s);
            }
        }
        AST::new(Ast_Type::AST_NOOP)
    }

    pub fn visit_function_definition(&mut self, node: &mut AST) -> AST {
        if let Some(scope) = node.scope.as_ref() {
            let mut new_scope = scope.clone();
            new_scope.add_function_definition(node.clone());
        }
        node.clone()
    }
}