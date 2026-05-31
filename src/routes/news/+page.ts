import type { PageLoad } from './$types';
import { getNews } from '$lib/api';
import type { NewsItem } from '$lib/types';

const LIMIT = 40;

export const load: PageLoad = async ({ fetch }) => {
	try {
		const items = await getNews(undefined, LIMIT, fetch);
		return { items, error: null as string | null };
	} catch (e) {
		return {
			items: [] as NewsItem[],
			error: e instanceof Error ? e.message : 'Failed to load news'
		};
	}
};
