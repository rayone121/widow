#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    UnaryOp {
        op: String,
        expr: Box<Expr>,
    },
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    FuncCall {
        name: String,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    ArrayAccess {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    ArrayLiteral(Vec<Expr>),
    MapLiteral(Vec<(Expr, Expr)>),
    Grouped(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VariableDecl {
        name: String,
        expr: Option<Expr>,
    },
    ConstDecl {
        name: String,
        expr: Expr,
    },
    FuncDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    StructDecl {
        name: String,
        fields: Vec<(String, String)>, // field name + type
    },
    ImplDecl {
        type_name: String,
        methods: Vec<Stmt>, // Expect FuncDecls
    },
    Return(Expr),
    Assignment {
        target: Expr,
        value: Expr,
    },
    ExprStmt(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iter_expr: Expr,
        body: Vec<Stmt>,
    },
    Switch {
        expr: Expr,
        cases: Vec<(Expr, Vec<Stmt>)>,
        default: Option<Vec<Stmt>>,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
