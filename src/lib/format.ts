/** Presentation helpers for market data. */

const compact = new Intl.NumberFormat('en-US', {
	notation: 'compact',
	maximumFractionDigits: 2
});

/** $-prefixed price with 2 decimals. */
export function price(n: number): string {
	return `$${n.toFixed(2)}`;
}

/** Compact large numbers, e.g. 3.3T, 54.3M. */
export function compactNumber(n: number): string {
	return compact.format(n);
}

/** Compact market cap with a `$` prefix. */
export function marketCap(n: number): string {
	return `$${compact.format(n)}`;
}

/** Signed percentage with 2 decimals, e.g. +1.26%. */
export function percent(n: number): string {
	const sign = n > 0 ? '+' : '';
	return `${sign}${n.toFixed(2)}%`;
}

/** Optional numeric value, formatted to `digits` decimals or em-dash. */
export function optional(n: number | null, digits = 2): string {
	return n === null || n === undefined ? '—' : n.toFixed(digits);
}

/** Sign class for coloring deltas. */
export function trend(n: number): 'up' | 'down' | 'flat' {
	if (n > 0) return 'up';
	if (n < 0) return 'down';
	return 'flat';
}

const dateFmt = new Intl.DateTimeFormat('en-US', {
	month: 'short',
	day: 'numeric',
	year: 'numeric'
});

const dateTimeFmt = new Intl.DateTimeFormat('en-US', {
	month: 'short',
	day: 'numeric',
	hour: 'numeric',
	minute: '2-digit'
});

/** Epoch-seconds → short date (e.g. "May 31, 2026"); em-dash if missing. */
export function shortDate(tsSeconds: number | null | undefined): string {
	if (tsSeconds === null || tsSeconds === undefined || !Number.isFinite(tsSeconds)) return '—';
	return dateFmt.format(new Date(tsSeconds * 1000));
}

/** Nice labels for known chart-pattern kinds (keyed by lowercased kind). */
const patternLabels: Record<string, string> = {
	channelup: 'Channel Up',
	channeldown: 'Channel Down',
	triangleascending: 'Ascending Triangle',
	triangledescending: 'Descending Triangle',
	trianglesymmetric: 'Symmetric Triangle',
	wedge: 'Wedge',
	doubletop: 'Double Top',
	doublebottom: 'Double Bottom',
	headandshoulders: 'Head & Shoulders'
};

/**
 * Humanizes a pattern `kind` to a display label (case-insensitive match on the
 * known kinds), falling back to the raw kind string when unrecognized.
 */
export function patternLabel(kind: string): string {
	if (!kind) return '—';
	return patternLabels[kind.toLowerCase()] ?? kind;
}

/**
 * Epoch-seconds → relative phrase for recent times ("just now", "5m ago",
 * "3h ago", "2d ago"), falling back to a short date+time for older items.
 */
export function relativeDate(tsSeconds: number | null | undefined): string {
	if (tsSeconds === null || tsSeconds === undefined || !Number.isFinite(tsSeconds)) return '—';
	const diffMs = Date.now() - tsSeconds * 1000;
	const min = Math.floor(diffMs / 60000);
	if (min < 1) return 'just now';
	if (min < 60) return `${min}m ago`;
	const hr = Math.floor(min / 60);
	if (hr < 24) return `${hr}h ago`;
	const day = Math.floor(hr / 24);
	if (day < 7) return `${day}d ago`;
	return dateTimeFmt.format(new Date(tsSeconds * 1000));
}
