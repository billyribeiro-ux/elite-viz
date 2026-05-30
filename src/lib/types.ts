/** Mirrors the JSON returned by the Rust backend. */

export interface ScreenerRow {
	symbol: string;
	name: string;
	sector: string;
	industry: string;
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
}

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
