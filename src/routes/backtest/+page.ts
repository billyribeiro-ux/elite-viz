import type { PageLoad } from './$types';
import { getBacktestRules } from '$lib/api';
import type { RuleSpec } from '$lib/types';

export const load: PageLoad = async ({ fetch }) => {
	// Resilient: never throw — render an empty/error state instead so the
	// page is usable even while the backend is being rebuilt.
	try {
		const rules = await getBacktestRules(fetch);
		return { rules, rulesError: null as string | null };
	} catch (e) {
		const rulesError = e instanceof Error ? e.message : String(e);
		return { rules: [] as RuleSpec[], rulesError };
	}
};
