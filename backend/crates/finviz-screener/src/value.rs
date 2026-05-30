//! Runtime values and the `Row` abstraction the evaluator runs against.

/// A dynamically-typed field value pulled from a screener row.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    /// Missing / unavailable data.
    Null,
}

/// Anything the screener can evaluate filters against. Implementors expose
/// their columns by (already-normalized, lowercase) field name.
pub trait Row {
    fn field(&self, name: &str) -> Value;
}

/// Canonicalize a user-written field name to its internal key, resolving the
/// common aliases a trader would type (e.g. `marketcap`, `mktcap`).
pub fn canonical_field(name: &str) -> &'static str {
    match name
        .trim()
        .to_ascii_lowercase()
        .replace(['_', ' '], "")
        .as_str()
    {
        "symbol" | "ticker" => "symbol",
        "name" | "company" => "name",
        "sector" => "sector",
        "industry" => "industry",
        "exchange" => "exchange",
        "price" | "close" | "last" => "price",
        "change" | "chg" => "change",
        "changepct" | "changepercent" | "percentchange" | "chgpct" => "change_pct",
        "volume" | "vol" => "volume",
        "marketcap" | "mktcap" | "cap" => "market_cap",
        "pe" | "peratio" => "pe",
        "eps" => "eps",
        "dividendyield" | "dividend" | "yield" | "div" => "dividend_yield",
        "beta" => "beta",
        _ => "",
    }
}
