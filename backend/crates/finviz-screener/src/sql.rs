//! Compiles a parsed [`Expr`] into a parameterized SQL `WHERE` clause.
//!
//! Used by the PostgreSQL-backed path. Field names are mapped to column names
//! via [`canonical_field`], and unknown fields are rejected so we never
//! interpolate raw user input into SQL.

use crate::ast::{CmpOp, Expr, Literal};
use crate::error::ScreenerError;
use crate::value::canonical_field;

/// A bound parameter for the compiled query, in `$1, $2, ...` order.
#[derive(Debug, Clone, PartialEq)]
pub enum SqlParam {
    F64(f64),
    Str(String),
    Bool(bool),
}

/// The result of compiling a filter: a `WHERE`-clause body plus its params.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledSql {
    pub clause: String,
    pub params: Vec<SqlParam>,
}

/// Compile `expr` into SQL. `next` tracks the placeholder index across the
/// whole expression so parameters stay aligned.
pub fn compile(expr: &Expr) -> Result<CompiledSql, ScreenerError> {
    let mut params = Vec::new();
    let clause = build(expr, &mut params)?;
    Ok(CompiledSql { clause, params })
}

fn build(expr: &Expr, params: &mut Vec<SqlParam>) -> Result<String, ScreenerError> {
    match expr {
        Expr::And(a, b) => Ok(format!("({} AND {})", build(a, params)?, build(b, params)?)),
        Expr::Or(a, b) => Ok(format!("({} OR {})", build(a, params)?, build(b, params)?)),
        Expr::Not(inner) => Ok(format!("(NOT {})", build(inner, params)?)),
        Expr::Compare { field, op, value } => {
            let col = canonical_field(field);
            if col.is_empty() {
                return Err(ScreenerError::UnknownField(field.clone()));
            }
            Ok(build_compare(col, op, value, params))
        }
    }
}

fn build_compare(col: &str, op: &CmpOp, value: &Literal, params: &mut Vec<SqlParam>) -> String {
    let placeholder = |params: &mut Vec<SqlParam>, p: SqlParam| -> String {
        params.push(p);
        format!("${}", params.len())
    };

    match (op, value) {
        (CmpOp::Like, Literal::Str(s)) => {
            let ph = placeholder(params, SqlParam::Str(s.clone()));
            format!("{col} ILIKE '%' || {ph} || '%'")
        }
        (CmpOp::Eq, Literal::Str(s)) => {
            let ph = placeholder(params, SqlParam::Str(s.clone()));
            format!("lower({col}) = lower({ph})")
        }
        (CmpOp::Ne, Literal::Str(s)) => {
            let ph = placeholder(params, SqlParam::Str(s.clone()));
            format!("lower({col}) <> lower({ph})")
        }
        (_, lit) => {
            let param = match lit {
                Literal::Num(n) => SqlParam::F64(*n),
                Literal::Str(s) => SqlParam::Str(s.clone()),
                Literal::Bool(b) => SqlParam::Bool(*b),
            };
            let ph = placeholder(params, param);
            let sql_op = match op {
                CmpOp::Gt => ">",
                CmpOp::Lt => "<",
                CmpOp::Ge => ">=",
                CmpOp::Le => "<=",
                CmpOp::Eq => "=",
                CmpOp::Ne => "<>",
                CmpOp::Like => "=", // unreachable for non-string
            };
            format!("{col} {sql_op} {ph}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn compiles_numeric_and_string_filters() {
        let expr = parse("price > 100 and sector = \"Tech\"").unwrap();
        let out = compile(&expr).unwrap();
        assert_eq!(out.clause, "(price > $1 AND lower(sector) = lower($2))");
        assert_eq!(
            out.params,
            vec![SqlParam::F64(100.0), SqlParam::Str("Tech".into())]
        );
    }

    #[test]
    fn rejects_unknown_fields() {
        let expr = parse("bogus > 1").unwrap();
        assert_eq!(
            compile(&expr),
            Err(ScreenerError::UnknownField("bogus".into()))
        );
    }

    #[test]
    fn like_compiles_to_parameterized_ilike() {
        // The user-supplied substring must be bound as a parameter (never
        // interpolated) so wildcards/quotes cannot break out of the literal.
        let expr = parse("name ~ \"a%b'c\"").unwrap();
        let out = compile(&expr).unwrap();
        assert_eq!(out.clause, "name ILIKE '%' || $1 || '%'");
        assert_eq!(out.params, vec![SqlParam::Str("a%b'c".into())]);
    }

    #[test]
    fn placeholders_are_numbered_in_left_to_right_order() {
        // Aliases canonicalize to real columns, and every literal becomes a
        // positional placeholder in encounter order across nested OR/NOT.
        let expr = parse("not (mktcap > 1e12 or pe < 15) and ticker = \"AAPL\"").unwrap();
        let out = compile(&expr).unwrap();
        assert_eq!(
            out.clause,
            "((NOT (market_cap > $1 OR pe < $2)) AND lower(symbol) = lower($3))"
        );
        assert_eq!(
            out.params,
            vec![
                SqlParam::F64(1e12),
                SqlParam::F64(15.0),
                SqlParam::Str("AAPL".into()),
            ]
        );
    }

    #[test]
    fn not_equal_string_uses_case_insensitive_compare() {
        let expr = parse("sector <> \"Energy\"").unwrap();
        let out = compile(&expr).unwrap();
        assert_eq!(out.clause, "lower(sector) <> lower($1)");
        assert_eq!(out.params, vec![SqlParam::Str("Energy".into())]);
    }
}
