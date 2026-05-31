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
        // identity / descriptive
        ("symbol", "string"),
        ("name", "string"),
        ("sector", "string"),
        ("industry", "string"),
        ("exchange", "string"),
        ("country", "string"),
        ("target_price", "number"),
        ("avg_volume", "number"),
        ("rel_volume", "number"),
        ("float_shares", "number"),
        ("recom", "number"),
        // market / quote
        ("price", "number"),
        ("change", "number"),
        ("change_pct", "number"),
        ("volume", "number"),
        // valuation
        ("market_cap", "number"),
        ("pe", "number"),
        ("forward_pe", "number"),
        ("peg", "number"),
        ("ps", "number"),
        ("pb", "number"),
        ("price_to_fcf", "number"),
        ("eps", "number"),
        ("dividend_yield", "number"),
        ("beta", "number"),
        // profitability
        ("roa", "number"),
        ("roe", "number"),
        ("roic", "number"),
        ("gross_margin", "number"),
        ("oper_margin", "number"),
        ("profit_margin", "number"),
        ("payout_ratio", "number"),
        // financial health
        ("current_ratio", "number"),
        ("quick_ratio", "number"),
        ("debt_equity", "number"),
        ("lt_debt_equity", "number"),
        // ownership
        ("insider_own", "number"),
        ("inst_own", "number"),
        ("short_float", "number"),
        ("short_ratio", "number"),
        // performance
        ("perf_week", "number"),
        ("perf_month", "number"),
        ("perf_quarter", "number"),
        ("perf_half", "number"),
        ("perf_year", "number"),
        ("perf_ytd", "number"),
        // technical
        ("volatility_w", "number"),
        ("volatility_m", "number"),
        ("rsi14", "number"),
        ("atr", "number"),
        ("sma20_rel", "number"),
        ("sma50_rel", "number"),
        ("sma200_rel", "number"),
        ("high_52w_pct", "number"),
        ("low_52w_pct", "number"),
    ]
}

