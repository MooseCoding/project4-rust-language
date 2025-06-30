use std::rc::Rc;
use std::cell::RefCell;

use crate::ast::{AST, Ast_Type, Data_Type};
use crate::lexer::Lexer;
use crate::token::{Token, Types};
use crate::scope::SharedScope;

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
        match self.current_token.kind {
            Types::TOKEN_STRING => self.parse_string(),
            Types::TOKEN_ID => self.parse_id(),
            Types::TOKEN_FLOAT => self.parse_float(),
            Types::TOKEN_INT => self.parse_integer(),
            Types::TOKEN_BOOL => self.parse_bool(),
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
            _ => self.parse_term(),
        }
    }

    pub fn parse_id(&mut self) -> AST {
        match self.current_token.value.as_str() {
            "int" | "str" | "bool" | "float" => self.parse_variable_definition(),
            "fun" => self.parse_function_definition(),
            _ => self.parse_variable(),
        }
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
        ast.bool_value = Some(self.current_token.value != "false");
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

    fn parse_function_body(&mut self) -> AST {
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
}