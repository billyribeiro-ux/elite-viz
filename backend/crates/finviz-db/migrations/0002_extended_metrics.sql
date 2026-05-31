-- Wave 10: materialize the full FINVIZ-style screener-row surface.
--
-- The original schema (0001) carried only the ~14 core screener columns
-- (identity + quote + a handful of fundamentals). The `ScreenerRow` domain
-- type and the in-memory seed have since grown to the full ~56-field surface
-- (descriptive, valuation, profitability, financial health, ownership,
-- performance, technical). This migration adds a dedicated `screener_metrics`
-- table, keyed by `symbol` (1:1 with `instruments`), holding every extended
-- field so the Postgres path can reconstruct a complete `ScreenerRow`.
--
-- Kept in a separate table (rather than widening `fundamentals`) so the core
-- valuation columns stay cohesive and the extended/derived metric surface can
-- evolve independently. `screener_rows()` LEFT JOINs it, so partial data (an
-- instrument with no metrics row yet) degrades gracefully to sensible defaults.

CREATE TABLE IF NOT EXISTS screener_metrics (
    symbol         TEXT PRIMARY KEY REFERENCES instruments (symbol) ON DELETE CASCADE,

    -- descriptive
    country        TEXT NOT NULL DEFAULT '',
    target_price   DOUBLE PRECISION,
    avg_volume     DOUBLE PRECISION NOT NULL DEFAULT 0,
    rel_volume     DOUBLE PRECISION NOT NULL DEFAULT 0,
    float_shares   DOUBLE PRECISION NOT NULL DEFAULT 0,
    recom          DOUBLE PRECISION,

    -- valuation
    forward_pe     DOUBLE PRECISION,
    peg            DOUBLE PRECISION,
    ps             DOUBLE PRECISION,
    pb             DOUBLE PRECISION,
    price_to_fcf   DOUBLE PRECISION,

    -- profitability (percent)
    roa            DOUBLE PRECISION,
    roe            DOUBLE PRECISION,
    roic           DOUBLE PRECISION,
    gross_margin   DOUBLE PRECISION,
    oper_margin    DOUBLE PRECISION,
    profit_margin  DOUBLE PRECISION,
    payout_ratio   DOUBLE PRECISION,

    -- financial health
    current_ratio  DOUBLE PRECISION,
    quick_ratio    DOUBLE PRECISION,
    debt_equity    DOUBLE PRECISION,
    lt_debt_equity DOUBLE PRECISION,

    -- ownership (percent)
    insider_own    DOUBLE PRECISION,
    inst_own       DOUBLE PRECISION,
    short_float    DOUBLE PRECISION,
    short_ratio    DOUBLE PRECISION,

    -- performance (percent)
    perf_week      DOUBLE PRECISION NOT NULL DEFAULT 0,
    perf_month     DOUBLE PRECISION NOT NULL DEFAULT 0,
    perf_quarter   DOUBLE PRECISION NOT NULL DEFAULT 0,
    perf_half      DOUBLE PRECISION NOT NULL DEFAULT 0,
    perf_year      DOUBLE PRECISION NOT NULL DEFAULT 0,
    perf_ytd       DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- technical
    volatility_w   DOUBLE PRECISION NOT NULL DEFAULT 0,
    volatility_m   DOUBLE PRECISION NOT NULL DEFAULT 0,
    rsi14          DOUBLE PRECISION NOT NULL DEFAULT 0,
    atr            DOUBLE PRECISION NOT NULL DEFAULT 0,
    sma20_rel      DOUBLE PRECISION NOT NULL DEFAULT 0,
    sma50_rel      DOUBLE PRECISION NOT NULL DEFAULT 0,
    sma200_rel     DOUBLE PRECISION NOT NULL DEFAULT 0,
    high_52w_pct   DOUBLE PRECISION NOT NULL DEFAULT 0,
    low_52w_pct    DOUBLE PRECISION NOT NULL DEFAULT 0
);

-- Common filter columns get indexes; the screener frequently sorts/filters on
-- country (descriptive facet) and RSI (a popular technical screen).
CREATE INDEX IF NOT EXISTS idx_screener_metrics_country ON screener_metrics (country);
CREATE INDEX IF NOT EXISTS idx_screener_metrics_rsi14 ON screener_metrics (rsi14);
