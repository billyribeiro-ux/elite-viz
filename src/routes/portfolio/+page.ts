import type { PageLoad } from './$types';
import { getPortfolioSummary } from '$lib/api';

export const load: PageLoad = async ({ fetch }) => {
	return { summary: await getPortfolioSummary(fetch) };
};
