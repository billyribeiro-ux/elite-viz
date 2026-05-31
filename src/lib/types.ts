/** Mirrors the JSON returned by the Rust backend. */

export interface ScreenerRow {
	symbol: string;
	name: string;
	sector: string;
	industry: string;
	country: string;
	exchange: string;
	price: number;
	change: number;
	change_pct: number;
	volume: number;
	market_cap: number;
	pe: number | null;
	eps: number | null;
	dividend_yield: number | null;
	beta: number | null;
	// Performance windows (percent).
	perf_week: number;
	perf_month: number;
	perf_quarter: number;
	perf_half: number;
	perf_year: number;
	perf_ytd: number;
	// Technicals.
	rsi14: number | null;
	// Expanded fundamentals (Option fields may be null).
	forward_pe: number | null;
	peg: number | null;
	ps: number | null;
	pb: number | null;
	roe: number | null;
	roa: number | null;
	debt_equity: number | null;
	profit_margin: number | null;
	short_float: number | null;
	inst_own: number | null;
}

/** Aggregated row from `/api/v1/groups`. */
export interface GroupRow {
	name: string;
	count: number;
	avg_change_pct: number;
	avg_pe: number;
	total_market_cap: number;
	avg_perf_week: number;
	avg_perf_month: number;
	avg_perf_year: number;
}

export type GroupBy = 'sector' | 'industry' | 'country';

export interface ScreenResponse {
	query: string;
	total: number;
	matched: number;
	rows: ScreenerRow[];
}

export interface Preset {
	id: string;
	label: string;
	query: string;
}

export interface FieldInfo {
	name: string;
	kind: 'number' | 'string';
}

export type SortOrder = 'asc' | 'desc';

export interface ScreenRequest {
	query: string;
	sort?: string;
	order?: SortOrder;
	limit?: number;
	offset?: number;
}

export interface ApiError {
	error: string;
	message: string;
}

export interface Quote {
	symbol: string;
	price: number;
	change: number;
	change_pct: number;
	volume: number;
	prev_close: number;
	day_high: number;
	day_low: number;
	ts: number;
}

export interface Fundamentals {
	symbol: string;
	market_cap: number;
	pe: number | null;
	eps: number | null;
	dividend_yield: number | null;
	beta: number | null;
	shares_outstanding: number;
}

export interface Instrument {
	symbol: string;
	name: string;
	sector: string;
	industry: string;
	exchange: string;
}

export interface Bar {
	ts: number;
	open: number;
	high: number;
	low: number;
	close: number;
	volume: number;
}

export interface IndicatorPoint {
	ts: number;
	value: number;
}

export interface IndicatorSeries {
	symbol: string;
	indicator: string;
	period: number;
	points: IndicatorPoint[];
}

export interface MacdPoint {
	ts: number;
	macd: number;
	signal: number;
	hist: number;
}

export interface BBandPoint {
	ts: number;
	middle: number;
	upper: number;
	lower: number;
}

export interface Watchlist {
	id: string;
	name: string;
	symbols: string[];
}

export interface Position {
	symbol: string;
	quantity: number;
	avg_price: number;
}

export interface PositionValue extends Position {
	last_price: number;
	market_value: number;
	cost_basis: number;
	unrealized_pnl: number;
	unrealized_pnl_pct: number;
}

export interface PortfolioSummary {
	positions: PositionValue[];
	market_value: number;
	cost_basis: number;
	unrealized_pnl: number;
	unrealized_pnl_pct: number;
}

export interface QuoteTick {
	symbol: string;
	price: number;
	change: number;
	change_pct: number;
	ts: number;
}

export type ProviderKind = 'mock' | 'finnhub' | 'polygon' | 'generic';

/** Provider settings as returned by the API (api key is write-only). */
export interface ProviderView {
	kind: ProviderKind;
	base_url: string | null;
	enabled: boolean;
	api_key_set: boolean;
	api_key_hint: string | null;
}

/** Provider settings as submitted from the UI. */
export interface ProviderConfigInput {
	kind: ProviderKind;
	api_key?: string;
	base_url?: string;
	enabled: boolean;
}

export interface ProviderTestResult {
	ok: boolean;
	message: string;
}

export interface User {
	id: string;
	email: string;
}

export interface AuthResponse {
	token: string;
	user: User;
}

export interface Alert {
	id: string;
	symbol: string;
	query: string;
	note: string;
}

export interface AlertStatus extends Alert {
	triggered: boolean;
}
