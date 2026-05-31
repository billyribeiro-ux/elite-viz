/** Thin client for the same-origin `/api/v1` proxy. */
import { authHeaders } from './token';
import type {
	Alert,
	AlertStatus,
	ApiError,
	AuthResponse,
	BacktestRequest,
	BacktestResult,
	Bar,
	BBandPoint,
	FieldInfo,
	Fundamentals,
	GroupBy,
	GroupRow,
	IndicatorSeries,
	Instrument,
	MacdPoint,
	PortfolioSummary,
	Position,
	Preset,
	ProviderConfigInput,
	ProviderTestResult,
	ProviderView,
	Quote,
	RuleSpec,
	ScreenRequest,
	ScreenResponse,
	User,
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

// ---- groups ---------------------------------------------------------------

export async function getGroups(
	by: GroupBy = 'sector',
	fetchFn: FetchLike = fetch
): Promise<GroupRow[]> {
	return json<GroupRow[]>(
		await fetchFn(`/api/v1/groups?by=${encodeURIComponent(by)}`)
	);
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
	indicator: 'sma' | 'ema' | 'rsi',
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

/** Bollinger Bands — best-effort; backend returns `{ ..., points: BBandPoint[] }`. */
export async function getBBands(
	symbol: string,
	opts: { period?: number; limit?: number } = {},
	fetchFn: FetchLike = fetch
): Promise<BBandPoint[]> {
	const params = new URLSearchParams();
	if (opts.period) params.set('period', String(opts.period));
	if (opts.limit) params.set('limit', String(opts.limit));
	const qs = params.toString();
	const res = await fetchFn(
		`/api/v1/indicators/bbands/${encodeURIComponent(symbol)}${qs ? `?${qs}` : ''}`
	);
	const body = await json<{ points?: BBandPoint[] } | BBandPoint[]>(res);
	return Array.isArray(body) ? body : (body.points ?? []);
}

/** MACD — best-effort; backend returns `{ ..., points: MacdPoint[] }`. */
export async function getMacd(
	symbol: string,
	opts: { limit?: number } = {},
	fetchFn: FetchLike = fetch
): Promise<MacdPoint[]> {
	const params = new URLSearchParams();
	if (opts.limit) params.set('limit', String(opts.limit));
	const qs = params.toString();
	const res = await fetchFn(
		`/api/v1/indicators/macd/${encodeURIComponent(symbol)}${qs ? `?${qs}` : ''}`
	);
	const body = await json<{ points?: MacdPoint[] } | MacdPoint[]>(res);
	return Array.isArray(body) ? body : (body.points ?? []);
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

// ---- settings (data provider) ---------------------------------------------

export async function getProvider(fetchFn: FetchLike = fetch): Promise<ProviderView> {
	return json<ProviderView>(await fetchFn('/api/v1/settings/provider'));
}

export async function saveProvider(
	cfg: ProviderConfigInput,
	fetchFn: FetchLike = fetch
): Promise<ProviderView> {
	return json<ProviderView>(
		await fetchFn('/api/v1/settings/provider', {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(cfg)
		})
	);
}

export async function testProvider(
	cfg: ProviderConfigInput,
	fetchFn: FetchLike = fetch
): Promise<ProviderTestResult> {
	return json<ProviderTestResult>(
		await fetchFn('/api/v1/settings/provider/test', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(cfg)
		})
	);
}

// ---- auth -----------------------------------------------------------------

export async function register(
	creds: { email: string; password: string },
	fetchFn: FetchLike = fetch
): Promise<AuthResponse> {
	return json<AuthResponse>(
		await fetchFn('/api/v1/auth/register', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(creds)
		})
	);
}

export async function login(
	creds: { email: string; password: string },
	fetchFn: FetchLike = fetch
): Promise<AuthResponse> {
	return json<AuthResponse>(
		await fetchFn('/api/v1/auth/login', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(creds)
		})
	);
}

export async function getMe(fetchFn: FetchLike = fetch): Promise<User> {
	return json<User>(await fetchFn('/api/v1/auth/me', { headers: authHeaders() }));
}

// ---- alerts ---------------------------------------------------------------

export async function getAlerts(fetchFn: FetchLike = fetch): Promise<Alert[]> {
	return json<Alert[]>(await fetchFn('/api/v1/alerts'));
}

export async function checkAlerts(fetchFn: FetchLike = fetch): Promise<AlertStatus[]> {
	return json<AlertStatus[]>(await fetchFn('/api/v1/alerts/check'));
}

export async function createAlert(
	body: { symbol: string; query: string; note?: string },
	fetchFn: FetchLike = fetch
): Promise<Alert> {
	return json<Alert>(
		await fetchFn('/api/v1/alerts', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(body)
		})
	);
}

export async function deleteAlert(id: string, fetchFn: FetchLike = fetch): Promise<void> {
	const res = await fetchFn(`/api/v1/alerts/${encodeURIComponent(id)}`, { method: 'DELETE' });
	if (!res.ok) throw new Error(`failed to delete alert (${res.status})`);
}

// ---- backtesting ----------------------------------------------------------

/** Catalog of available entry-rule kinds and their params. */
export async function getBacktestRules(fetchFn: FetchLike = fetch): Promise<RuleSpec[]> {
	const body = await json<RuleSpec[] | { rules?: RuleSpec[] }>(
		await fetchFn('/api/v1/backtest/rules')
	);
	return Array.isArray(body) ? body : (body.rules ?? []);
}

export async function runBacktest(
	req: BacktestRequest,
	fetchFn: FetchLike = fetch
): Promise<BacktestResult> {
	return json<BacktestResult>(
		await fetchFn('/api/v1/backtest', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(req)
		})
	);
}
