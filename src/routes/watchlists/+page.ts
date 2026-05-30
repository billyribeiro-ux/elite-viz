import type { PageLoad } from './$types';
import { getWatchlists } from '$lib/api';

export const load: PageLoad = async ({ fetch }) => {
	return { watchlists: await getWatchlists(fetch) };
};
