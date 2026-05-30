import type { PageLoad } from './$types';
import { getFields, getPresets, runScreen } from '$lib/api';

export const load: PageLoad = async ({ fetch }) => {
	const [presets, fields, initial] = await Promise.all([
		getPresets(fetch),
		getFields(fetch),
		runScreen({ query: '', sort: 'market_cap', order: 'desc', limit: 50 }, fetch)
	]);
	return { presets, fields, initial };
};
