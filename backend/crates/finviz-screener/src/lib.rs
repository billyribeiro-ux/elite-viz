//! FINVIZ Elite+ screener engine.
//!
//! A small filter DSL with two backends:
//! - an in-memory [`eval`]uator over [`Row`]s (default runtime), and
//! - a parameterized [`sql`] compiler for the PostgreSQL path.

pub mod ast;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod sql;
pub mod value;

pub use ast::Expr;
pub use error::ScreenerError;
pub use eval::{evaluate, filter};
pub use parser::parse;
pub use sql::{compile, CompiledSql, SqlParam};
pub use value::{canonical_field, Row, Value};

use finviz_types::ScreenerRow;

/// The set of fields the screener understands, with their value kind. Exposed
/// over the API so the UI can offer autocomplete.
pub fn known_fields() -> &'static [(&'static str, &'static str)] {
    &[
        ("symbol", "string"),
        ("name", "string"),
        ("sector", "string"),
        ("industry", "string"),
        ("exchange", "string"),
        ("price", "number"),
        ("change", "number"),
        ("change_pct", "number"),
        ("volume", "number"),
        ("market_cap", "number"),
        ("pe", "number"),
        ("eps", "number"),
        ("dividend_yield", "number"),
        ("beta", "number"),
    ]
}

impl Row for ScreenerRow {
    fn field(&self, name: &str) -> Value {
        match name {
            "symbol" => Value::Str(self.symbol.clone()),
            "name" => Value::Str(self.name.clone()),
            "sector" => Value::Str(self.sector.clone()),
            "industry" => Value::Str(self.industry.clone()),
            "exchange" => Value::Str(self.exchange.clone()),
            "price" => Value::Num(self.price),
            "change" => Value::Num(self.change),
            "change_pct" => Value::Num(self.change_pct),
            "volume" => Value::Num(self.volume as f64),
            "market_cap" => Value::Num(self.market_cap),
            "pe" => self.pe.map_or(Value::Null, Value::Num),
            "eps" => self.eps.map_or(Value::Null, Value::Num),
            "dividend_yield" => self.dividend_yield.map_or(Value::Null, Value::Num),
            "beta" => self.beta.map_or(Value::Null, Value::Num),
            _ => Value::Null,
        }
    }
}