impl Row for ScreenerRow {
    fn field(&self, name: &str) -> Value {
        match name {
            // identity / descriptive
            "symbol" => Value::Str(self.symbol.clone()),
            "name" => Value::Str(self.name.clone()),
            "sector" => Value::Str(self.sector.clone()),
            "industry" => Value::Str(self.industry.clone()),
            "exchange" => Value::Str(self.exchange.clone()),
            "country" => Value::Str(self.country.clone()),
            "target_price" => self.target_price.map_or(Value::Null, Value::Num),
            "avg_volume" => Value::Num(self.avg_volume),
            "rel_volume" => Value::Num(self.rel_volume),
            "float_shares" => Value::Num(self.float_shares),
            "recom" => self.recom.map_or(Value::Null, Value::Num),
            // market / quote
            "price" => Value::Num(self.price),
            "change" => Value::Num(self.change),
            "change_pct" => Value::Num(self.change_pct),
            "volume" => Value::Num(self.volume as f64),
            // valuation
            "market_cap" => Value::Num(self.market_cap),
            "pe" => self.pe.map_or(Value::Null, Value::Num),
            "forward_pe" => self.forward_pe.map_or(Value::Null, Value::Num),
            "peg" => self.peg.map_or(Value::Null, Value::Num),
            "ps" => self.ps.map_or(Value::Null, Value::Num),
            "pb" => self.pb.map_or(Value::Null, Value::Num),
            "price_to_fcf" => self.price_to_fcf.map_or(Value::Null, Value::Num),
            "eps" => self.eps.map_or(Value::Null, Value::Num),
            "dividend_yield" => self.dividend_yield.map_or(Value::Null, Value::Num),
            "beta" => self.beta.map_or(Value::Null, Value::Num),
            // profitability
            "roa" => self.roa.map_or(Value::Null, Value::Num),
            "roe" => self.roe.map_or(Value::Null, Value::Num),
            "roic" => self.roic.map_or(Value::Null, Value::Num),
            "gross_margin" => self.gross_margin.map_or(Value::Null, Value::Num),
            "oper_margin" => self.oper_margin.map_or(Value::Null, Value::Num),
            "profit_margin" => self.profit_margin.map_or(Value::Null, Value::Num),
            "payout_ratio" => self.payout_ratio.map_or(Value::Null, Value::Num),
            // financial health
            "current_ratio" => self.current_ratio.map_or(Value::Null, Value::Num),
            "quick_ratio" => self.quick_ratio.map_or(Value::Null, Value::Num),
            "debt_equity" => self.debt_equity.map_or(Value::Null, Value::Num),
            "lt_debt_equity" => self.lt_debt_equity.map_or(Value::Null, Value::Num),
            // ownership
            "insider_own" => self.insider_own.map_or(Value::Null, Value::Num),
            "inst_own" => self.inst_own.map_or(Value::Null, Value::Num),
            "short_float" => self.short_float.map_or(Value::Null, Value::Num),
            "short_ratio" => self.short_ratio.map_or(Value::Null, Value::Num),
            // performance
            "perf_week" => Value::Num(self.perf_week),
            "perf_month" => Value::Num(self.perf_month),
            "perf_quarter" => Value::Num(self.perf_quarter),
            "perf_half" => Value::Num(self.perf_half),
            "perf_year" => Value::Num(self.perf_year),
            "perf_ytd" => Value::Num(self.perf_ytd),
            // technical
            "volatility_w" => Value::Num(self.volatility_w),
            "volatility_m" => Value::Num(self.volatility_m),
            "rsi14" => Value::Num(self.rsi14),
            "atr" => Value::Num(self.atr),
            "sma20_rel" => Value::Num(self.sma20_rel),
            "sma50_rel" => Value::Num(self.sma50_rel),
            "sma200_rel" => Value::Num(self.sma200_rel),
            "high_52w_pct" => Value::Num(self.high_52w_pct),
            "low_52w_pct" => Value::Num(self.low_52w_pct),
            _ => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A neutral baseline `ScreenerRow`; tests override only the fields they
    /// exercise via `..base()`.
    fn base() -> ScreenerRow {
        ScreenerRow {
            symbol: "TEST".into(),
            name: "Test Co.".into(),
            sector: "Technology".into(),
            industry: "Software".into(),
            exchange: "NASDAQ".into(),
            country: "USA".into(),
            target_price: Some(110.0),
            avg_volume: 1_000_000.0,
            rel_volume: 1.0,
            float_shares: 9_000_000.0,
            recom: Some(2.0),
            price: 100.0,
            change: 1.0,
            change_pct: 1.0,
            volume: 1_000_000,
            market_cap: 1.0e11,
            pe: Some(25.0),
            forward_pe: Some(22.0),
            peg: Some(1.5),
            ps: Some(5.0),
            pb: Some(6.0),
            price_to_fcf: Some(30.0),
            eps: Some(4.0),
            dividend_yield: Some(1.0),
            beta: Some(1.1),
            roa: Some(10.0),
            roe: Some(15.0),
            roic: Some(12.0),
            gross_margin: Some(50.0),
            oper_margin: Some(25.0),
            profit_margin: Some(20.0),
            payout_ratio: Some(30.0),
            current_ratio: Some(2.0),
            quick_ratio: Some(1.5),
            debt_equity: Some(0.4),
            lt_debt_equity: Some(0.3),
            insider_own: Some(1.0),
            inst_own: Some(70.0),
            short_float: Some(2.0),
            short_ratio: Some(1.5),
            perf_week: 0.0,
            perf_month: 0.0,
            perf_quarter: 0.0,
            perf_half: 0.0,
            perf_year: 0.0,
            perf_ytd: 0.0,
            volatility_w: 2.0,
            volatility_m: 3.0,
            rsi14: 50.0,
            atr: 2.0,
            sma20_rel: 0.0,
            sma50_rel: 0.0,
            sma200_rel: 0.0,
            high_52w_pct: -5.0,
            low_52w_pct: 20.0,
        }
    }

    #[test]
    fn filters_on_new_fundamental_fields() {
        // `roe > 20 and pe < 30`: quality + reasonably valued.
        let expr = parse("roe > 20 and pe < 30").unwrap();
        let quality = ScreenerRow {
            roe: Some(35.0),
            pe: Some(18.0),
            ..base()
        };
        let weak_roe = ScreenerRow {
            roe: Some(10.0),
            pe: Some(18.0),
            ..base()
        };
        let pricey = ScreenerRow {
            roe: Some(35.0),
            pe: Some(45.0),
            ..base()
        };
        assert!(evaluate(&expr, &quality));
        assert!(!evaluate(&expr, &weak_roe));
        assert!(!evaluate(&expr, &pricey));
    }

    #[test]
    fn filters_on_technical_and_alias_fields() {
        // RSI alias resolves, and oversold/overbought thresholds work.
        let oversold = parse("rsi < 30").unwrap();
        let row_low = ScreenerRow {
            rsi14: 25.0,
            ..base()
        };
        let row_high = ScreenerRow {
            rsi14: 75.0,
            ..base()
        };
        assert!(evaluate(&oversold, &row_low));
        assert!(!evaluate(&oversold, &row_high));

        // Below-200d preset semantics + short-interest alias.
        let signal = parse("sma200 < 0 and shortfloat > 15").unwrap();
        let hit = ScreenerRow {
            sma200_rel: -8.0,
            short_float: Some(20.0),
            ..base()
        };
        let miss = ScreenerRow {
            sma200_rel: 5.0,
            short_float: Some(20.0),
            ..base()
        };
        assert!(evaluate(&signal, &hit));
        assert!(!evaluate(&signal, &miss));
    }

    #[test]
    fn missing_optional_field_excludes_row() {
        // A None metric yields Value::Null, which never satisfies a comparison.
        let expr = parse("peg < 2").unwrap();
        let present = ScreenerRow {
            peg: Some(1.0),
            ..base()
        };
        let missing = ScreenerRow {
            peg: None,
            ..base()
        };
        assert!(evaluate(&expr, &present));
        assert!(!evaluate(&expr, &missing));
    }

    #[test]
    fn known_fields_match_row_lookup() {
        // Every advertised numeric field must resolve to a non-Null value on a
        // fully-populated row (catches a field registered but not wired in Row).
        let row = base();
        for &(name, kind) in known_fields() {
            if kind == "number" {
                assert!(
                    matches!(row.field(name), Value::Num(_)),
                    "field `{name}` did not resolve to a number"
                );
            }
        }
    }
}
