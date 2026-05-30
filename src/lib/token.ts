import { browser } from '$app/environment';

const KEY = 'elite-viz-token';

let token: string | null = browser ? localStorage.getItem(KEY) : null;

export function getToken(): string | null {
	return token;
}

export function setToken(value: string | null): void {
	token = value;
	if (browser) {
		if (value) localStorage.setItem(KEY, value);
		else localStorage.removeItem(KEY);
	}
}

/** Authorization header for authenticated requests, or empty if signed out. */
export function authHeaders(): Record<string, string> {
	return token ? { authorization: `Bearer ${token}` } : {};
}
