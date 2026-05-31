import type { PageLoad } from './$types';
import { getEtfs } from '$lib/api';
import type { EtfProfile } from '$lib/types';

export const load: PageLoad = async ({ fetch }) => {
	try {
		const etfs = await getEtfs(fetch);
		return { etfs, error: null as string | null };
	} catch (e) {
		return { etfs: [] as EtfProfile[], error: e instanceof Error ? e.message : String(e) };
	}
};
