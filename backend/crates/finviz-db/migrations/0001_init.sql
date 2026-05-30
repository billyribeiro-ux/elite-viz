-- FINVIZ Elite+ core schema (Phase 3).
-- Prices stored as double precision for direct f64 mapping; production may
-- migrate hot monetary columns to NUMERIC. Bars use a BRIN index on the
-- time column, which is ideal for naturally time-ordered append data.

CREATE TABLE IF NOT EXISTS users (
    id          BIGSERIAL PRIMARY KEY,
    email       TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS instruments (
    symbol      TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    sector      TEXT NOT NULL DEFAULT '',
    industry    TEXT NOT NULL DEFAULT '',
    exchange    TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_instruments_sector ON instruments (sector);

CREATE TABLE IF NOT EXISTS quotes (
    symbol      TEXT PRIMARY KEY REFERENCES instruments (symbol) ON DELETE CASCADE,
    price       DOUBLE PRECISION NOT NULL,
    change      DOUBLE PRECISION NOT NULL,
    change_pct  DOUBLE PRECISION NOT NULL,
    volume      BIGINT NOT NULL,
    prev_close  DOUBLE PRECISION NOT NULL,
    day_high    DOUBLE PRECISION NOT NULL,
    day_low     DOUBLE PRECISION NOT NULL,
    ts          BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS fundamentals (
    symbol             TEXT PRIMARY KEY REFERENCES instruments (symbol) ON DELETE CASCADE,
    market_cap         DOUBLE PRECISION NOT NULL,
    pe                 DOUBLE PRECISION,
    eps                DOUBLE PRECISION,
    dividend_yield     DOUBLE PRECISION,
    beta               DOUBLE PRECISION,
    shares_outstanding DOUBLE PRECISION NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_fundamentals_market_cap ON fundamentals (market_cap);

CREATE TABLE IF NOT EXISTS bars (
    symbol      TEXT NOT NULL REFERENCES instruments (symbol) ON DELETE CASCADE,
    interval    TEXT NOT NULL,
    ts          BIGINT NOT NULL,
    open        DOUBLE PRECISION NOT NULL,
    high        DOUBLE PRECISION NOT NULL,
    low         DOUBLE PRECISION NOT NULL,
    close       DOUBLE PRECISION NOT NULL,
    volume      BIGINT NOT NULL,
    PRIMARY KEY (symbol, interval, ts)
);

CREATE INDEX IF NOT EXISTS brin_bars_ts ON bars USING brin (ts);

CREATE TABLE IF NOT EXISTS watchlists (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT REFERENCES users (id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS watchlist_items (
    watchlist_id BIGINT NOT NULL REFERENCES watchlists (id) ON DELETE CASCADE,
    symbol       TEXT NOT NULL REFERENCES instruments (symbol) ON DELETE CASCADE,
    position     INT NOT NULL DEFAULT 0,
    PRIMARY KEY (watchlist_id, symbol)
);

CREATE TABLE IF NOT EXISTS positions (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT REFERENCES users (id) ON DELETE CASCADE,
    symbol      TEXT NOT NULL REFERENCES instruments (symbol) ON DELETE CASCADE,
    quantity    DOUBLE PRECISION NOT NULL,
    avg_price   DOUBLE PRECISION NOT NULL,
    UNIQUE (user_id, symbol)
);
