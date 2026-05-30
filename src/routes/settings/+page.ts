import type { PageLoad } from './$types';
import { getProvider } from '$lib/api';

export const load: PageLoad = async ({ fetch }) => {
	return { provider: await getProvider(fetch) };
};
