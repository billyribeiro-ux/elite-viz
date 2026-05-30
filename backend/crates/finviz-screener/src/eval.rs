//! Evaluates a parsed [`Expr`] against in-memory [`Row`]s.

use crate::ast::{CmpOp, Expr, Literal};
use crate::value::{canonical_field, Row, Value};

/// Returns `true` if `row` satisfies `expr`.
pub fn evaluate<R: Row>(expr: &Expr, row: &R) -> bool {
    match expr {
        Expr::And(a, b) => evaluate(a, row) && evaluate(b, row),
        Expr::Or(a, b) => evaluate(a, row) || evaluate(b, row),
        Expr::Not(inner) => !evaluate(inner, row),
        Expr::Compare { field, op, value } => {
            let key = canonical_field(field);
            if key.is_empty() {
                return false; // unknown field never matches
            }
            compare(&row.field(key), op, value)
        }
    }
}

/// Keep only the rows satisfying `expr`.
pub fn filter<'a, R: Row>(expr: &Expr, rows: &'a [R]) -> Vec<&'a R> {
    rows.iter().filter(|r| evaluate(expr, *r)).collect()
}

fn compare(actual: &Value, op: &CmpOp, expected: &Literal) -> bool {
    match (actual, expected) {
        (Value::Num(a), Literal::Num(b)) => match op {
            CmpOp::Gt => a > b,
            CmpOp::Lt => a < b,
            CmpOp::Ge => a >= b,
            CmpOp::Le => a <= b,
            CmpOp::Eq => (a - b).abs() < f64::EPSILON,
            CmpOp::Ne => (a - b).abs() >= f64::EPSILON,
            CmpOp::Like => false,
        },
        (Value::Str(a), Literal::Str(b)) => {
            let (al, bl) = (a.to_ascii_lowercase(), b.to_ascii_lowercase());
            match op {
                CmpOp::Eq => al == bl,
                CmpOp::Ne => al != bl,
                CmpOp::Like => al.contains(&bl),
                CmpOp::Gt => al > bl,
                CmpOp::Lt => al < bl,
                CmpOp::Ge => al >= bl,
                CmpOp::Le => al <= bl,
            }
        }
        (Value::Bool(a), Literal::Bool(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            _ => false,
        },
        // Null or mismatched types never match (missing data is excluded).
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    struct TestRow {
        price: f64,
        pe: Value,
        sector: String,
    }

    impl Row for TestRow {
        fn field(&self, name: &str) -> Value {
            match name {
                "price" => Value::Num(self.price),
                "pe" => self.pe.clone(),
                "sector" => Value::Str(self.sector.clone()),
                _ => Value::Null,
            }
        }
    }

    fn row(price: f64, pe: Option<f64>, sector: &str) -> TestRow {
        TestRow {
            price,
            pe: pe.map(Value::Num).unwrap_or(Value::Null),
            sector: sector.into(),
        }
    }

    #[test]
    fn numeric_and_string_filters() {
        let expr = parse("price > 100 and sector = \"technology\"").unwrap();
        assert!(evaluate(&expr, &row(150.0, Some(20.0), "Technology")));
        assert!(!evaluate(&expr, &row(50.0, Some(20.0), "Technology")));
        assert!(!evaluate(&expr, &row(150.0, Some(20.0), "Energy")));
    }

    #[test]
    fn missing_data_is_excluded() {
        let expr = parse("pe < 30").unwrap();
        assert!(!evaluate(&expr, &row(10.0, None, "Tech")));
        assert!(evaluate(&expr, &row(10.0, Some(15.0), "Tech")));
    }

    #[test]
    fn substring_match_and_negation() {
        let contains = parse("sector ~ \"tech\"").unwrap();
        assert!(evaluate(&contains, &row(1.0, None, "Technology")));
        let negated = parse("not sector = \"energy\"").unwrap();
        assert!(evaluate(&negated, &row(1.0, None, "Technology")));
    }
}
