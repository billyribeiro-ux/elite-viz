import type { PageLoad } from './$types';
import { getCrypto, getForex, getFutures } from '$lib/api';

/**
 * Loads all three market boards in parallel. Each board is resilient on its
 * own — a failing backend endpoint yields an empty board rather than failing
 * the whole page, so the UI can render a per-board empty state.
 */
export const load: PageLoad = async ({ fetch }) => {
	const [futures, forex, crypto] = await Promise.all([
		getFutures(fetch).catch(() => []),
		getForex(fetch).catch(() => []),
		getCrypto(fetch).catch(() => [])
	]);
	return { futures, forex, crypto };
};
