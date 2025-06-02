// Widow Programming Language
// Abstract Syntax Tree (AST) module

use std::fmt;
use crate::error::Location;

/// Node represents a position in the source code
#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub line: usize,
    pub column: usize,
}

impl Node {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
    
    pub fn location(&self) -> Location {
        Location::new(self.line, self.column)
    }
}

/// Program is the root of the AST
#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// Statement types
#[derive(Debug)]
pub enum Statement {
    Expression(ExpressionStatement),
    Declaration(Declaration),
    Assignment(AssignmentStatement),
    Block(BlockStatement),
    If(IfStatement),
    For(ForStatement),
    Switch(SwitchStatement),
    Return(ReturnStatement),
    Break(Node),
    Continue(Node),
}

/// Expression statement
#[derive(Debug)]
pub struct ExpressionStatement {
    pub node: Node,
    pub expression: Expression,
}

/// Assignment statement
#[derive(Debug)]
pub struct AssignmentStatement {
    pub node: Node,
    pub target: Expression,
    pub value: Expression,
}

/// Block statement
#[derive(Debug)]
pub struct BlockStatement {
    pub node: Node,
    pub statements: Vec<Statement>,
}

/// If statement with optional else clause
#[derive(Debug)]
pub struct IfStatement {
    pub node: Node,
    pub condition: Expression,
    pub consequence: BlockStatement,
    pub alternative: Option<Box<Statement>>, // Either BlockStatement or another IfStatement
}

/// For loop statement
#[derive(Debug)]
pub enum ForStatement {
    // Simple loop with condition
    Condition {
        node: Node,
        condition: Expression,
        body: BlockStatement,
    },
    // Range-based loop
    Range {
        node: Node,
        variable: String,
        start: Expression,
        end: Expression,
        body: BlockStatement,
    },
    // Collection iteration loop
    Iteration {
        node: Node,
        variable: String,
        collection: Expression,
        body: BlockStatement,
    },
}

/// Switch statement
#[derive(Debug)]
pub struct SwitchStatement {
    pub node: Node,
    pub value: Expression,
    pub cases: Vec<CaseClause>,
    pub default: Option<BlockStatement>,
}

/// Case clause in a switch statement
#[derive(Debug)]
pub struct CaseClause {
    pub node: Node,
    pub values: Vec<Expression>,
    pub body: BlockStatement,
}

/// Return statement
#[derive(Debug)]
pub struct ReturnStatement {
    pub node: Node,
    pub values: Vec<Expression>,
}

/// Declaration types
#[derive(Debug)]
pub enum Declaration {
    Variable(VariableDeclaration),
    Function(FunctionDeclaration),
    Struct(StructDeclaration),
    Implementation(ImplementationDeclaration),
}

/// Variable declaration
#[derive(Debug)]
pub struct VariableDeclaration {
    pub node: Node,
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub value: Option<Expression>,
    pub is_const: bool,
}

/// Function declaration
#[derive(Debug)]
pub struct FunctionDeclaration {
    pub node: Node,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
    pub body: BlockStatement,
}

/// Function parameter
#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub node: Node,
}

/// Struct declaration
#[derive(Debug)]
pub struct StructDeclaration {
    pub node: Node,
    pub name: String,
    pub fields: Vec<StructField>,
}

/// Struct field
#[derive(Debug)]
pub struct StructField {
    pub node: Node,
    pub name: String,
    pub type_annotation: TypeAnnotation,
    pub default_value: Option<Expression>,
}

/// Implementation declaration
#[derive(Debug)]
pub struct ImplementationDeclaration {
    pub node: Node,
    pub struct_name: String,
    pub methods: Vec<FunctionDeclaration>,
}

/// Expression types
#[derive(Debug)]
pub enum Expression {
    Identifier(IdentifierExpression),
    Literal(LiteralExpression),
    Prefix(Box<PrefixExpression>),
    Infix(Box<InfixExpression>),
    Call(Box<CallExpression>),
    Index(Box<IndexExpression>),
    Dot(Box<DotExpression>),
    Array(ArrayExpression),
    HashMap(HashMapExpression),
    StructInit(StructInitExpression),
}

/// Identifier expression
#[derive(Debug)]
pub struct IdentifierExpression {
    pub node: Node,
    pub value: String,
}

