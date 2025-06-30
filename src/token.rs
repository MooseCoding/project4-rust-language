#[derive(Clone, PartialEq, Debug)]
pub enum Types {
        TOKEN_ID, TOKEN_EQUALS, TOKEN_STRING, TOKEN_SEMI,
        TOKEN_LPARENT, TOKEN_RPARENT, TOKEN_COMMA, TOKEN_EOF,
        TOKEN_LBRACK, TOKEN_RBRACK, TOKEN_COLON, TOKEN_FSLASH,
        TOKEN_BSLASH, TOKEN_ASTERISK, TOKEN_INT, TOKEN_BOOL,
        TOKEN_ADD, TOKEN_SUBTRACT, TOKEN_FLOAT,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub kind: Types,
    pub value: String, 
}

impl Token {
    pub fn new(kind: Types, value: String) -> Self {
        Token{
            kind, 
            value,
        }
    }
}