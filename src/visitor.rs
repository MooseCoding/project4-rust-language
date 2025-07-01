use crate::ast::{AST, Ast_Type, Data_Type};
use crate::token::{Types};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::scope::{Scope};


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
            Ast_Type::AST_COMPOUND => self.visit_compound(node),
            Ast_Type::AST_FUNCTION_CALL => self.visit_function_call(node),
            Ast_Type::AST_BINARY => self.visit_binary(node),
            Ast_Type::AST_RETURN => self.visit_return(node),
            Ast_Type::AST_IF => self.visit_if(node),
            Ast_Type::AST_WHILE => self.visit_while(node), 
            Ast_Type::AST_REASSIGN => self.visit_reassign(node),
            Ast_Type::AST_INCREMENT => self.visit_increment(node),
            Ast_Type::AST_DECREMENT => self.visit_decrement(node), 
            Ast_Type::AST_FOR => self.visit_for(node),
            _ => node.clone(),
        }
    }

    pub fn visit_variable(&mut self, node: &mut AST) -> AST {
        let name = node.variable_name.as_ref().expect("Variable name is missing");
        let scope = node.scope.as_ref().expect("Scope is missing");

        if let Some(var_def) = scope.borrow().get_variable_definition(name) {
            if let Some(val) = &var_def.variable_definition_value {
                return *val.clone();
            } else {
                panic!("Variable '{}' has no value in scope {:p}", name, &*node.scope.as_ref().expect(""));
            }
        }

        panic!("Undefined variable: {}", name);
    }

    pub fn visit_function_call(&mut self, node: &mut AST) -> AST {
        let name = node.function_call_name.as_ref().expect("Missing function name");

        let empty_vec = &vec![];

        let args_vec = node.function_call_args.as_ref().unwrap_or(empty_vec).clone();
        let mut evaluated_args = args_vec
            .into_iter()
            .map(|mut arg| self.visit(&mut arg))
            .collect::<Vec<_>>();


        if let Some(f) = self.builtins.get(name) {
            return f(&evaluated_args);
        }

        let def_scope = node.scope.as_ref().expect("Function call missing scope");
        let def = def_scope
            .borrow()
            .get_function_definition(name)
            .unwrap_or_else(|| panic!("Function '{}' not defined", name));

        let params = def.function_definition_args.as_ref().unwrap_or(empty_vec);

        if evaluated_args.len() != params.len() {
            panic!(
                "Function {} expected {} arguments, got {}",
                name,
                params.len(),
                evaluated_args.len()
            );
        }

        let func_scope = def.scope.as_ref().expect("Function def missing scope");
        let new_scope = Rc::new(RefCell::new(Scope::new_with_parent(func_scope.clone())));

        for (param, arg) in params.iter().zip(evaluated_args.iter()) {
            let expected = param.variable_type.as_ref().unwrap();

            if param.variable_type.as_ref() != Some(&arg.data_type) && !(expected == &Data_Type::FLOAT && arg.data_type == Data_Type::INT){
                panic!(
                    "Function {} argument type mismatch: expected {:?}, got {:?}",
                    name, param.variable_type, arg.data_type
                );
            }

            let mut value = arg.clone();
            value.scope = Some(new_scope.clone()); 

            let mut var_def = AST::new(Ast_Type::AST_VARIABLE_DEF);
            var_def.variable_definition_variable_name = param.variable_definition_variable_name.clone(); 
            var_def.variable_definition_value = Some(Box::new(value));
            var_def.variable_type = param.variable_type.clone();
            var_def.scope = Some(new_scope.clone());

            new_scope.borrow_mut().add_variable_definition(var_def);
        }

        let mut body = def.function_definition_body.as_ref().expect("Missing body").clone();
        self.set_scope_recursively(&mut body, new_scope.clone());
        let result = self.visit(&mut body);

        if result.ast_type == Ast_Type::AST_RETURN {
            if let Some(value) = result.return_value {
                return *value;
            }
            else {
                return result; 
            }
        }

        AST::new(Ast_Type::AST_NOOP)
    }

    pub fn visit_compound(&mut self, node: &mut AST) -> AST {
        if let Some(compound) = node.compound_value.as_mut() {
            let scope = node.scope.as_mut().expect("Compound block missing scope");

            for stmt in compound.iter_mut() {
                if stmt.ast_type == Ast_Type::AST_FUNCTION_DEF {
                    let func_name = stmt.function_definition_name.as_ref().unwrap().clone();
                    scope.borrow_mut().add_function_definition(stmt.clone());
                }
                else if stmt.ast_type == Ast_Type::AST_VARIABLE_DEF {
                    let var_name = stmt.variable_definition_variable_name.as_ref().unwrap().clone();

                    self.set_scope_recursively(stmt, scope.clone());

                    if let Some(value_expr) = stmt.variable_definition_value.as_mut() {
                        let evaluated = self.visit(value_expr);
                        stmt.variable_definition_value = Some(Box::new(evaluated));
                    }

                    scope.borrow_mut().add_variable_definition(stmt.clone());
                }
                else {
                    let result = self.visit(stmt);
                    if result.ast_type == Ast_Type::AST_RETURN {
                        return result;
                    }
                }
            }
        }

        AST::new(Ast_Type::AST_NOOP)
    }

    fn set_scope_recursively(&self, node: &mut AST, scope: Rc<RefCell<Scope>>) {
        node.scope = Some(scope.clone());

        if let Some(children) = node.compound_value.as_mut() {
            for child in children.iter_mut() {
                self.set_scope_recursively(child, scope.clone());
            }
        }

        if let Some(left) = node.left.as_mut() {
            self.set_scope_recursively(left, scope.clone());
        }

        if let Some(right) = node.right.as_mut() {
            self.set_scope_recursively(right, scope.clone());
        }

        if let Some(args) = node.function_call_args.as_mut() {
            for arg in args.iter_mut() {
                self.set_scope_recursively(arg, scope.clone());
            }
        }

        if let Some(body) = node.function_definition_body.as_mut() {
            self.set_scope_recursively(body, scope.clone());
        }

        if let Some(val) = node.variable_definition_value.as_mut() {
            self.set_scope_recursively(val, scope.clone());
        }

        if let Some(r) = node.return_value.as_mut() {
            self.set_scope_recursively(r, scope.clone());
        }
    
        if let Some(body) = node.for_body.as_mut() {
            self.set_scope_recursively(body, scope.clone());
        }

        if let Some(init) = node.for_init.as_mut() {
            self.set_scope_recursively(init, scope.clone());
        }

        if let Some(increment) = node.for_increment.as_mut() {
            self.set_scope_recursively(increment, scope.clone());
        }

        if let Some(cond) = node.for_condition.as_mut() {
            self.set_scope_recursively(cond, scope.clone());
        }

        if let Some(r) = node.reassign_value.as_mut()  {
            self.set_scope_recursively(r, scope.clone());
        }
    }

    pub fn visit_binary(&mut self, node: &mut AST) -> AST {
        let op = node.operator.as_ref().expect("Missing operator");

        let left_eval = self.visit(node.left.as_mut().expect("Missing left operand"));
        let right_eval = self.visit(node.right.as_mut().expect("Missing right operand"));

        if *op == Types::TOKEN_ADD {
            if let (Some(ls), Some(rs)) = (&left_eval.string_value, &right_eval.string_value) {
                let mut n = AST::new(Ast_Type::AST_STRING);
                n.string_value = Some(format!("{}{}", ls, rs));
                n.scope = Some(node.scope.clone().unwrap_or_else(|| Rc::new(RefCell::new(crate::scope::Scope::new()))));
                return n;
            }
        }

        let (l_val, r_val) = (
            left_eval.float_value.or_else(|| left_eval.int_value.map(|v| v as f64)),
            right_eval.float_value.or_else(|| right_eval.int_value.map(|v| v as f64)),
        );

        match (l_val, r_val) {
            (Some(l), Some(r)) => {
                let result = match op {
                    Types::TOKEN_ADD => l + r,
                    Types::TOKEN_SUBTRACT => l - r,
                    Types::TOKEN_ASTERISK => l * r,
                    Types::TOKEN_FSLASH => l / r,
                    Types::TOKEN_GREATER_THAN => return AST::from_bool(l > r),
                    Types::TOKEN_LESS_THAN => return AST::from_bool(l < r),
                    Types::TOKEN_LEQ => return AST::from_bool(l <= r),
                    Types::TOKEN_GEQ => return AST::from_bool(l >= r),
                    Types::TOKEN_EE => return AST::from_bool(l == r),
                    _ => panic!("Unknown operator"),
                };

                let result_type = if left_eval.float_init.unwrap_or(false)
                    || right_eval.float_init.unwrap_or(false)
                {
                    Data_Type::FLOAT
                } else {
                    Data_Type::INT
                };

                let mut node = match result_type {
                    Data_Type::FLOAT => {
                        let mut n = AST::new(Ast_Type::AST_FLOAT);
                        n.float_value = Some(result);
                        n.float_init = Some(true);
                        n.data_type = Data_Type::FLOAT;
                        n.scope = Some(node.scope.clone().unwrap_or_else(|| Rc::new(RefCell::new(crate::scope::Scope::new()))));                       
                        n
                    }
                    Data_Type::INT => {
                        let mut n = AST::new(Ast_Type::AST_INT);
                        n.int_value = Some(result as i32);
                        n.int_init = Some(true);
                        n.data_type = Data_Type::INT;
                        n.scope = Some(node.scope.clone().unwrap_or_else(|| Rc::new(RefCell::new(crate::scope::Scope::new()))));
                        n
                    }
                    _ => panic!("Unsupported type"),
                };

                node
            }
            _ => panic!("Invalid operands to binary expression"),
        }
    }

    pub fn visit_if(&mut self, node: &mut AST) -> AST {
        let condition = self.visit(node.if_condition.as_mut().expect("No if condition"));

        let is_true = match condition.ast_type {
            Ast_Type::AST_BOOL => condition.bool_value.unwrap_or(false),
            Ast_Type::AST_INT => condition.int_value.unwrap_or(0) != 0,
            Ast_Type::AST_FLOAT => condition.float_value.unwrap_or(0.0) != 0.0,
            _ => panic!("Invalid type for if statement"),
        };

        if is_true {
            self.visit(node.if_body.as_mut().expect("Missing if body"))
        }
        else if let Some(e) = node.else_body.as_mut() {
            self.visit(e)
        }
        else {
            AST::new(Ast_Type::AST_NOOP)
        }
    }
    
    pub fn visit_return(&mut self, node: &mut AST) -> AST {
        if let Some(ret) = node.return_value.as_mut() {
            let r = self.visit(ret);
            let mut return_node = AST::new(Ast_Type::AST_RETURN);
            return_node.return_value = Some(Box::new(r));
            return return_node;
        }

        AST::new(Ast_Type::AST_RETURN)
    }

    pub fn visit_reassign(&mut self, node: &mut AST) -> AST {
        let name = node.reassign_name.clone();
        let new_value = self.visit(&mut *node.reassign_value.clone().as_mut().unwrap());

        let mut scope_ref = node.scope.clone().unwrap();

        let original_value = scope_ref.borrow_mut().get_variable_definition(&name.clone().unwrap()).unwrap_or_else(|| panic!("Variable {} not defined", name.clone().unwrap()));

        let mut updated_value = original_value.clone();
        updated_value.variable_definition_value = Some(Box::new(new_value.clone()));

        scope_ref.borrow_mut().update_variable_definition(name.clone().unwrap(), updated_value);

        new_value
    }

    pub fn visit_increment(&mut self, node: &mut AST) -> AST {
        let name = node.reassign_name.clone().unwrap();
        let scope = node.scope.clone().unwrap();

        let original_value = scope.borrow_mut().get_variable_definition(&name).unwrap_or_else(|| panic!("Variable {} not defined", name));
    
        let mut val = original_value.variable_definition_value.clone().unwrap_or_else(|| panic!("Variable {} has no value", name));

        let mut eval = self.visit(&mut *val);

        match eval.ast_type {
            Ast_Type::AST_INT => {
                eval.int_value = Some(eval.int_value.unwrap() + 1);
                eval.int_init = Some(true);
            }
            Ast_Type::AST_FLOAT => {
                eval.float_value = Some(eval.float_value.unwrap() + 1.0);
                eval.float_init = Some(true);
            }
            _ => panic!("Cannot apply ++ to non-numeric type (only int/float supported)"),
        };

        let mut updated = original_value.clone();
        updated.variable_definition_value = Some(Box::new(eval.clone()));
        scope.borrow_mut().update_variable_definition(name.clone(), updated);

        eval
    }

    pub fn visit_decrement(&mut self, node: &mut AST) -> AST {
        let name = node.reassign_name.clone().unwrap();
        let scope = node.scope.clone().unwrap();

        let original_value = scope.borrow_mut().get_variable_definition(&name).unwrap_or_else(|| panic!("Variable {} not defined", name));
    
        let mut val = original_value.variable_definition_value.clone().unwrap_or_else(|| panic!("Variable {} has no value", name));

        let mut eval = self.visit(&mut *val);

        match eval.ast_type {
            Ast_Type::AST_INT => {
                eval.int_value = Some(eval.int_value.unwrap() - 1);
                eval.int_init = Some(true);
            }
            Ast_Type::AST_FLOAT => {
                eval.float_value = Some(eval.float_value.unwrap() - 1.0);
                eval.float_init = Some(true);
            }
            _ => panic!("Cannot apply -- to non-numeric type (only int/float supported)"),
        };

        let mut updated = original_value.clone();
        updated.variable_definition_value = Some(Box::new(eval.clone()));
        scope.borrow_mut().update_variable_definition(name.clone(), updated);

        eval
    }

    pub fn visit_while(&mut self, node: &mut AST) -> AST {
        loop {
            let condition = self.visit(&mut *node.while_condition.clone().unwrap());
                
            let is_true = match condition.ast_type {
                Ast_Type::AST_BOOL => condition.bool_value.unwrap_or(false),
                Ast_Type::AST_INT => condition.int_value.unwrap_or(0) != 0,
                Ast_Type::AST_FLOAT => condition.float_value.unwrap_or(0.0) != 0.0,
                _ => panic!("Invalid type for if statement"),
            };

            if !is_true {
                break;
            }

            self.visit(&mut *node.while_body.clone().unwrap());
        }
        AST::new(Ast_Type::AST_NOOP)
    }

    pub fn visit_for(&mut self, node: &mut AST) -> AST {
        let loop_scope = Rc::new(RefCell::new(Scope::new_with_parent(node.scope.as_ref().unwrap().clone())));
        
        let mut init = node.for_init.as_ref().unwrap().clone(); 
        self.set_scope_recursively(&mut init, loop_scope.clone()); 

        if init.ast_type == Ast_Type::AST_VARIABLE_DEF {
            let name = init.variable_definition_variable_name.clone().expect("Var def has no name");

            let mut value = *init.variable_definition_value.clone().expect("Missing value");
            self.set_scope_recursively(&mut value, loop_scope.clone());
            let mut eval = self.visit(&mut value);

            init.variable_definition_value = Some(Box::new(eval.clone()));

            let def = init.clone(); 

            loop_scope.borrow_mut().add_variable_definition(*def)
        }

        self.set_scope_recursively(&mut init, loop_scope.clone());

        while {
            let mut cond = *node.for_condition.clone().unwrap(); 
            self.set_scope_recursively(&mut cond, loop_scope.clone()); 
            let condition = self.visit(&mut cond);
            
            match condition.ast_type {
                Ast_Type::AST_BOOL => condition.bool_value.unwrap_or(false),
                Ast_Type::AST_INT => condition.int_value.unwrap_or(0) != 0,
                Ast_Type::AST_FLOAT => condition.float_value.unwrap_or(0.0) != 0.0,
                _ => panic!("For condition not a boolean"), 
            }
        } {
            let mut body = *node.for_body.clone().unwrap(); 
            self.set_scope_recursively(&mut body, loop_scope.clone()); 
            
            self.visit(&mut body);
            
            let mut increment = *node.for_increment.clone().unwrap();
            self.set_scope_recursively(&mut increment, loop_scope.clone()); 
            
            self.visit(&mut increment);
        }

        AST::new(Ast_Type::AST_NOOP)
    }
}