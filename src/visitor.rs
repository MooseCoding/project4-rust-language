use crate::ast::{AST, Ast_Type, Data_Type};
use crate::token::{Types};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::scope::{Scope, SharedScope};
use crate::lexer::Lexer;
use crate::parser::Parser; 

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
        // println!("Visiting {:#?}", node.ast_type); 
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
            Ast_Type::AST_UNARY => self.visit_unary(node), 
            Ast_Type::AST_ARRAY_ACCESS => self.visit_array_access(node),
            Ast_Type::AST_DOT => self.visit_dot(node),
            Ast_Type::AST_IMPORT => self.visit_import(node), 
            Ast_Type::AST_CLASS_INSTANCE => self.visit_class_creation(node),
            Ast_Type::AST_CLASS_ACCESS => self.visit_class_access(node), 
            _ => node.clone(),
        }
    }

    pub fn visit_class_creation(&mut self, node: &mut AST) -> AST {
        let name = node.class_name.as_ref().expect("Missing class name");

        let empty_vec = &vec![];

        let args = node.class_args.as_ref().unwrap_or(empty_vec).clone(); 
        let mut eval_args = args 
            .into_iter()
            .map(|mut arg| { return self.visit(&mut arg);})
            .collect::<Vec<_>>();

        let scope = node.scope.as_ref().expect("Class instanitation missing scope");

        let def = scope.borrow().get_class_definition(&node.class_name.as_ref().unwrap()).unwrap_or_else(|| panic!("Class {} not defined", name));

        let params = def.class_definition_args.as_ref().unwrap_or(empty_vec); 

        if eval_args.len() != params.len() {
            panic!(
                "Class instationation {} expected {} arguments, got {}",
                name,
                params.len(),
                eval_args.len()
            );
        }

        let class_scope = def.scope.as_ref().expect("Class def missing scope");
        let new_scope = Rc::new(RefCell::new(Scope::new_with_parent(class_scope.clone()))); 

        for (param, arg) in params.iter().zip(eval_args.iter()) {
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

            new_scope.borrow_mut().update_variable_definition(param.variable_definition_variable_name.as_ref().unwrap().clone(), var_def);
        }
        let mut body = def.class_definition_body.as_ref().expect("Missing body").clone();
        self.set_scope_recursively(&mut body, new_scope.clone());

        AST::new(Ast_Type::AST_NOOP)
    }

    pub fn visit_class_access(&mut self, node: &mut AST) -> AST {
        let instance = node.dot_left.as_mut().unwrap(); 
        let instance_name = instance.variable_name.as_ref().unwrap();
        
        let instance_def = node.scope.clone().unwrap().borrow()
            .get_variable_definition(instance_name)
            .unwrap_or_else(|| panic!("No instance {} found", instance_name));

        let class_name = instance_def.class_name.as_ref()
            .unwrap_or_else(|| panic!("Instance {} has no associated class", instance_name));

        let class_def = node.scope.clone().unwrap().borrow()
            .get_class_definition(class_name)
            .unwrap_or_else(|| panic!("No class definition found for {}", class_name));

        let class_scope = class_def.scope.as_ref()
            .unwrap_or_else(|| panic!("Instance {} has no scope", instance_name));

        match node.dot_right.as_ref().unwrap().ast_type {
            Ast_Type::AST_VARIABLE => {
                if node.reassign_value.is_some() {
                    let name = node.dot_right.as_ref().unwrap().variable_name.as_ref().unwrap();

                    let mut var_def = class_scope.borrow().get_variable_definition(name)
                        .unwrap_or_else(|| panic!("Undefined variable: {}", name));

                    if node.reassign_value.is_some() {
                        var_def.variable_definition_value = Some(Box::new(*node.reassign_value.as_ref().unwrap().clone()));
                    }

                    class_scope.borrow_mut().update_variable_definition(name.to_string(), var_def.clone()); 

                    let r = match var_def.ast_type {
                        Ast_Type::AST_VARIABLE_DEF => {
                            if let Some(val) = &var_def.variable_definition_value {
                                return self.visit(&mut *val.clone());
                            } else {
                                panic!("Variable '{}' has no value", name);
                            }
                        }
                        Ast_Type::AST_ARRAY_DEF => var_def.clone(),
                        Ast_Type::AST_INT | Ast_Type::AST_FLOAT | Ast_Type::AST_BOOL => var_def.clone(),
                        _ => panic!("Unknown variable type '{}'", name),
                    };

                    return r;
                }
                
                let name = node.dot_right.as_ref().unwrap().variable_name.as_ref().unwrap();

                if let Some(mut val) = class_scope.borrow().get_variable_definition(&name) {
                    return self.visit(&mut val.variable_definition_value.unwrap().clone()); 
                }

                panic!("Value not found for node.dot_right");
            }
            Ast_Type::AST_FUNCTION_CALL => {
                let a = node.dot_right.as_ref().unwrap(); 
                let name = a.function_call_name.as_ref().expect("Missing function name");

                let empty_vec = &vec![];

                let args_vec = a.function_call_args.as_ref().unwrap_or(empty_vec).clone();
                let mut evaluated_args = args_vec
                    .into_iter()
                    .map(|mut arg| self.visit(&mut arg))
                    .collect::<Vec<_>>();


                if let Some(f) = self.builtins.get(name) {
                    return f(&evaluated_args);
                }

                let def = class_scope
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

            _ => panic!("Invalid class access on dot_right"),
        }
    }

    pub fn visit_dot(&mut self, node: &mut AST) -> AST {
        let left = self.visit(node.dot_left.as_mut().unwrap());

        if left.ast_type == Ast_Type::AST_NOOP {
            panic!("Cannot access property on AST_NOOP (likely uninitialized)");
        }

        match left.ast_type {
            Ast_Type::AST_VARIABLE | Ast_Type::AST_IMPORT => {
                let name = left.variable_name.as_ref().unwrap();

                let scope = node.scope.clone().unwrap();

                self.set_scope_recursively(node, scope.clone());

                if let Some(import) = scope.borrow().get_import(name) {
                    let f = node.dot_right.as_ref().unwrap();

                    if f.ast_type == Ast_Type::AST_FUNCTION_CALL {
                        let f_name = f.function_call_name.as_ref().unwrap();
                        let args = f.function_call_args.clone().unwrap_or(vec![]);

                        if import.is_builtin.unwrap_or(false) {
                            return self.call_library_function(name, f_name, args, &node.scope);
                        }

                        let import_scope = import.scope.clone().expect("Imported AST missing scope");

                        let def = import_scope.borrow().get_function_definition(f_name)
                            .unwrap_or_else(|| panic!("Function '{}' not found in imported AST", f_name));

                        let mut f_call = AST::new(Ast_Type::AST_FUNCTION_CALL);
                        f_call.function_call_name = Some(f_name.clone());
                        f_call.function_call_args = Some(args);
                        f_call.scope = Some(import_scope.clone());

                        return self.visit_function_call(&mut f_call);
                    }

                    panic!("Cannot call dot access on non-function node");
                }

                panic!("Library `{}` not found in imports", name);
            }

            _ => panic!("Dot access not supported on {:#?}", left.ast_type),
        }
    }

    pub fn visit_import(&mut self, node: &mut AST) -> AST {
        let lib = node.variable_name.as_ref().unwrap().clone();
        let scope = node.scope.clone().expect("Import node missing scope"); 

        if node.is_builtin.unwrap_or(false) {
            let mut import_wrapper = AST::new(Ast_Type::AST_IMPORT);
            import_wrapper.variable_name = Some(lib.clone());
            import_wrapper.scope = Some(scope.clone());
            import_wrapper.is_builtin = Some(true);

            scope.borrow_mut().update_import(import_wrapper.clone());

            return AST::new(Ast_Type::AST_NOOP);
        }

        let path = format!("examples/lib/{}.steel", lib);
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Library `{}` not found at path {}", lib, path));

        let mut lexer = Lexer::new(&contents);
        let mut parser = Parser::new(&mut lexer, scope.clone());

        let mut lib_ast = parser.parse();
        lib_ast.scope = Some(scope.clone()); 

        let mut import_wrapper = AST::new(Ast_Type::AST_IMPORT);
        import_wrapper.variable_name = Some(lib.clone());
        import_wrapper.scope = Some(scope.clone());
        import_wrapper.imported_ast = Some(Box::new(lib_ast.clone()));
     
        scope.borrow_mut().update_import(import_wrapper.clone());

        node.scope = Some(scope.clone()); 

        AST::new(Ast_Type::AST_NOOP)
    }
    
    pub fn visit_variable(&mut self, node: &mut AST) -> AST {
        let name = node.variable_name.as_ref().unwrap();
        let scope = node.scope.as_ref().unwrap();

        if let Some(import) = scope.borrow().get_import(name) {
            let mut dummy = AST::new(Ast_Type::AST_IMPORT);
            dummy.variable_name = Some(name.clone());
            dummy.scope = Some(scope.clone());
            dummy.is_builtin = import.is_builtin;
            dummy.imported_ast = import.imported_ast.clone();

            return dummy;
        }

        let var_def = scope.borrow().get_variable_definition(name)
            .unwrap_or_else(|| panic!("Undefined variable: {}", name));

        match var_def.ast_type {
            Ast_Type::AST_VARIABLE_DEF => {
                if let Some(val) = &var_def.variable_definition_value {
                    return self.visit(&mut *val.clone());
                } else {
                    panic!("Variable '{}' has no value", name);
                }
            }
            Ast_Type::AST_ARRAY_DEF => var_def.clone(),
            Ast_Type::AST_INT | Ast_Type::AST_FLOAT | Ast_Type::AST_BOOL => var_def.clone(),
            _ => panic!("Unknown variable type '{}'", name),
        }
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

    pub fn visit_if(&mut self, node: &mut AST) -> AST {
        let condition = self.visit(node.if_condition.as_mut().expect("No if condition"));

        let is_true = match condition.ast_type {
            Ast_Type::AST_BOOL => condition.bool_value.unwrap_or(false),
            Ast_Type::AST_INT => condition.int_value.unwrap_or(0) != 0,
            Ast_Type::AST_FLOAT => condition.float_value.unwrap_or(0.0) != 0.0,
            _ => panic!("Invalid type for if statement"),
        };

        let result = if is_true {
            self.visit(node.if_body.as_mut().expect("Missing if body"))
        }
        else if let Some(e) = node.else_body.as_mut() {
            self.visit(e)
        }
        else {
            AST::new(Ast_Type::AST_NOOP)
        };

        result
    }
    
    pub fn call_library_function(&mut self,module: &str,function: &str,args: Vec<AST>,scope: &Option<SharedScope>) -> AST {
        match module {
            "math" => match function {
                "sqrt" => {
                    let num = self.visit(&mut args[0].clone());

                    let mut result = AST::new(Ast_Type::AST_FLOAT);
                    result.float_init = Some(true);
                    result.data_type = Data_Type::FLOAT;

                    result.float_value = Some(match num.ast_type {
                        Ast_Type::AST_FLOAT => num.float_value.unwrap().sqrt(),
                        Ast_Type::AST_INT => (num.int_value.unwrap() as f64).sqrt(),
                        _ => panic!("sqrt() requires int or float"),
                    });

                    return result;
                }
                "abs" => {
                    let num = self.visit(&mut args[0].clone());
                    let mut result = AST::new(Ast_Type::AST_FLOAT);
                    result.float_init = Some(true);
                    result.data_type = Data_Type::FLOAT;

                    result.float_value = Some(match num.ast_type {
                        Ast_Type::AST_FLOAT => num.float_value.unwrap().abs(),
                        Ast_Type::AST_INT => (num.int_value.unwrap() as f64).abs(),
                        _ => panic!("abs() requires int or float"),
                    });

                    return result;
                }
                "floor" => {
                    let num = self.visit(&mut args[0].clone());
                    let mut result = AST::new(Ast_Type::AST_FLOAT);
                    result.float_init = Some(true);
                    result.data_type = Data_Type::FLOAT;

                    result.float_value = Some(match num.ast_type {
                        Ast_Type::AST_FLOAT => num.float_value.unwrap().floor(),
                        Ast_Type::AST_INT => (num.int_value.unwrap() as f64).floor(),
                        _ => panic!("Floor requires an integer or a float"),
                    });
                    
                    return result; 
                }
                _ => panic!("Function `{}` not found in <math>", function),
            },
            _ => panic!("Built-in library `{}` not implemented", module),
        }
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
                else if stmt.ast_type == Ast_Type::AST_ARRAY_DEF {                    
                    self.set_scope_recursively(stmt, scope.clone());
                    scope.borrow_mut().add_variable_definition(stmt.clone()); 
                }
                else {
                    let result = self.visit(stmt);
                    if result.ast_type == Ast_Type::AST_RETURN {
                        return result;
                    }
                    else if result.ast_type == Ast_Type::AST_BREAK {
                        return AST::new(Ast_Type::AST_BREAK); 
                    }
                }
            }
        }

        AST::new(Ast_Type::AST_NOOP)
    }

    pub fn set_scope_recursively(&self, node: &mut AST, scope: Rc<RefCell<Scope>>) {
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

        if let Some(args) = node.function_definition_args.as_mut() {
            for arg in args.iter_mut() {
                self.set_scope_recursively(arg, scope.clone());
            }
        }

        if let Some(ret) = node.return_value.as_mut() {
            self.set_scope_recursively(ret, scope.clone());
        }

        if let Some(val) = node.variable_definition_value.as_mut() {
            self.set_scope_recursively(val, scope.clone());
        }

        if let Some(reassign) = node.reassign_value.as_mut() {
            self.set_scope_recursively(reassign, scope.clone());
        }

        if let Some(init) = node.for_init.as_mut() {
            self.set_scope_recursively(init, scope.clone());
        }
        if let Some(cond) = node.for_condition.as_mut() {
            self.set_scope_recursively(cond, scope.clone());
        }
        if let Some(incr) = node.for_increment.as_mut() {
            self.set_scope_recursively(incr, scope.clone());
        }
        if let Some(body) = node.for_body.as_mut() {
            self.set_scope_recursively(body, scope.clone());
        }

        if let Some(cond) = node.if_condition.as_mut() {
            self.set_scope_recursively(cond, scope.clone());
        }
        if let Some(if_body) = node.if_body.as_mut() {
            self.set_scope_recursively(if_body, scope.clone());
        }
        if let Some(else_body) = node.else_body.as_mut() {
            self.set_scope_recursively(else_body, scope.clone());
        }
        if let Some(cond) = node.while_condition.as_mut() {
            self.set_scope_recursively(cond, scope.clone());
        }
        if let Some(body) = node.while_body.as_mut() {
            self.set_scope_recursively(body, scope.clone());
        }

        if let Some(elements) = node.array_elements.as_mut() {
            for elem in elements.iter_mut() {
                self.set_scope_recursively(elem, scope.clone());
            }
        }
        if let Some(index) = node.array_index.as_mut() {
            self.set_scope_recursively(index, scope.clone());
        }
        if let Some(value) = node.array_assign_value.as_mut() {
            self.set_scope_recursively(value, scope.clone());
        }

        if let Some(left) = node.dot_left.as_mut() {
            self.set_scope_recursively(left, scope.clone());
        }
        if let Some(right) = node.dot_right.as_mut() {
            self.set_scope_recursively(right, scope.clone());
        }

        if let Some(args) = node.class_args.as_mut() {
            for arg in args.iter_mut() {
                self.set_scope_recursively(arg, scope.clone());
            }
        }

        if let Some(args) = node.class_definition_args.as_mut() {
            for arg in args.iter_mut() {
                self.set_scope_recursively(arg, scope.clone());
            }
        }

        if let Some(body) = node.class_definition_body.as_mut() {
            self.set_scope_recursively(body, scope.clone());
        }

        if let Some(imported) = node.imported_ast.as_mut() {
            self.set_scope_recursively(imported, scope.clone());
        }
    }

    pub fn visit_binary(&mut self, node: &mut AST) -> AST {
        let op = node.operator.as_ref().expect("Missing operator");

        let left_eval = self.visit(node.left.as_mut().expect("Missing left operand"));
        let right_eval = self.visit(node.right.as_mut().expect("Missing right operand"));

        // Handle boolean logic
        if matches!(op, Types::TOKEN_OR | Types::TOKEN_AND) {
            let l_bool = match left_eval.ast_type {
                Ast_Type::AST_BOOL => left_eval.bool_value.unwrap_or(false),
                Ast_Type::AST_INT => left_eval.int_value.unwrap_or(0) != 0,
                Ast_Type::AST_FLOAT => left_eval.float_value.unwrap_or(0.0) != 0.0,
                _ => panic!("Invalid left operand type for boolean operation"),
            };

            let r_bool = match right_eval.ast_type {
                Ast_Type::AST_BOOL => right_eval.bool_value.unwrap_or(false),
                Ast_Type::AST_INT => right_eval.int_value.unwrap_or(0) != 0,
                Ast_Type::AST_FLOAT => right_eval.float_value.unwrap_or(0.0) != 0.0,
                _ => panic!("Invalid right operand type for boolean operation"),
            };

            let result = match op {
                Types::TOKEN_OR => l_bool || r_bool,
                Types::TOKEN_AND => l_bool && r_bool,
                _ => unreachable!(),
            };

            let mut result_ast = AST::from_bool(result);
            result_ast.scope = Some(node.scope.clone().unwrap_or_else(|| Rc::new(RefCell::new(crate::scope::Scope::new()))));
            return result_ast;
        }

        if *op == Types::TOKEN_ADD {
            if let (Some(ls), Some(rs)) = (&left_eval.string_value, &right_eval.string_value) {
                let mut n = AST::new(Ast_Type::AST_STRING);
                n.string_value = Some(format!("{}{}", ls, rs));
                n.scope = Some(node.scope.clone().unwrap_or_else(|| Rc::new(RefCell::new(crate::scope::Scope::new()))));
                return n;
            }
        }

        let l_val = match left_eval.ast_type {
            Ast_Type::AST_FLOAT => left_eval.float_value.unwrap(),
            Ast_Type::AST_INT => left_eval.int_value.unwrap() as f64,
            _ => panic!("Invalid left operand type"),
        };

        let r_val = match right_eval.ast_type {
            Ast_Type::AST_FLOAT => right_eval.float_value.unwrap(),
            Ast_Type::AST_INT => right_eval.int_value.unwrap() as f64,
            _ => panic!("Invalid right operand type"),
        };

        let cmp_result = match op {
            Types::TOKEN_GREATER_THAN => Some(l_val > r_val),
            Types::TOKEN_LESS_THAN => Some(l_val < r_val),
            Types::TOKEN_LEQ => Some(l_val <= r_val),
            Types::TOKEN_GEQ => Some(l_val > r_val),
            Types::TOKEN_EE => Some((l_val - r_val).abs() < 1e-8),
            _ => None,
        };

        if let Some(bool_val) = cmp_result {
            let mut b = AST::from_bool(bool_val);
            b.scope = Some(node.scope.clone().unwrap_or_else(|| Rc::new(RefCell::new(crate::scope::Scope::new()))));
            return b;
        }

        let result = match op {
            Types::TOKEN_ADD => l_val + r_val,
            Types::TOKEN_SUBTRACT => l_val - r_val,
            Types::TOKEN_ASTERISK => l_val * r_val,
            Types::TOKEN_FSLASH => l_val / r_val,
            Types::TOKEN_PERCENT => l_val % r_val,
            Types::TOKEN_CARROT => l_val.powf(r_val), 
            _ => panic!("Unknown operator"),
        };

        let result_type = if left_eval.float_init.unwrap_or(false)
            || right_eval.float_init.unwrap_or(false)
        {
            Data_Type::FLOAT
        } else {
            Data_Type::INT
        };

        let mut result_node = match result_type {
            Data_Type::FLOAT => {
                let mut n = AST::new(Ast_Type::AST_FLOAT);
                n.float_value = Some(result);
                n.float_init = Some(true);
                n.data_type = Data_Type::FLOAT;
                n
            }
            Data_Type::INT => {
                let mut n = AST::new(Ast_Type::AST_INT);
                n.int_value = Some(result as i32);
                n.int_init = Some(true);
                n.data_type = Data_Type::INT;
                n
            }
            _ => panic!("Unsupported result type"),
        };

        result_node.scope = Some(node.scope.clone().unwrap_or_else(|| Rc::new(RefCell::new(crate::scope::Scope::new()))));
        result_node
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
        let cond_node = node.while_condition.as_mut().expect("Missing while condition");
        let body_node = node.while_body.as_mut().expect("Missing while body");

        loop {
            let condition = self.visit(cond_node);

            let is_true = match condition.ast_type {
                Ast_Type::AST_BOOL => condition.bool_value.unwrap_or(false),
                Ast_Type::AST_INT => condition.int_value.unwrap_or(0) != 0,
                Ast_Type::AST_FLOAT => condition.float_value.unwrap_or(0.0) != 0.0,
                _ => panic!("Invalid type for while condition"),
            };

            if !is_true {
                break;
            }

            let result = self.visit(body_node);

            if result.ast_type == Ast_Type::AST_RETURN {
                return result;  
            }

            if result.ast_type == Ast_Type::AST_BREAK {
                return result; 
            }
        }

        AST::new(Ast_Type::AST_NOOP)
    }

    pub fn visit_unary(&mut self, node: &mut AST) -> AST {
        let op = node.operator.as_ref().expect("unary operator missing");
        let mut operand = self.visit(node.right.as_mut().expect("unary operand missing"));

        match op {
            Types::TOKEN_SUBTRACT => match operand.ast_type {
                Ast_Type::AST_INT => {
                    operand.int_value = Some(-operand.int_value.unwrap());
                    operand
                }
                Ast_Type::AST_FLOAT => {
                    operand.float_value = Some(-operand.float_value.unwrap());
                    operand
                }
                _ => panic!("Unary minus only supports int and float"),
            },
            Types::TOKEN_NOT => match operand.ast_type {
                Ast_Type::AST_BOOL => {
                    operand.bool_value = Some(!operand.bool_value.unwrap());
                    operand
                }
                Ast_Type::AST_INT => {
                    operand.int_value = Some(if operand.int_value.unwrap() == 0 { 1 } else { 0 });
                    operand
                }
                Ast_Type::AST_FLOAT => {
                    operand.float_value = Some(if operand.float_value.unwrap() == 0.0 { 1.0 } else { 0.0 });
                    operand
                }
                _ => panic!("Unary not only supports bool, int, float"),
            },
            _ => panic!("Unknown unary operator {:?}", op),
        }
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
            
            let result = self.visit(&mut body);
            
            if result.ast_type == Ast_Type::AST_RETURN {
                break; 
            }
            
            let mut increment = *node.for_increment.clone().unwrap();
            self.set_scope_recursively(&mut increment, loop_scope.clone()); 
            
            self.visit(&mut increment);
        }

        AST::new(Ast_Type::AST_NOOP)
    }

    pub fn visit_array_access(&mut self, node: &mut AST) -> AST {
        let name = node.array_name.clone().unwrap();
        let index = self.visit(&mut *node.array_index.as_mut().unwrap());
        let scope = node.scope.clone().unwrap();
        let def = scope.borrow().get_variable_definition(&name).unwrap_or_else(|| panic!("Array {} not defined", name.clone()));
        let value = node.array_assign_value.as_ref().unwrap(); 

        let idx = match index.ast_type {
            Ast_Type::AST_INT => index.int_value.unwrap(),
            _ => panic!("Array index must be an int"),
        };

        let mut elements = def.array_elements.clone().unwrap();

        if idx < 0 || (idx as usize) >= elements.len() {
            panic!("index {} out of bounds for array {}", idx.clone(), name.clone());
        }

        elements[idx as usize] = *value.clone(); 

        let mut new_def = AST::new(Ast_Type::AST_ARRAY_DEF);
        new_def.array_name = Some(name.clone());
        new_def.array_elements = Some(elements.clone());
        
        scope.borrow_mut().update_variable_definition(name.clone(), new_def); 

        self.visit(&mut elements[idx as usize].clone()) 
    }
}