/** Rune-based auth store: current user + token lifecycle. */
import { getMe, login as apiLogin, register as apiRegister } from './api';
import { getToken, setToken } from './token';
import type { User } from './types';

export const auth = $state<{ user: User | null; ready: boolean }>({
	user: null,
	ready: false
});

/** Resolve the current user from a stored token (call once on mount). */
export async function initAuth(): Promise<void> {
	if (auth.ready) return;
	if (getToken()) {
		try {
			auth.user = await getMe();
		} catch {
			setToken(null);
		}
	}
	auth.ready = true;
}

export async function login(email: string, password: string): Promise<void> {
	const res = await apiLogin({ email, password });
	setToken(res.token);
	auth.user = res.user;
}

export async function register(email: string, password: string): Promise<void> {
	const res = await apiRegister({ email, password });
	setToken(res.token);
	auth.user = res.user;
}

export function logout(): void {
	setToken(null);
	auth.user = null;
}
