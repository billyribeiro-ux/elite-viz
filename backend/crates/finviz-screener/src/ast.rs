//! Abstract syntax tree for the screener filter DSL.

#[derive(Debug, Clone, PartialEq)]
pub enum CmpOp {
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
    Like,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Num(f64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// `<field> <op> <literal>`
    Compare {
        field: String,
        op: CmpOp,
        value: Literal,
    },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}
