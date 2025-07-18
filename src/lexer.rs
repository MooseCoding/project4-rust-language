use crate::token::{Token, Types};

#[derive(Clone, PartialEq, Debug)]
pub struct Lexer {
    current_char: Option<char>, 
    index: u64, 
    input: Vec<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let first = chars.get(0).copied();

        Lexer {
            current_char: first,
            index: 0, 
            input: chars,
        }
    }

    pub fn advance(&mut self) {
        self.index += 1;
        if self.index < self.input.len().try_into().unwrap() {
            self.current_char = Some(self.input[self.index as usize]);
        }
        else {
            self.current_char = None; 
        }
    }

    pub fn skip_space(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.advance();
            }
            else {
                break; 
            }
        }
    }

    pub fn collect_id(&mut self) -> Token {
        let mut result = String::new();
        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                result.push(c);
                self.advance();
            }
            else {
                break; 
            }
        }

        let kind = match result.as_str() {
            "true" | "false" => Types::TOKEN_BOOL,
            _ => Types::TOKEN_ID,
        };

        Token::new(kind, result)
    }

    pub fn collect_num(&mut self) -> Token {
        let mut result = String::new();
        while let Some(c) = self.current_char {
            if(c.is_ascii_digit() || c == '.') {
                result.push(c);
                self.advance();
            }
            else {
                break;
            }
        }

        if result.contains('.') {
            return Token::new(Types::TOKEN_FLOAT, result);
        }
        
        Token::new(Types::TOKEN_INT, result)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_space();

        match self.current_char {
            Some('"') => {
                self.advance() ;
                let mut string = String::new();

                while let Some(c) = self.current_char {
                    if c == '"' {
                        break;
                    }
                    string.push(c);
                    self.advance()
                }

                self.advance();
                Token::new(Types::TOKEN_STRING, string)
            }
            Some('%') => {
                self.advance();
                Token::new(Types::TOKEN_PERCENT, "%".to_string())
            }
            Some('/') => {
                self.advance();

                 if self.current_char == Some('*') {
                    self.advance(); 

                    loop {
                        match self.current_char {
                            Some('*') => {
                                self.advance(); 
                                if self.current_char == Some('/') {
                                    self.advance(); 
                                    break; 
                                }
                            }
                            Some(_) => {
                                self.advance();
                            }
                            None => break, 
                        }
                    }

                    return self.next_token(); 
                }

                Token::new(Types::TOKEN_FSLASH, "/".to_string())
            }

            Some('.') => {
                self.advance();

                Token::new(Types::TOKEN_DOT, ".".to_string())
            }
            Some('!') => {
                self.advance();
                
                if self.current_char == Some('=') {
                    self.advance();
                    return Token::new(Types::TOKEN_NEQ, "!=".to_string());
                }
                
                Token::new(Types::TOKEN_NOT, "!".to_string())
            }
            Some('&') => {
                self.advance();
                if self.current_char == Some('&') {
                    self.advance();
                    return Token::new(Types::TOKEN_AND, "&&".to_string());
                }
                
                Token::new(Types::TOKEN_EOF, "".to_string())
            }
            Some('|') => {
                self.advance();
                if self.current_char == Some('|') {
                    self.advance();
                    return Token::new(Types::TOKEN_OR, "||".to_string());
                }
                
                Token::new(Types::TOKEN_EOF, "".to_string())
            }
            Some('=') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance(); 
                    return Token::new(Types::TOKEN_EE, "==".to_string());
                }
                Token::new(Types::TOKEN_EQUALS, "=".to_string())
            }
            Some(';') => {
                self.advance();
                Token::new(Types::TOKEN_SEMI, ";".to_string())
            }
            Some('+') => {
                self.advance();

                if self.current_char == Some('+') {
                    self.advance(); 
                    return Token::new(Types::TOKEN_INCREMENT, "++".to_string());
                }

                Token::new(Types::TOKEN_ADD, "+".to_string())
            }
            Some(',') => {
                self.advance();
                Token::new(Types::TOKEN_COMMA, ",".to_string())
            }
            Some('-') => {
                self.advance();

                if self.current_char == Some('-') {
                    self.advance();
                    return Token::new(Types::TOKEN_DECREMENT, "--".to_string());
                }

                Token::new(Types::TOKEN_SUBTRACT, "-".to_string())
            }
            Some('*') => {
                self.advance();

                Token::new(Types::TOKEN_ASTERISK, "*".to_string())
            }
            Some('(') => {
                self.advance();
                Token::new(Types::TOKEN_LPARENT, "(".to_string())
            }
            Some(')') => {
                self.advance();
                Token::new(Types::TOKEN_RPARENT, ")".to_string())
            }
            Some('{') => {
                self.advance();
                Token::new(Types::TOKEN_LBRACK, "{".to_string())
            }
            Some('}') => {
                self.advance();
                if self.current_char == Some('=') {
                    return Token::new(Types::TOKEN_GEQ, ">=".to_string());
                }
                Token::new(Types::TOKEN_RBRACK, "}".to_string())
            }
            Some('<') => {
                self.advance();

                if self.current_char == Some('=') {
                    self.advance();
                    return Token::new(Types::TOKEN_LEQ, "<=".to_string());
                }

                Token::new(Types::TOKEN_LESS_THAN, "<".to_string())
            }
            Some('>') => {
                self.advance();
            
                if self.current_char == Some('=') {
                    self.advance();
                    return Token::new(Types::TOKEN_GEQ, ">=".to_string());
                }

                Token::new(Types::TOKEN_GREATER_THAN, ">".to_string())
            }
            Some('[') => {
                self.advance();
                Token::new(Types::TOKEN_LBOX, "[".to_string())
            }
            Some(']') => {
                self.advance(); 
                Token::new(Types::TOKEN_RBOX, "]".to_string())
            }
            Some('^') => {
                self.advance(); 
                Token::new(Types::TOKEN_CARROT, "^".to_string()) 
            }
            Some(c) if c.is_ascii_digit() => self.collect_num(),
            Some(c) if c.is_alphabetic() || c == '_'  => self.collect_id(),
            Some(_) => {
                let c = self.current_char.unwrap().to_string();
                self.advance();
                Token::new(Types::TOKEN_EOF, c)
            }
            none => Token::new(Types::TOKEN_EOF, "".to_string()),
        }
    }
}