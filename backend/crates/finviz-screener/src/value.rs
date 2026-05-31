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
        // identity / descriptive
        "symbol" | "ticker" => "symbol",
        "name" | "company" => "name",
        "sector" => "sector",
        "industry" => "industry",
        "exchange" => "exchange",
        "country" => "country",
        "targetprice" | "target" | "pricetarget" => "target_price",
        "avgvolume" | "averagevolume" | "avgvol" => "avg_volume",
        "relvolume" | "relativevolume" | "relvol" => "rel_volume",
        "floatshares" | "float" => "float_shares",
        "recom" | "recommendation" | "analystrecom" => "recom",
        // market / quote
        "price" | "close" | "last" => "price",
        "change" | "chg" => "change",
        "changepct" | "changepercent" | "percentchange" | "chgpct" => "change_pct",
        "volume" | "vol" => "volume",
        // valuation
        "marketcap" | "mktcap" | "cap" => "market_cap",
        "pe" | "peratio" | "p/e" => "pe",
        "forwardpe" | "fpe" | "forwardp/e" => "forward_pe",
        "peg" | "pegratio" => "peg",
        "ps" | "psratio" | "p/s" => "ps",
        "pb" | "pbratio" | "p/b" => "pb",
        "pricetofcf" | "pfcf" | "p/fcf" => "price_to_fcf",
        "eps" => "eps",
        "dividendyield" | "dividend" | "yield" | "div" => "dividend_yield",
        "beta" => "beta",
        // profitability
        "roa" => "roa",
        "roe" => "roe",
        "roic" => "roic",
        "grossmargin" | "grossm" => "gross_margin",
        "opermargin" | "operatingmargin" | "operm" => "oper_margin",
        "profitmargin" | "netmargin" | "profitm" => "profit_margin",
        "payoutratio" | "payout" => "payout_ratio",
        // financial health
        "currentratio" | "curratio" => "current_ratio",
        "quickratio" => "quick_ratio",
        "debtequity" | "debt/equity" | "de" | "d/e" => "debt_equity",
        "ltdebtequity" | "ltdebt/equity" | "ltde" => "lt_debt_equity",
        // ownership
        "insiderown" | "insiderownership" | "insider" => "insider_own",
        "instown" | "institutionalownership" | "institutionalown" | "inst" => "inst_own",
        "shortfloat" | "shortinterest" => "short_float",
        "shortratio" | "daystocover" => "short_ratio",
        // performance
        "perfweek" | "performanceweek" | "perfw" => "perf_week",
        "perfmonth" | "performancemonth" | "perfm" => "perf_month",
        "perfquarter" | "performancequarter" | "perfq" => "perf_quarter",
        "perfhalf" | "performancehalf" | "perfhalfyear" | "perf6m" => "perf_half",
        "perfyear" | "performanceyear" | "perfy" => "perf_year",
        "perfytd" | "performanceytd" | "ytd" => "perf_ytd",
        // technical
        "volatilityw" | "volatilityweek" | "volw" => "volatility_w",
        "volatilitym" | "volatilitymonth" | "volm" => "volatility_m",
        "rsi14" | "rsi" => "rsi14",
        "atr" => "atr",
        "sma20rel" | "sma20" => "sma20_rel",
        "sma50rel" | "sma50" => "sma50_rel",
        "sma200rel" | "sma200" => "sma200_rel",
        "high52wpct" | "high52w" | "52whigh" | "fromhigh" => "high_52w_pct",
        "low52wpct" | "low52w" | "52wlow" | "fromlow" => "low_52w_pct",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::canonical_field;

    #[test]
    fn resolves_extended_field_aliases() {
        // Case, spacing, underscores and slash forms all normalize to the
        // canonical internal key.
        assert_eq!(canonical_field("ForwardPE"), "forward_pe");
        assert_eq!(canonical_field("forward_pe"), "forward_pe");
        assert_eq!(canonical_field("P/E"), "pe");
        assert_eq!(canonical_field("P/S"), "ps");
        assert_eq!(canonical_field("p/b"), "pb");
        assert_eq!(canonical_field("ROE"), "roe");
        assert_eq!(canonical_field("roa"), "roa");
        assert_eq!(canonical_field("roic"), "roic");
        assert_eq!(canonical_field("Debt/Equity"), "debt_equity");
        assert_eq!(canonical_field("debtequity"), "debt_equity");
        assert_eq!(canonical_field("ShortFloat"), "short_float");
        assert_eq!(canonical_field("instOwn"), "inst_own");
        assert_eq!(canonical_field("institutional ownership"), "inst_own");
        assert_eq!(canonical_field("insiderOwn"), "insider_own");
        assert_eq!(canonical_field("PerformanceWeek"), "perf_week");
        assert_eq!(canonical_field("perf week"), "perf_week");
        assert_eq!(canonical_field("RSI"), "rsi14");
        assert_eq!(canonical_field("country"), "country");
        assert_eq!(canonical_field("targetPrice"), "target_price");
        assert_eq!(canonical_field("avgVolume"), "avg_volume");
        assert_eq!(canonical_field("relVolume"), "rel_volume");
        assert_eq!(canonical_field("recommendation"), "recom");
        assert_eq!(canonical_field("sma200"), "sma200_rel");
        assert_eq!(canonical_field("totally-unknown"), "");
    }
}
