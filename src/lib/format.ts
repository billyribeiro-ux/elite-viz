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
