import type { PageLoad } from './$types';
import { getFields, getPresets, getSavedScreens, runScreen } from '$lib/api';
import type { SavedScreen } from '$lib/types';

export const load: PageLoad = async ({ fetch }) => {
	const [presets, fields, initial, saved] = await Promise.all([
		getPresets(fetch),
		getFields(fetch),
		runScreen({ query: '', sort: 'market_cap', order: 'desc', limit: 50 }, fetch),
		// Resilient: an empty saved list rather than failing the whole page.
		getSavedScreens(fetch).catch(() => [] as SavedScreen[])
	]);
	return { presets, fields, initial, saved };
};
