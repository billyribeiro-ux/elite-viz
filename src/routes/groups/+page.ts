import type { PageLoad } from './$types';
import { getGroups } from '$lib/api';

export const load: PageLoad = async ({ fetch }) => {
	try {
		const groups = await getGroups('sector', fetch);
		return { groups, by: 'sector' as const, error: null as string | null };
	} catch (e) {
		return {
			groups: [],
			by: 'sector' as const,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
