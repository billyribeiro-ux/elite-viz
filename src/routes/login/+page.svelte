<script lang="ts">
	import { goto } from '$app/navigation';
	import { login, register } from '$lib/auth.svelte';

	let mode = $state<'login' | 'register'>('login');
	let email = $state('');
	let password = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);

	async function submit(event: Event) {
		event.preventDefault();
		busy = true;
		error = null;
		try {
			if (mode === 'login') await login(email, password);
			else await register(email, password);
			await goto('/');
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head>
	<title>{mode === 'login' ? 'Sign in' : 'Create account'} · FINVIZ Elite+</title>
</svelte:head>

<div class="auth">
	<h2>{mode === 'login' ? 'Sign in' : 'Create account'}</h2>
	<form onsubmit={submit}>
		<label>
			<span>Email</span>
			<input type="email" bind:value={email} autocomplete="username" required />
		</label>
		<label>
			<span>Password</span>
			<input
				type="password"
				bind:value={password}
				autocomplete={mode === 'login' ? 'current-password' : 'new-password'}
				minlength="8"
				required
			/>
		</label>
		{#if error}
			<p class="error" role="alert">⚠ {error}</p>
		{/if}
		<button type="submit" disabled={busy}>
			{busy ? 'Please wait…' : mode === 'login' ? 'Sign in' : 'Create account'}
		</button>
	</form>
	<p class="switch">
		{#if mode === 'login'}
			No account?
			<button type="button" onclick={() => (mode = 'register')}>Create one</button>
		{:else}
			Already registered?
			<button type="button" onclick={() => (mode = 'login')}>Sign in</button>
		{/if}
	</p>
</div>

<style>
	.auth {
		max-width: 400px;
		margin: 2rem auto;
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 2rem;
	}
	h2 {
		margin: 0 0 1.25rem;
	}
	form {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	label {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}
	label span {
		font-size: 0.85rem;
		font-weight: 600;
	}
	input {
		background: var(--bg);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.6rem 0.75rem;
		font: inherit;
	}
	button[type='submit'] {
		background: var(--accent);
		color: #07121f;
		border: none;
		font-weight: 700;
		border-radius: var(--radius);
		padding: 0.65rem;
	}
	button[type='submit']:disabled {
		opacity: 0.55;
	}
	.switch {
		margin: 1.25rem 0 0;
		font-size: 0.88rem;
		color: var(--muted);
	}
	.switch button {
		background: none;
		border: none;
		color: var(--accent);
		font: inherit;
		padding: 0;
		text-decoration: underline;
	}
	.error {
		margin: 0;
		color: var(--danger);
		font-size: 0.88rem;
	}
</style>
