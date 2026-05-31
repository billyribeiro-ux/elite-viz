import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';
import {
	getAnalystRatings,
	getBBands,
	getBars,
	getFundamentals,
	getIndicator,
	getInsiderTrades,
	getInstruments,
	getNews,
	getOptionChain,
	getPatterns,
	getQuote
} from '$lib/api';
import type {
	AnalystRating,
	BBandPoint,
	IndicatorSeries,
	InsiderTrade,
	NewsItem,
	OptionChain,
	Pattern
} from '$lib/types';

const LIMIT = 120;

export const load: PageLoad = async ({ params, fetch }) => {
	const symbol = params.ticker.toUpperCase();

	// Core data: a failure here means the symbol genuinely has no data (404).
	let quote, fundamentals, bars, sma, instruments;
	try {
		[quote, fundamentals, bars, sma, instruments] = await Promise.all([
			getQuote(symbol, fetch),
			getFundamentals(symbol, fetch),
			getBars(symbol, { interval: '1d', limit: LIMIT }, fetch),
			getIndicator('sma', symbol, { period: 20, limit: LIMIT }, fetch),
			getInstruments(fetch)
		]);
	} catch {
		throw error(404, `No data for ${symbol}`);
	}

	const instrument = instruments.find((i) => i.symbol === symbol) ?? null;

	// Extended indicators are best-effort enhancements; null on failure.
	const [ema, rsi, bbands, news, insider, ratings, options, patterns] = await Promise.all([
		getIndicator('ema', symbol, { period: 20, limit: LIMIT }, fetch).catch(
			() => null as IndicatorSeries | null
		),
		getIndicator('rsi', symbol, { period: 14, limit: LIMIT }, fetch).catch(
			() => null as IndicatorSeries | null
		),
		getBBands(symbol, { period: 20, limit: LIMIT }, fetch).catch(() => [] as BBandPoint[]),
		getNews(symbol, 10, fetch).catch(() => [] as NewsItem[]),
		getInsiderTrades(symbol, fetch).catch(() => [] as InsiderTrade[]),
		getAnalystRatings(symbol, fetch).catch(() => [] as AnalystRating[]),
		getOptionChain(symbol, {}, fetch).catch(() => null as OptionChain | null),
		getPatterns(symbol, { limit: 20 }, fetch).catch(() => [] as Pattern[])
	]);

	return {
		symbol,
		quote,
		fundamentals,
		bars,
		sma,
		ema,
		rsi,
		bbands,
		instrument,
		news,
		insider,
		ratings,
		options,
		patterns
	};
};
