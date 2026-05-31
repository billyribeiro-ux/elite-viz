import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';
import { getEtf } from '$lib/api';

export const load: PageLoad = async ({ params, fetch }) => {
	const symbol = params.symbol.toUpperCase();
	try {
		const etf = await getEtf(symbol, fetch);
		return { etf };
	} catch {
		throw error(404, `No ETF data for ${symbol}`);
	}
};
