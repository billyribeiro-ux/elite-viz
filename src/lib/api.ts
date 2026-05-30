/** Thin client for the same-origin `/api/v1` proxy. */
import type {
	ApiError,
	Bar,
	FieldInfo,
	Fundamentals,
	IndicatorSeries,
	Instrument,
	PortfolioSummary,
	Position,
	Preset,
	Quote,
	ScreenRequest,
	ScreenResponse,
	Watchlist
} from './types';

type FetchLike = typeof fetch;

async function json<T>(res: Response): Promise<T> {
	const body = await res.json().catch(() => null);
	if (!res.ok) {
		const err = body as ApiError | null;
		throw new Error(err?.message ?? `request failed (${res.status})`);
	}
	return body as T;
}

export async function runScreen(
	req: ScreenRequest,
	fetchFn: FetchLike = fetch
): Promise<ScreenResponse> {
	const res = await fetchFn('/api/v1/screener/run', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(req)
	});
	return json<ScreenResponse>(res);
}

export async function getPresets(fetchFn: FetchLike = fetch): Promise<Preset[]> {
	return json<Preset[]>(await fetchFn('/api/v1/screener/presets'));
}

export async function getFields(fetchFn: FetchLike = fetch): Promise<FieldInfo[]> {
	return json<FieldInfo[]>(await fetchFn('/api/v1/screener/fields'));
}

// ---- market data ----------------------------------------------------------

export async function getInstruments(fetchFn: FetchLike = fetch): Promise<Instrument[]> {
	return json<Instrument[]>(await fetchFn('/api/v1/market-data/instruments'));
}

export async function getQuote(symbol: string, fetchFn: FetchLike = fetch): Promise<Quote> {
	return json<Quote>(await fetchFn(`/api/v1/market-data/quote/${encodeURIComponent(symbol)}`));
}

export async function getFundamentals(
	symbol: string,
	fetchFn: FetchLike = fetch
): Promise<Fundamentals> {
	return json<Fundamentals>(
		await fetchFn(`/api/v1/market-data/fundamentals/${encodeURIComponent(symbol)}`)
	);
}

export async function getBars(
	symbol: string,
	opts: { interval?: string; limit?: number } = {},
	fetchFn: FetchLike = fetch
): Promise<Bar[]> {
	const params = new URLSearchParams();
	if (opts.interval) params.set('interval', opts.interval);
	if (opts.limit) params.set('limit', String(opts.limit));
	const qs = params.toString();
	return json<Bar[]>(
		await fetchFn(`/api/v1/market-data/bars/${encodeURIComponent(symbol)}${qs ? `?${qs}` : ''}`)
	);
}

export async function getIndicator(
	indicator: 'sma' | 'rsi',
	symbol: string,
	opts: { period?: number; limit?: number } = {},
	fetchFn: FetchLike = fetch
): Promise<IndicatorSeries> {
	const params = new URLSearchParams();
	if (opts.period) params.set('period', String(opts.period));
	if (opts.limit) params.set('limit', String(opts.limit));
	const qs = params.toString();
	return json<IndicatorSeries>(
		await fetchFn(
			`/api/v1/indicators/${indicator}/${encodeURIComponent(symbol)}${qs ? `?${qs}` : ''}`
		)
	);
}

// ---- watchlists -----------------------------------------------------------

export async function getWatchlists(fetchFn: FetchLike = fetch): Promise<Watchlist[]> {
	return json<Watchlist[]>(await fetchFn('/api/v1/watchlists'));
}

export async function createWatchlist(
	body: { name: string; symbols: string[] },
	fetchFn: FetchLike = fetch
): Promise<Watchlist> {
	return json<Watchlist>(
		await fetchFn('/api/v1/watchlists', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(body)
		})
	);
}

export async function deleteWatchlist(id: string, fetchFn: FetchLike = fetch): Promise<void> {
	const res = await fetchFn(`/api/v1/watchlists/${encodeURIComponent(id)}`, { method: 'DELETE' });
	if (!res.ok) throw new Error(`failed to delete watchlist (${res.status})`);
}

// ---- portfolio ------------------------------------------------------------

export async function getPortfolioSummary(fetchFn: FetchLike = fetch): Promise<PortfolioSummary> {
	return json<PortfolioSummary>(await fetchFn('/api/v1/portfolio/summary'));
}

export async function getPositions(fetchFn: FetchLike = fetch): Promise<Position[]> {
	return json<Position[]>(await fetchFn('/api/v1/portfolio/positions'));
}
