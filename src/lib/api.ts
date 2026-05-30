/** Thin client for the same-origin `/api/v1` proxy. */
import type { ApiError, FieldInfo, Preset, ScreenRequest, ScreenResponse } from './types';

type FetchLike = typeof fetch;

async function json<T>(res: Response): Promise<T> {
	const body = await res.json().catch(() => null);
	if (!res.ok) {
		const err = body as ApiError | null;
		throw new Error(err?.message ?? `request failed (${res.status})`);
	}
	return body as T;
}

export async function runScreen(
	req: ScreenRequest,
	fetchFn: FetchLike = fetch
): Promise<ScreenResponse> {
	const res = await fetchFn('/api/v1/screener/run', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(req)
	});
	return json<ScreenResponse>(res);
}

export async function getPresets(fetchFn: FetchLike = fetch): Promise<Preset[]> {
	return json<Preset[]>(await fetchFn('/api/v1/screener/presets'));
}

export async function getFields(fetchFn: FetchLike = fetch): Promise<FieldInfo[]> {
	return json<FieldInfo[]>(await fetchFn('/api/v1/screener/fields'));
}
