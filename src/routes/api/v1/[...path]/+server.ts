/**
 * Same-origin proxy to the Rust backend. The browser talks to SvelteKit at
 * `/api/v1/*`; this forwards to `BACKEND_URL` (default http://localhost:8080),
 * keeping the backend address server-side and sidestepping CORS in production.
 */
import { env } from '$env/dynamic/private';
import type { RequestHandler } from './$types';

const BACKEND = env.BACKEND_URL ?? 'http://localhost:8080';

async function forward(
	method: string,
	path: string,
	search: string,
	body: string | undefined
): Promise<Response> {
	const target = `${BACKEND}/api/v1/${path}${search}`;
	let res: Response;
	try {
		res = await fetch(target, {
			method,
			headers: { 'content-type': 'application/json' },
			body: body && body.length > 0 ? body : undefined
		});
	} catch {
		return new Response(
			JSON.stringify({ error: 'bad_gateway', message: `backend unreachable at ${BACKEND}` }),
			{ status: 502, headers: { 'content-type': 'application/json' } }
		);
	}
	return new Response(res.body, {
		status: res.status,
		headers: { 'content-type': res.headers.get('content-type') ?? 'application/json' }
	});
}

const withBody: RequestHandler = ({ params, url, request }) =>
	request.text().then((body) => forward(request.method, params.path, url.search, body));

export const GET: RequestHandler = ({ params, url }) =>
	forward('GET', params.path, url.search, undefined);
export const POST = withBody;
export const PUT = withBody;
export const PATCH = withBody;
export const DELETE: RequestHandler = ({ params, url }) =>
	forward('DELETE', params.path, url.search, undefined);
