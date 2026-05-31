import type { PageLoad } from './$types';
import { getFields, getPresets, getSavedScreens, runScreen } from '$lib/api';
import type { SavedScreen, SortOrder } from '$lib/types';

export const load: PageLoad = async ({ fetch, url }) => {
	// Deep-link support: seed the screen from the URL query string when present,
	// otherwise fall back to the default empty screen. Stays resilient to junk
	// params (e.g. an unexpected `order`) by normalising before use.
	const q = url.searchParams.get('q') ?? '';
	const sort = url.searchParams.get('sort') || 'market_cap';
	const orderParam = url.searchParams.get('order');
	const order: SortOrder = orderParam === 'asc' ? 'asc' : 'desc';

	const [presets, fields, initial, saved] = await Promise.all([
		getPresets(fetch),
		getFields(fetch),
		runScreen({ query: q, sort, order, limit: q ? 100 : 50 }, fetch),
		// Resilient: an empty saved list rather than failing the whole page.
		getSavedScreens(fetch).catch(() => [] as SavedScreen[])
	]);
	// Echo the resolved sort/order so the page can seed its controls from them.
	return { presets, fields, initial, saved, sort, order };
};
