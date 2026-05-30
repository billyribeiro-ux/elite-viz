use thiserror::Error;

/// Errors produced while lexing, parsing, or compiling a screener filter.
#[derive(Debug, Error, PartialEq)]
pub enum ScreenerError {
    #[error("lex error: {0}")]
    Lex(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unknown field: {0}")]
    UnknownField(String),
}
