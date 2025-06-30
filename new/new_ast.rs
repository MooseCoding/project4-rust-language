// This is the new AST file after the old one needs to be replaced

#[derive(Debug, Clone)]
pub enum ASTNode {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),

    Variable(String),
    VariableDef {
        name: String,
        value: Box<ASTNode>,
    },

    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Box<ASTNode>, 
    },
    FunctionCall {
        name: String,
        args: Vec<ASTNode>,
    },

    If {
        condition: Box<ASTNode>,
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    While {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    For {
        init: Box<ASTNode>,
        condition: Box<ASTNode>,
        increment: Box<ASTNode>,
        body: Box<ASTNode>,
    },

    // Classes and objects
    ClassDef {
        name: String,
        methods: Vec<ASTNode>, // List of FunctionDefs
    },
    ClassCall {
        object: String,
        method: String,
        args: Vec<ASTNode>,
    },

    // Import support
    Import(String), // e.g., "math" -> import math

    // Arithmetic and binary ops
    BinaryOp {
        op: BinaryOperator,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },

    // Multiple expressions in a block
    Compound(Vec<ASTNode>),

    // Placeholder
    NoOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessEquals,
    GreaterEquals,
    And,
    Or,
}