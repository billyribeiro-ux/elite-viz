import type { PageLoad } from './$types';
import { runScreen } from '$lib/api';

export const load: PageLoad = async ({ fetch }) => {
	try {
		const res = await runScreen(
			{ query: '', sort: 'market_cap', order: 'desc', limit: 500 },
			fetch
		);
		return { rows: res.rows, error: null as string | null };
	} catch (e) {
		return { rows: [], error: e instanceof Error ? e.message : String(e) };
	}
};