/// Literal expression
#[derive(Debug)]
pub enum LiteralExpression {
    Int {
        node: Node,
        value: i64,
    },
    Float {
        node: Node,
        value: f64,
    },
    String {
        node: Node,
        value: String,
    },
    Char {
        node: Node,
        value: char,
    },
    Bool {
        node: Node,
        value: bool,
    },
    Nil {
        node: Node,
    },
}

/// Prefix expression
#[derive(Debug)]
pub struct PrefixExpression {
    pub node: Node,
    pub operator: PrefixOperator,
    pub right: Box<Expression>,
}

/// Prefix operators
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrefixOperator {
    Minus,
    Not,
}

impl fmt::Display for PrefixOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrefixOperator::Minus => write!(f, "-"),
            PrefixOperator::Not => write!(f, "!"),
        }
    }
}

/// Infix expression
#[derive(Debug)]
pub struct InfixExpression {
    pub node: Node,
    pub left: Box<Expression>,
    pub operator: InfixOperator,
    pub right: Box<Expression>,
}

/// Infix operators
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InfixOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

impl fmt::Display for InfixOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InfixOperator::Plus => write!(f, "+"),
            InfixOperator::Minus => write!(f, "-"),
            InfixOperator::Multiply => write!(f, "*"),
            InfixOperator::Divide => write!(f, "/"),
            InfixOperator::Modulo => write!(f, "%"),
            InfixOperator::Equal => write!(f, "=="),
            InfixOperator::NotEqual => write!(f, "!="),
            InfixOperator::LessThan => write!(f, "<"),
            InfixOperator::GreaterThan => write!(f, ">"),
            InfixOperator::LessEqual => write!(f, "<="),
            InfixOperator::GreaterEqual => write!(f, ">="),
            InfixOperator::And => write!(f, "&&"),
            InfixOperator::Or => write!(f, "||"),
        }
    }
}

/// Call expression
#[derive(Debug)]
pub struct CallExpression {
    pub node: Node,
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}

/// Index expression
#[derive(Debug)]
pub struct IndexExpression {
    pub node: Node,
    pub left: Box<Expression>,
    pub index: Box<Expression>,
}

/// Dot expression for member access
#[derive(Debug)]
pub struct DotExpression {
    pub node: Node,
    pub left: Box<Expression>,
    pub identifier: String,
}

/// Array expression
#[derive(Debug)]
pub struct ArrayExpression {
    pub node: Node,
    pub elements: Vec<Expression>,
}

/// HashMap expression
#[derive(Debug)]
pub struct HashMapExpression {
    pub node: Node,
    pub pairs: Vec<(Expression, Expression)>,
}

/// Struct initialization expression
#[derive(Debug)]
pub struct StructInitExpression {
    pub node: Node,
    pub struct_name: String,
    pub fields: Vec<(String, Expression)>,
}

/// Type annotation
#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    // Primitive types
    I8,
    I32,
    I64,
    I128,
    IArch,
    U8,
    U32,
    U64,
    U128,
    UArch,
    F32,
    F64,
    FArch,
    Bool,
    Char,
    String,
    // Compound types
    Array(Box<TypeAnnotation>),
    HashMap(Box<TypeAnnotation>, Box<TypeAnnotation>),
    // User-defined types
    Struct(String),
}

impl fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeAnnotation::I8 => write!(f, "i8"),
            TypeAnnotation::I32 => write!(f, "i32"),
            TypeAnnotation::I64 => write!(f, "i64"),
            TypeAnnotation::I128 => write!(f, "i128"),
            TypeAnnotation::IArch => write!(f, "iarch"),
            TypeAnnotation::U8 => write!(f, "u8"),
            TypeAnnotation::U32 => write!(f, "u32"),
            TypeAnnotation::U64 => write!(f, "u64"),
            TypeAnnotation::U128 => write!(f, "u128"),
            TypeAnnotation::UArch => write!(f, "uarch"),
            TypeAnnotation::F32 => write!(f, "f32"),
            TypeAnnotation::F64 => write!(f, "f64"),
            TypeAnnotation::FArch => write!(f, "farch"),
            TypeAnnotation::Bool => write!(f, "bool"),
            TypeAnnotation::Char => write!(f, "char"),
            TypeAnnotation::String => write!(f, "string"),
            TypeAnnotation::Array(elem_type) => write!(f, "[{}]", elem_type),
            TypeAnnotation::HashMap(key_type, val_type) => write!(f, "hm<{}, {}>", key_type, val_type),
            TypeAnnotation::Struct(name) => write!(f, "{}", name),
        }
    }
}