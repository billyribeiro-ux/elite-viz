import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';
import { getBars, getFundamentals, getIndicator, getInstruments, getQuote } from '$lib/api';

export const load: PageLoad = async ({ params, fetch }) => {
	const symbol = params.ticker.toUpperCase();
	try {
		const [quote, fundamentals, bars, sma, instruments] = await Promise.all([
			getQuote(symbol, fetch),
			getFundamentals(symbol, fetch),
			getBars(symbol, { interval: '1d', limit: 120 }, fetch),
			getIndicator('sma', symbol, { period: 20, limit: 120 }, fetch),
			getInstruments(fetch)
		]);
		const instrument = instruments.find((i) => i.symbol === symbol) ?? null;
		return { symbol, quote, fundamentals, bars, sma, instrument };
	} catch {
		throw error(404, `No data for ${symbol}`);
	}
};
