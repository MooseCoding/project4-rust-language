use crate::ast::{AST, Ast_Type, Data_Type};
use crate::lexer::Lexer;
use crate::token::{Token, Types};
use crate::scope::Scope;

#[derive(PartialEq, Debug)]
pub struct Parser<'a> {
    pub lexer: &'a mut Lexer,
    pub current_token: Token,
    pub prev_token: Option<Token>,
    pub scope: &'a mut Scope,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer, scope: &'a mut Scope) -> Self {
        let first_token = lexer.next_token(); 
        Parser {
            lexer, 
            current_token: first_token,
            prev_token:None, 
            scope: scope,
        }
    }

    pub fn eat(&mut self, token_type: Types) {
        if self.current_token.kind == token_type  {
            self.prev_token = Some(self.current_token.clone());
            self.current_token = self.lexer.next_token();
        }
        else {
            panic!(
                "Unexpected Parsed Token: {:?}, got {:?}",
                token_type, self.current_token.kind
            );
        }
    }

    pub fn parse(&mut self) -> AST {
        self.parse_statements()
    }

    pub fn parse_statements(&mut self) -> AST {
        let mut compound = AST::new(Ast_Type::AST_COMPOUND);
        compound.scope = Some(Box::new(self.scope.clone()));
        compound.compound_value = Some(Vec::new());

        let ast_statement = self.parse_statement();
        if let Some(ref mut v) =  compound.compound_value {
            v.push(ast_statement);
        }

        while(self.current_token.kind == Types::TOKEN_SEMI) {
            self.eat(Types::TOKEN_SEMI); 
            let next = self.parse_statement();
            if let Some(ref mut v) = compound.compound_value {
                v.push(next); 
            }
        }

        compound 
    }

    pub fn parse_statement(&mut self) -> AST {
        match self.current_token.kind {
            Types::TOKEN_ID => self.parse_id(),
            Types::TOKEN_ASTERISK => self.parse_factor(),
            Types::TOKEN_ADD => self.parse_term(),
            Types::TOKEN_FSLASH => self.parse_factor(),
            Types::TOKEN_SUBTRACT => self.parse_term(),
            _ => AST::new(Ast_Type::AST_NOOP),
        }
    }

    pub fn parse_expr(&mut self) -> AST {
        match self.current_token.kind {
            Types::TOKEN_STRING => self.parse_string(),
            Types::TOKEN_ID => self.parse_id(),
            Types::TOKEN_NUM => self.parse_num(),
            Types::TOKEN_BOOL => self.parse_bool(),
            Types::TOKEN_ADD => self.parse_term(),
            Types::TOKEN_ASTERISK => self.parse_factor(),
            Types::TOKEN_FSLASH => self.parse_factor(),
            _ => AST::new(Ast_Type::AST_NOOP),
        }
    }

    pub fn parse_function_definition(&mut self) -> AST {
        let mut ast = AST::new(Ast_Type::AST_FUNCTION_DEF);

        self.eat(Types::TOKEN_ID); // Keyword eating

        let name = self.current_token.value.clone();
        ast.function_definition_name = Some(name.clone());
        self.eat(Types::TOKEN_ID);

        self.eat(Types::TOKEN_LPARENT);

        let mut args = vec![];

        while self.current_token.kind != Types::TOKEN_RPARENT {
            let t = match self.current_token.value.as_str() {
                "str" => Data_Type::STR,
                "int" => Data_Type::INT,
                "float" => Data_Type::FLOAT,
                "bool" => Data_Type::BOOL,
                _ => panic!("Incorrect type for function {}", name),
            };

            self.eat(Types::TOKEN_ID);

            let n = self.current_token.value.clone();
            self.eat(Types::TOKEN_ID);

            let mut arg = AST::new(Ast_Type::AST_VARIABLE_DEF);
            arg.variable_definition_variable_name = Some(n);
            arg.variable_type = Some(t);

            args.push(arg);

            if self.current_token.kind == Types::TOKEN_COMMA {
                self.eat(Types::TOKEN_COMMA);
            }
            else {
                break; 
            }
        }

        for arg in &args {
            self.scope.add_variable_definition(arg.clone());
        }

        self.eat(Types::TOKEN_RPARENT);
        self.eat(Types::TOKEN_LBRACK);

        ast.function_definition_body = Some(Box::new(self.parse_statement()));
        self.eat(Types::TOKEN_RBRACK);

        ast.function_definition_args = Some(args);
        ast.scope = Some(Box::new(self.scope.clone()));
        ast
    }

    pub fn parse_function_call(&mut self) -> AST {
        let mut ast = AST::new(Ast_Type::AST_FUNCTION_CALL);
        ast.function_call_name = Some(self.prev_token.as_ref().unwrap().value.clone());

        self.eat(Types::TOKEN_LPARENT);

        let mut args = vec![self.parse_expr()];

        while self.current_token.kind == Types::TOKEN_COMMA {
            self.eat(Types::TOKEN_COMMA);
            args.push(self.parse_expr());
        }

        if matches!(self.current_token.kind, Types::TOKEN_ADD | Types::TOKEN_SUBTRACT) {
            args[0] = self.parse_expr();
            while self.current_token.kind == Types::TOKEN_ADD {
                args.push(self.parse_expr());
            }
        } else if matches!(self.current_token.kind, Types::TOKEN_ASTERISK | Types::TOKEN_FSLASH) {
            args[0] = self.parse_expr();
            while self.current_token.kind == Types::TOKEN_ASTERISK {
                args.push(self.parse_expr());
            }
        }

        self.eat(Types::TOKEN_RPARENT);
        ast.function_call_args = Some(args);
        ast.scope = Some(Box::new(self.scope.clone()));
        ast 
    }

    pub fn parse_variable_definition(&mut self) -> AST {
        let t = match self.current_token.value.as_str() {
            "str" => Data_Type::STR,
            "int" => Data_Type::INT,
            "bool" => Data_Type::BOOL,
            _ => Data_Type::FLOAT
        };

        self.eat(Types::TOKEN_ID);
        let n = self.current_token.value.clone();
        self.eat(Types::TOKEN_ID);
        self.eat(Types::TOKEN_EQUALS);

        let val = self.parse_expr();
        let mut def = AST::new(Ast_Type::AST_VARIABLE_DEF);

        def.variable_definition_variable_name = Some(n);
        def.variable_definition_value = Some(Box::new(val.clone()));
        def.scope = Some(Box::new(self.scope.clone()));

        def.variable_type = Some(if val.float_init.unwrap_or(false) {
            Data_Type::FLOAT 
        } else if val.string_value.is_some() {
            Data_Type::STR 
        }
        else if val.int_init.unwrap_or(false) {
            Data_Type::INT 
        }
        else if val.bool_init.unwrap_or(false) {
            Data_Type::BOOL
        }
        else {
            Data_Type::VOID
        }
        );

        if def.variable_type != Some(t) {
            panic!("Variabe {:#?} is not the type that you assigned it",
                def.variable_definition_variable_name.unwrap() 
            )
        }

        self.scope.add_variable_definition(def.clone());
        def 
    }

    pub fn parse_variable(&mut self) -> AST {
        let t = self.current_token.value.clone();

        self.eat(Types::TOKEN_ID);

        if self.current_token.kind == Types::TOKEN_LPARENT {
            return self.parse_function_call();
        }

        if self.current_token.kind == Types::TOKEN_ADD {
            return self.parse_term();
        }

        let mut ast = AST::new(Ast_Type::AST_VARIABLE);
        ast.variable_name = Some(t);
        ast.scope = Some(Box::new(self.scope.clone()));
        ast 
    }

    pub fn parse_id(&mut self) -> AST {
        match self.current_token.value.as_str() {
            "int" | "str" | "bool" | "float" => self.parse_variable_definition(),
            "fun" => self.parse_function_definition(),
            _ => self.parse_variable()
        }
    }

    pub fn parse_num(&mut self) -> AST {
        let mut ast = AST::new(Ast_Type::AST_NOOP);

        if self.current_token.value.contains('.') {
            ast.past_decimal = Some(self.current_token.value.split('.').nth(1).unwrap().len() as i32);
            ast.ast_type = Ast_Type::AST_FLOAT;
            ast.data_type = Data_Type::FLOAT; 
            ast.float_value = Some(self.current_token.value.parse().unwrap());
            ast.float_init = Some(true);
        }
        else {
            ast.ast_type = Ast_Type::AST_INT;
            ast.data_type = Data_Type::INT;
            ast.int_value = Some(self.current_token.value.parse().unwrap());
            ast.int_init = Some(true); 
        }

        self.eat(Types::TOKEN_NUM);

        match self.current_token.kind {
            Types::TOKEN_ADD => self.parse_term(),
            Types::TOKEN_ASTERISK | Types::TOKEN_FSLASH => self.parse_factor(),
            _ => {
                ast.scope = Some(Box::new(self.scope.clone())); 
                return ast; 
            },
        };

        ast 
    }

    pub fn parse_bool(&mut self) -> AST {
        let mut ast = AST::new(Ast_Type::AST_BOOL);
        ast.bool_value = Some(self.current_token.value != "false");
        ast.bool_init = Some(true);
        self.eat(Types::TOKEN_BOOL);

        if self.current_token.kind == Types::TOKEN_ADD {
            return self.parse_term();
        }

        ast.scope = Some(Box::new(self.scope.clone()));
        ast
    }

    pub fn parse_string(&mut self) -> AST {
        let mut ast = AST::new(Ast_Type::AST_STRING);
        ast.string_value = Some(self.current_token.value.clone());
        self.eat(Types::TOKEN_STRING);

        if self.current_token.kind == Types::TOKEN_ADD {
            return self.parse_term();
        }

        ast.scope = Some(Box::new(self.scope.clone()));
        ast 
    }

    pub fn parse_factor(&mut self) -> AST {
        let mut past_dec = 0;
        let mut lpd = 0;
        let mut return_value = 1.0;
        let mut can_break = false;

        let mut var = self.scope
            .get_variable_definition(&self.prev_token.as_ref().unwrap().value)
            .cloned()
            .unwrap_or_else(|| AST::new(Ast_Type::AST_NOOP));

        if self.current_token.kind == Types::TOKEN_ASTERISK {
            while (self.prev_token.as_ref().unwrap().value.parse::<f64>().is_ok()
                || var.int_init.unwrap_or(false)
                || var.float_init.unwrap_or(false)
            ) {
                let currentN = if let Ok(n) = self.prev_token.as_ref().unwrap().value.parse::<f64>() {
                    n 
                } 
                else if var.int_init.unwrap_or(false) {
                    var.int_value.unwrap_or(0) as f64
                }
                else {
                    var.float_value.unwrap_or(1.0)
                };

                if matches!(
                    self.current_token.kind,
                    Types::TOKEN_RPARENT | Types::TOKEN_SEMI
                ) {
                    can_break = true; 
                }

                if self.prev_token.as_ref().unwrap().value.contains('.') {
                    past_dec = self.prev_token.as_ref().unwrap().value.split('.').nth(1).unwrap().len() as i32;
                }

                if self.current_token.kind == Types::TOKEN_ASTERISK {
                    self.eat(Types::TOKEN_ASTERISK);
                    self.eat(self.current_token.kind.clone());
                }

                if let Some(nV) = self.scope.get_variable_definition(&self.prev_token.as_ref().unwrap().value) {
                    var = nV.clone();
                } 

                return_value *= currentN;

                if past_dec > lpd {
                    lpd = past_dec;
                }

                if can_break {
                    if let Some(_) = var.variable_definition_value {
                        var.float_init = Some(false);
                        var.int_init = Some(false); 
                    }
                    break; 
                }
            }

            let mut ast = if lpd != 0 {
                let mut ast = AST::new(Ast_Type::AST_FLOAT);
                ast.float_value = Some(return_value);
                ast.past_decimal = Some(lpd);
                ast.float_init = Some(true);
                ast 
            } else {
                let mut ast = AST::new(Ast_Type::AST_INT);
                ast.int_value = Some(return_value as i32);
                ast.int_init = Some(true);
                ast
            };

            return ast; 
        }

        if var.int_init.unwrap_or(false) {
            return_value = var.int_value.unwrap_or(0) as f64;
        }
        else if var.float_init.unwrap_or(false) {
            return_value = var.float_value.unwrap_or(1.0);
        }
        else if let Ok(n) = self.prev_token.as_ref().unwrap().value.parse::<f64>() {
            return_value = n; 
        }

        while self.prev_token.as_ref().unwrap().value.parse::<f64>().is_ok()
            || var.int_init.unwrap_or(false)
            || var.float_init.unwrap_or(false)
        {
            let currentN = if let Ok(n) = self.prev_token.as_ref().unwrap().value.parse::<f64>() {
                n
            } else if var.int_init.unwrap_or(false) {
                var.int_value.unwrap_or(0) as f64
            }
            else {
                var.float_value.unwrap_or(1.0)
            };

            if matches!(
                self.current_token.kind,
                Types::TOKEN_RPARENT | Types::TOKEN_SEMI
            ) {
                can_break = true;
            }

            if self.prev_token.as_ref().unwrap().value.contains('.') {
                past_dec = self.prev_token.as_ref().unwrap().value.split('.').nth(1).unwrap().len() as i32;
            }
            if self.current_token.kind == Types::TOKEN_FSLASH {
                self.eat(Types::TOKEN_FSLASH);
                self.eat(self.current_token.kind.clone());
            }

            if let Some(new_var) = self.scope.get_variable_definition(&self.prev_token.as_ref().unwrap().value) {
                var = new_var.clone();
            }

            return_value /= currentN;

            if past_dec > lpd {
                lpd = past_dec;
            }

            if can_break {
                if let Some(_) = var.variable_definition_value {
                    var.float_init = Some(false);
                    var.int_init = Some(false);
                }
                break;
            }
        }

        if return_value < 1.0 {
            lpd += 1;
        }

        let mut ast = if lpd != 0 {
            let mut ast = AST::new(Ast_Type::AST_FLOAT);
            ast.float_value = Some(return_value);
            ast.past_decimal = Some(lpd);
            ast.float_init = Some(true);
            ast
        } else {
            let mut ast = AST::new(Ast_Type::AST_INT);
            ast.int_value = Some(return_value as i32);
            ast.int_init = Some(true);
            ast
        };
        ast
    }
    

    pub fn parse_term(&mut self) -> AST {
        let mut return_value = String::new();
        let mut return_value_size = 0;
        let mut past_decimal = 0;
        let mut lpd = 0;
        let mut sum = 0.0;
        let mut r_type = 0;
        let mut can_break = false;

        let mut var = self.scope
            .get_variable_definition(&self.prev_token.as_ref().unwrap().value)
            .cloned()
            .unwrap_or_else(|| AST::new(Ast_Type::AST_NOOP));

        if self.current_token.kind == Types::TOKEN_SUBTRACT {
            sum = if let Some(val) = var.int_value {
                val as f64
            } else if let Some(val) = var.float_value {
                val
            } else {
                self.prev_token.as_ref().unwrap().value.parse().unwrap_or(0.0)
            };

            while self.prev_token.as_ref().unwrap().value.parse::<f64>().is_ok()
                || var.int_init.unwrap_or(false)
                || var.float_init.unwrap_or(false)
            {
                let current_n = if let Ok(n) = self.prev_token.as_ref().unwrap().value.parse::<f64>() {
                    n
                } else if var.int_init.unwrap_or(false) {
                    var.int_value.unwrap_or(0) as f64
                } else {
                    var.float_value.unwrap_or(1.0)
                };

                if matches!(
                    self.current_token.kind,
                    Types::TOKEN_RPARENT | Types::TOKEN_SEMI
                ) {
                    can_break = true;
                }

                if self.prev_token.as_ref().unwrap().value.contains('.') {
                    past_decimal = self.prev_token.as_ref().unwrap().value.split('.').nth(1).unwrap().len() as i32;
                }

                if self.current_token.kind == Types::TOKEN_SUBTRACT {
                    self.eat(Types::TOKEN_SUBTRACT);
                    self.eat(self.current_token.kind.clone());
                }

                if let Some(new_var) = self.scope.get_variable_definition(&self.prev_token.as_ref().unwrap().value) {
                    var = new_var.clone();
                }

                sum -= current_n;

                if past_decimal > lpd || var.past_decimal.unwrap_or(0) > lpd {
                    lpd = past_decimal;
                }

                if can_break {
                    var.int_init = Some(false);
                    var.float_init = Some(false);
                    break;
                }
            }

            return if lpd != 0 {
                let mut ast = AST::new(Ast_Type::AST_FLOAT);
                ast.float_value = Some(sum);
                ast.past_decimal = Some(lpd);
                ast.float_init = Some(true);
                ast
            } else {
                let mut ast = AST::new(Ast_Type::AST_INT);
                ast.int_value = Some(sum as i32);
                ast.int_init = Some(true);
                ast
            };
        }

        while self.prev_token.as_ref().unwrap().value.parse::<f64>().is_ok()
            || var.int_init.unwrap_or(false)
            || var.float_init.unwrap_or(false)
        {
            let current_n = if let Ok(n) = self.prev_token.as_ref().unwrap().value.parse::<f64>() {
                n
            } else if var.int_init.unwrap_or(false) {
                var.int_value.unwrap_or(0) as f64
            } else {
                var.float_value.unwrap_or(1.0)
            };

            if matches!(
                self.current_token.kind,
                Types::TOKEN_RPARENT | Types::TOKEN_SEMI
            ) {
                can_break = true;
            }

            if self.prev_token.as_ref().unwrap().value.contains('.') {
                past_decimal = self.prev_token.as_ref().unwrap().value.split('.').nth(1).unwrap().len() as i32;
            }

            if self.current_token.kind == Types::TOKEN_ADD {
                self.eat(Types::TOKEN_ADD);
                self.eat(self.current_token.kind.clone());
            }

            if let Some(new_var) = self.scope.get_variable_definition(&self.prev_token.as_ref().unwrap().value) {
                var = new_var.clone();
            }

            sum += current_n;

            if past_decimal > lpd || var.past_decimal.unwrap_or(0) > lpd {
                lpd = past_decimal;
            }

            let str_value = format!("{:.l$}", current_n, l = past_decimal as usize);
            return_value.push_str(&str_value);
            return_value_size += str_value.len();

            if can_break {
                var.int_init = Some(false);
                var.float_init = Some(false);
                break;
            }
        }

        while matches!(
            self.prev_token.as_ref().unwrap().kind,
            Types::TOKEN_STRING | Types::TOKEN_BOOL
        ) || var.string_value.is_some()
            || var.bool_init.unwrap_or(false)
        {
            let mut fragment = String::new();
            if let Some(new_var) = self.scope.get_variable_definition(&self.prev_token.as_ref().unwrap().value) {
                var = new_var.clone();
                if let Some(s) = &var.string_value {
                    fragment = s.clone();
                } else if let Some(b) = var.bool_value {
                    fragment = if b { "true" } else { "false" }.to_string();
                }
            } else {
                fragment = self.prev_token.as_ref().unwrap().value.clone();
            }

            return_value.push_str(&fragment);
            return_value_size += fragment.len();

            if matches!(
                self.current_token.kind,
                Types::TOKEN_RPARENT | Types::TOKEN_SEMI
            ) {
                break;
            }

            if self.current_token.kind == Types::TOKEN_ADD {
                self.eat(Types::TOKEN_ADD);
                self.eat(self.current_token.kind.clone());
            }

            if let Some(new_var) = self.scope.get_variable_definition(&self.prev_token.as_ref().unwrap().value) {
                var = new_var.clone();
            } else {
                var = AST::new(Ast_Type::AST_NOOP);
            }

            r_type = 1;
        }

        if r_type == 1 {
            let mut ast = AST::new(Ast_Type::AST_STRING);
            ast.string_value = Some(return_value);
            return ast;
        }

        if lpd != 0 {
            let mut ast = AST::new(Ast_Type::AST_FLOAT);
            ast.float_value = Some(sum);
            ast.past_decimal = Some(lpd);
            ast.float_init = Some(true);
            return ast;
        }

        let mut ast = AST::new(Ast_Type::AST_INT);
        ast.int_value = Some(sum as i32);
        ast.int_init = Some(true);
        ast
    }

}