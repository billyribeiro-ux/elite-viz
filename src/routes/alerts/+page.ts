import type { PageLoad } from './$types';
import { checkAlerts } from '$lib/api';

export const load: PageLoad = async ({ fetch }) => {
	return { alerts: await checkAlerts(fetch) };
};
