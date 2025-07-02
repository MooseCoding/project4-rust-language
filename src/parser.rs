use std::rc::Rc;
use std::cell::RefCell;

use crate::ast::{AST, Ast_Type, Data_Type};
use crate::lexer::Lexer;
use crate::token::{Token, Types};
use crate::scope::{Scope, SharedScope};

#[derive(Debug)]
pub struct Parser<'a> {
    pub lexer: &'a mut Lexer,
    pub current_token: Token,
    pub prev_token: Option<Token>,
    pub scope: SharedScope,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer, scope: SharedScope) -> Self {
        let f_t = lexer.next_token();
        Parser {
            lexer, 
            current_token: f_t,
            prev_token: None,
            scope,
        }
    }

    pub fn eat(&mut self, t: Types) {
        if self.current_token.kind == t {
            self.prev_token = Some(self.current_token.clone());
            self.current_token = self.lexer.next_token();
        }
        else {
            panic!("Unexpected token parser {:?} got {:?}", t, self.current_token.kind);
        }
    }

    pub fn parse(&mut self) -> AST {
        self.parse_statements()
    }

    pub fn parse_statements(&mut self) -> AST {
        let mut comp = AST::new(Ast_Type::AST_COMPOUND);
        comp.scope = Some(self.scope.clone());
        comp.compound_value = Some(Vec::new());
        
        while self.current_token.kind != Types::TOKEN_EOF {
            let ast_state = self.parse_statement();

            if let Some(ref mut v) = comp.compound_value {
                v.push(ast_state);
            }

            if self.current_token.kind == Types::TOKEN_SEMI {
                self.eat(Types::TOKEN_SEMI);
            }
        }

        comp
    }

    pub fn parse_statement(&mut self) -> AST {
        match self.current_token.kind {
            Types::TOKEN_ID => self.parse_id(),
            _ => self.parse_expr(),
        }
    }

    pub fn parse_expr(&mut self) -> AST {
        self.parse_or()
    }

    pub fn parse_id(&mut self) -> AST {
        match self.current_token.value.as_str() {
            "int" | "str" | "bool" | "float" => self.parse_variable_definition(),
            "fun" => self.parse_function_definition(),
            "return" => self.parse_return(),
            "if" => self.parse_if(),
            "while" => self.parse_while(), 
            "for" => self.parse_for(), 
            _ => self.parse_variable(),
        }
    }

    pub fn parse_return(&mut self) -> AST {
        self.eat(Types::TOKEN_ID); 


        let mut node = AST::new(Ast_Type::AST_RETURN);

        if self.current_token.kind != Types::TOKEN_SEMI {
            let expr = self.parse_term();
            node.return_value = Some(Box::new(expr));
        }

        node
    }

    pub fn parse_integer(&mut self) -> AST {
        let t = self.current_token.clone();
        let mut ast = AST::new(Ast_Type::AST_INT);

        ast.int_value = Some(t.value.parse::<i32>().unwrap_or_else(|_| panic!("invalid integer {:#?}", t.value)));
        ast.int_init = Some(true);
        ast.data_type = Data_Type::INT;
        ast.scope = Some(self.scope.clone());
        self.eat(Types::TOKEN_INT);
        ast
    }

    pub fn parse_float(&mut self) -> AST {
        let t = self.current_token.clone();
        let mut ast = AST::new(Ast_Type::AST_FLOAT);

        ast.float_value = Some(t.value.parse::<f64>().unwrap_or_else(|_| panic!("invalid float {:#?}", t.value)));
        ast.float_init = Some(true);
        ast.data_type = Data_Type::FLOAT;
        ast.past_decimal = Some(t.value.split('.').nth(1).map_or(0, |s| s.len() as i32));
        ast.scope = Some(self.scope.clone());
        self.eat(Types::TOKEN_FLOAT);
        ast
    }

    pub fn parse_bool(&mut self) -> AST {
        let mut ast = AST::new(Ast_Type::AST_BOOL);
        ast.bool_value = Some(self.current_token.value == "true");
        ast.bool_init = Some(true);
        ast.scope = Some(self.scope.clone());
        self.eat(Types::TOKEN_BOOL);
        ast
    }

    pub fn parse_string(&mut self) -> AST {
        let mut ast = AST::new(Ast_Type::AST_STRING);
        ast.string_value = Some(self.current_token.value.clone());
        ast.scope = Some(self.scope.clone());
        self.eat(Types::TOKEN_STRING);
        ast
    }

    pub fn parse_term(&mut self) -> AST {
        self.parse_addition()
    }

    pub fn parse_addition(&mut self) -> AST {        
        let mut left = self.parse_multiplication();

        while matches!(self.current_token.kind, Types::TOKEN_ADD | Types::TOKEN_SUBTRACT) {
            let op = self.current_token.kind.clone();
            self.eat(op.clone());

            let right = self.parse_multiplication();

            left = self.combine_ast(left, op, right);
        }

        left
    }

    pub fn parse_multiplication(&mut self) -> AST {
        let mut left = self.parse_factor();

        while matches!(self.current_token.kind, Types::TOKEN_ASTERISK | Types::TOKEN_FSLASH) {
            let op = self.current_token.kind.clone();
            self.eat(op.clone());

            let right = self.parse_factor();

            left = self.combine_ast(left, op, right);
        }

        left
    }

    pub fn combine_ast(&mut self, left:AST, op:Types, right:AST) -> AST {
        let mut node = AST::new(Ast_Type::AST_BINARY);
        node.left=Some(Box::new(left));
        node.right = Some(Box::new(right));
        node.operator = Some(op);
        node.scope = Some(self.scope.clone());
        node
    }

    pub fn parse_factor(&mut self) -> AST {
        match self.current_token.kind {
            Types::TOKEN_FLOAT => self.parse_float(),
            Types::TOKEN_INT => self.parse_integer(),
            Types::TOKEN_ID => self.parse_variable(),
            Types::TOKEN_LPARENT => {
                self.eat(Types::TOKEN_LPARENT);
                let expr = self.parse_expr();
                self.eat(Types::TOKEN_RPARENT);
                expr
            }
            Types::TOKEN_SUBTRACT => {
                self.eat(Types::TOKEN_SUBTRACT);

                match self.current_token.kind {
                    Types::TOKEN_INT => {
                        let mut ast = self.parse_integer();
                        ast.int_value = Some(-ast.int_value.unwrap());
                        ast
                    }
                    Types::TOKEN_FLOAT => {
                        let mut ast = self.parse_float();
                        ast.float_value = Some(-ast.float_value.unwrap());
                        ast
                    }
                    Types::TOKEN_ID => {
                        let mut ast = self.parse_id();
                        let evaluated = self.eval_ast(ast.clone());

                        match evaluated.ast_type {
                            Ast_Type::AST_INT => {
                                let mut node = AST::new(Ast_Type::AST_INT);
                                node.int_value = Some(-evaluated.int_value.unwrap());
                                node.int_init = Some(true);
                                node.data_type = Data_Type::INT;
                                node.scope = Some(self.scope.clone());
                                node
                            }
                            Ast_Type::AST_FLOAT => {
                                let mut node = AST::new(Ast_Type::AST_FLOAT);
                                node.float_value = Some(-evaluated.float_value.unwrap());
                                node.float_init = Some(true);
                                node.data_type = Data_Type::FLOAT;
                                node.scope = Some(self.scope.clone());
                                node
                            }
                            _ => panic!("Cannot negate non-numeric type"),
                        }
                    }
                    _ => panic!("Unexpected token after unary minus: {:?}", self.current_token),
                }
            }
            Types::TOKEN_STRING => self.parse_string(),
            Types::TOKEN_BOOL => self.parse_bool(), 
            Types::TOKEN_NOT => {
                self.eat(Types::TOKEN_NOT);
                let expr = self.parse_factor();

                let mut node = AST::new(Ast_Type::AST_UNARY);
                node.operator = Some(Types::TOKEN_NOT);
                node.right = Some(Box::new(expr));
                node.scope = Some(self.scope.clone());
                node
            }
            _ => panic!("Unexpected token in factor {:?}", self.current_token.clone()),
        }
    }

    pub fn eval_ast(&mut self, ast: AST) -> AST {
        match ast.ast_type {
            Ast_Type::AST_INT | Ast_Type::AST_FLOAT => ast,
            Ast_Type::AST_VARIABLE_DEF => {
                if let Some(inner) = ast.variable_definition_value {
                    self.eval_ast(*inner)
                } else {
                    ast
                }
            }
            Ast_Type::AST_VARIABLE => {
                let name = ast.variable_name.clone().unwrap().trim_matches('"').to_string();

                let scope_rc = ast.scope.clone().expect("AST_VARIABLE missing scope");

                let maybe_val = {
                    let scope_ref = scope_rc.borrow();
                    scope_ref.get_variable_definition(&name)
                        .and_then(|def| def.variable_definition_value.clone())
                };

                if let Some(inner_ast) = maybe_val {
                    self.eval_ast(*inner_ast)
                } else {
                    ast
                }
            }
            _ => ast,
        }
    }

    pub fn parse_variable(&mut self) -> AST {
        let n = self.current_token.value.clone();

        self.eat(Types::TOKEN_ID);


        if self.current_token.kind == Types::TOKEN_LPARENT {
            return self.parse_function_call();
        }
        else if self.current_token.kind == Types::TOKEN_EQUALS {
            return self.parse_reassignment(n); 
        }
        else if self.current_token.kind == Types::TOKEN_INCREMENT {
            self.eat(Types::TOKEN_INCREMENT);
            let mut increment = AST::new(Ast_Type::AST_INCREMENT);
            increment.reassign_name = Some(n.clone());
            increment.scope = Some(self.scope.clone());
            
            return increment
        }
        else if self.current_token.kind == Types::TOKEN_DECREMENT {
            self.eat(Types::TOKEN_DECREMENT);
            let mut decrement = AST::new(Ast_Type::AST_DECREMENT);
            decrement.reassign_name = Some(n.clone());
            decrement.scope = Some(self.scope.clone());
            
            return decrement
        }

        let mut ast = AST::new(Ast_Type::AST_VARIABLE);
        ast.variable_name = Some(n);
        ast.scope = Some(self.scope.clone());
        ast
    }

    pub fn parse_variable_definition(&mut self) -> AST {
        let t = match self.current_token.value.as_str() {
            "str" => Data_Type::STR,
            "int" => Data_Type::INT,
            "bool" => Data_Type::BOOL,
            _ => Data_Type::FLOAT,
        };

        self.eat(Types::TOKEN_ID);
        let n = self.current_token.value.clone();
        self.eat(Types::TOKEN_ID);
        self.eat(Types::TOKEN_EQUALS);

        let val = self.parse_term();
        let evaluated = self.eval_ast(val.clone());

        let inferred_type = if evaluated.float_init.unwrap_or(false) {
            Data_Type::FLOAT
        } else if evaluated.string_value.is_some() {
            Data_Type::STR
        } else if evaluated.int_init.unwrap_or(false) {
            Data_Type::INT
        } else if evaluated.bool_init.unwrap_or(false) {
            Data_Type::BOOL
        } else {
            t.clone()
        };

        let mut val = val;
        val.data_type = inferred_type.clone();

        let mut def = AST::new(Ast_Type::AST_VARIABLE_DEF);
        def.variable_definition_variable_name = Some(n.clone());
        def.variable_definition_value = Some(Box::new(val.clone()));
        def.scope = Some(self.scope.clone());
        def.variable_type = Some(inferred_type.clone());

        if def.variable_type != Some(t.clone()) {
            panic!(
                "Variable {:?} is not the type {:?} that you assigned it, instead it's {:?}",
                def.variable_definition_variable_name.clone().unwrap(),
                t,
                def.variable_type.unwrap()
            );
        }

        //self.scope.borrow_mut().add_variable_definition(def.clone());
        def
    }

    pub fn parse_function_call(&mut self) -> AST {
        let mut ast = AST::new(Ast_Type::AST_FUNCTION_CALL);
        ast.function_call_name = Some(self.prev_token.as_ref().unwrap().value.clone());
        self.eat(Types::TOKEN_LPARENT);
        let mut args = Vec::new();

        if self.current_token.kind != Types::TOKEN_RPARENT {
            args.push(self.parse_term());

            while self.current_token.kind == Types::TOKEN_COMMA {
                self.eat(Types::TOKEN_COMMA);
                args.push(self.parse_term());
            }
        }

        self.eat(Types::TOKEN_RPARENT);
        ast.function_call_args = Some(args);
        ast.scope = Some(self.scope.clone());
        ast
    }

    pub fn parse_function_definition(&mut self) -> AST {
        let mut ast = AST::new(Ast_Type::AST_FUNCTION_DEF);

        self.eat(Types::TOKEN_ID);

        let n = self.current_token.value.clone();
        ast.function_definition_name = Some(n.clone());

        self.eat(Types::TOKEN_ID);
        self.eat(Types::TOKEN_LPARENT);

        let mut args = vec![];

        let func_scope = Rc::new(RefCell::new(crate::scope::Scope::new_with_parent(self.scope.clone())));

        while self.current_token.kind != Types::TOKEN_RPARENT {
            let t = match self.current_token.value.as_str() {
                "str" => Data_Type::STR,
                "int" => Data_Type::INT,
                "float" => Data_Type::FLOAT,
                "bool" => Data_Type::BOOL,
                _ => panic!("Incorrect type for function {}", n),
            };

            self.eat(Types::TOKEN_ID);

            let n2 = self.current_token.value.clone();
            
            self.eat(Types::TOKEN_ID);


            let mut arg = AST::new(Ast_Type::AST_VARIABLE_DEF);
            arg.variable_definition_variable_name = Some(n2);
            arg.variable_type = Some(t);
            arg.scope = Some(func_scope.clone()); 

            func_scope.borrow_mut().add_variable_definition(arg.clone());
            
            args.push(arg);

            if self.current_token.kind == Types::TOKEN_COMMA {
                self.eat(Types::TOKEN_COMMA);
            } else {
                break;
            }
        }

        self.eat(Types::TOKEN_RPARENT);
        self.eat(Types::TOKEN_LBRACK);

        let mut temp_parser = Parser {
            lexer: self.lexer,
            current_token: self.current_token.clone(),
            prev_token: self.prev_token.clone(),
           
            scope: func_scope.clone(),
        };

        ast.function_definition_body = Some(Box::new(temp_parser.parse_function_body()));

        self.current_token = temp_parser.current_token;
        self.prev_token = temp_parser.prev_token;

        self.eat(Types::TOKEN_RBRACK);

        ast.scope = Some(func_scope.clone());
        ast.function_definition_args = Some(args);

        self.scope.borrow_mut().add_function_definition(ast.clone());

        ast
    }

    pub fn parse_if(&mut self) -> AST {
        self.eat(Types::TOKEN_ID);
        self.eat(Types::TOKEN_LPARENT);

        let condition = self.parse_expr();

        self.eat(Types::TOKEN_RPARENT);
        self.eat(Types::TOKEN_LBRACK);

        let body = self.parse_function_body();

        self.eat(Types::TOKEN_RBRACK);

        let mut ast = AST::new(Ast_Type::AST_IF);
        ast.if_condition = Some(Box::new(condition.clone()));
        ast.if_body = Some(Box::new(body.clone()));

        if self.current_token.value == "else" {
            self.eat(Types::TOKEN_ID);
            self.eat(Types::TOKEN_LBRACK);

            let e = self.parse_function_body();

            self.eat(Types::TOKEN_RBRACK);

            ast.else_body = Some(Box::new(e.clone()));
        }

        ast.scope = Some(self.scope.clone());
        ast 
    }

    pub fn parse_function_body(&mut self) -> AST {
        let mut comp = AST::new(Ast_Type::AST_COMPOUND);
        comp.scope = Some(self.scope.clone());
        comp.compound_value = Some(Vec::new());
        
        while self.current_token.kind != Types::TOKEN_RBRACK {
            let ast_state = self.parse_statement();

            if let Some(ref mut v) = comp.compound_value {
                v.push(ast_state);
            }

            if self.current_token.kind == Types::TOKEN_SEMI {
                self.eat(Types::TOKEN_SEMI);
            }
        }

        comp
    }

    pub fn parse_comparison(&mut self) -> AST {
        let mut left = self.parse_term();


        while matches!(self.current_token.kind, 
            Types::TOKEN_GREATER_THAN |
            Types::TOKEN_LESS_THAN |
            Types::TOKEN_LEQ |
            Types::TOKEN_GEQ
        ) {
            let op = self.current_token.kind.clone();
            self.eat(op.clone());

            let right = self.parse_term();

            left = self.combine_ast(left, op, right)
        }

        left
    }

    pub fn parse_while(&mut self) -> AST {
        self.eat(Types::TOKEN_ID);
        self.eat(Types::TOKEN_LPARENT);

        let condition = self.parse_expr();

        self.eat(Types::TOKEN_RPARENT);
        self.eat(Types::TOKEN_LBRACK);

        let body = self.parse_function_body();

        self.eat(Types::TOKEN_RBRACK); 

        let mut ast = AST::new(Ast_Type::AST_WHILE);

        ast.while_condition = Some(Box::new(condition.clone()));
        ast.while_body = Some(Box::new(body.clone()));

        ast
    }

    pub fn parse_reassignment(&mut self, name: String) -> AST {
        self.eat(Types::TOKEN_EQUALS);
        let value = self.parse_term();

        let mut node = AST::new(Ast_Type::AST_REASSIGN);
        node.reassign_name = Some(name);
        node.reassign_value = Some(Box::new(value));
        node.scope = Some(self.scope.clone());
        node
    }

    pub fn parse_for(&mut self) -> AST {
        self.eat(Types::TOKEN_ID);
        self.eat(Types::TOKEN_LPARENT);

        let init = self.parse_statement();
        self.eat(Types::TOKEN_SEMI);

        let condition = self.parse_expr();
        self.eat(Types::TOKEN_SEMI);

        let increment = self.parse_statement();
        self.eat(Types::TOKEN_RPARENT);

        self.eat(Types::TOKEN_LBRACK);
        let body = self.parse_function_body();
        self.eat(Types::TOKEN_RBRACK);

        let mut ast = AST::new(Ast_Type::AST_FOR);
        let loop_scope = Rc::new(RefCell::new(Scope::new_with_parent(self.scope.clone())));

        ast.for_init = Some(Box::new(init));
        ast.for_condition = Some(Box::new(condition));
        ast.for_increment = Some(Box::new(increment));
        ast.for_body = Some(Box::new(body));
        ast.scope = Some(loop_scope.clone());

        self.set_scope_recursively(ast.for_init.as_mut().unwrap(), loop_scope.clone());
        self.set_scope_recursively(ast.for_condition.as_mut().unwrap(), loop_scope.clone());
        self.set_scope_recursively(ast.for_increment.as_mut().unwrap(), loop_scope.clone());
        self.set_scope_recursively(ast.for_body.as_mut().unwrap(), loop_scope.clone());

        ast
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
    }

    pub fn parse_or(&mut self) -> AST {
        let mut left = self.parse_and();

        while self.current_token.kind == Types::TOKEN_OR {
            let op = self.current_token.kind.clone();
            self.eat(op.clone());
            let right = self.parse_and();
            left = self.combine_ast(left, op, right);
        }

        left
    }

    pub fn parse_and(&mut self) -> AST {
        let mut left = self.parse_equality();

        while self.current_token.kind == Types::TOKEN_AND {
            let op = self.current_token.kind.clone();
            self.eat(op.clone());
            let right = self.parse_equality();
            left = self.combine_ast(left, op, right);
        }

        left
    }

    pub fn parse_equality(&mut self) -> AST {
        let mut left = self.parse_comparison();

        while matches!(self.current_token.kind, Types::TOKEN_EE | Types::TOKEN_NEQ) {
            let op = self.current_token.kind.clone();   
            self.eat(op.clone());
            let right = self.parse_comparison();
            left = self.combine_ast(left, op, right);
        }

        left
    }
}