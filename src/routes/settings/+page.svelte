<script lang="ts">
	import { untrack } from 'svelte';
	import { saveProvider, testProvider } from '$lib/api';
	import type { ProviderConfigInput, ProviderKind } from '$lib/types';

	let { data } = $props();

	let kind = $state<ProviderKind>(untrack(() => data.provider.kind));
	let apiKey = $state('');
	let baseUrl = $state(untrack(() => data.provider.base_url ?? ''));
	let enabled = $state(untrack(() => data.provider.enabled));
	let keyHint = $state(untrack(() => data.provider.api_key_hint));

	let saving = $state(false);
	let testing = $state(false);
	let message = $state<{ ok: boolean; text: string } | null>(null);

	const PROVIDERS: { value: ProviderKind; label: string; needsKey: boolean; note: string }[] = [
		{ value: 'mock', label: 'Built-in (demo data)', needsKey: false, note: 'No network. Synthetic dataset.' },
		{ value: 'finnhub', label: 'Finnhub', needsKey: true, note: 'Uses /quote with your API token.' },
		{ value: 'polygon', label: 'Polygon.io', needsKey: true, note: 'Uses the stock snapshot endpoint.' },
		{ value: 'generic', label: 'Generic HTTP / Webhook', needsKey: false, note: 'GET {url}?symbol=SYM → JSON with a price field. Optional Bearer token.' }
	];

	const current = $derived(PROVIDERS.find((p) => p.value === kind) ?? PROVIDERS[0]);
	const showBaseUrl = $derived(kind === 'generic' || kind !== 'mock');

	function payload(): ProviderConfigInput {
		return {
			kind,
			enabled,
			api_key: apiKey || undefined,
			base_url: baseUrl || undefined
		};
	}

	async function save() {
		saving = true;
		message = null;
		try {
			const view = await saveProvider(payload());
			keyHint = view.api_key_hint;
			apiKey = '';
			message = { ok: true, text: 'Settings saved.' };
		} catch (e) {
			message = { ok: false, text: e instanceof Error ? e.message : String(e) };
		} finally {
			saving = false;
		}
	}

	async function test() {
		testing = true;
		message = null;
		try {
			const result = await testProvider(payload());
			message = { ok: result.ok, text: result.message };
		} catch (e) {
			message = { ok: false, text: e instanceof Error ? e.message : String(e) };
		} finally {
			testing = false;
		}
	}
</script>

<svelte:head>
	<title>Settings · FINVIZ Elite+</title>
</svelte:head>

<h2>Data Provider</h2>
<p class="lead">
	Choose where live quotes come from. Enter your provider's API key (or a webhook
	URL for a custom feed) and connect. Until enabled, the platform serves built-in
	demo data.
</p>

<div class="panel">
	<label class="field">
		<span>Provider</span>
		<select bind:value={kind}>
			{#each PROVIDERS as p (p.value)}
				<option value={p.value}>{p.label}</option>
			{/each}
		</select>
		<small>{current.note}</small>
	</label>

	{#if showBaseUrl}
		<label class="field">
			<span>{kind === 'generic' ? 'Endpoint / Webhook URL' : 'Base URL (optional override)'}</span>
			<input
				bind:value={baseUrl}
				placeholder={kind === 'generic' ? 'https://my-feed.example.com/quote' : 'leave blank for default'}
				spellcheck="false"
				autocomplete="off"
			/>
		</label>
	{/if}

	{#if kind !== 'mock'}
		<label class="field">
			<span>API Key {current.needsKey ? '(required)' : '(optional)'}</span>
			<input
				type="password"
				bind:value={apiKey}
				placeholder={keyHint ? `saved: ${keyHint} — leave blank to keep` : 'paste your API key'}
				autocomplete="off"
			/>
		</label>
	{/if}

	<label class="toggle">
		<input type="checkbox" bind:checked={enabled} />
		<span>Use this provider for live quotes</span>
	</label>

	<div class="actions">
		<button class="primary" onclick={save} disabled={saving}>
			{saving ? 'Saving…' : 'Save'}
		</button>
		<button onclick={test} disabled={testing}>
			{testing ? 'Testing…' : 'Test connection'}
		</button>
	</div>

	{#if message}
		<p class="msg" class:ok={message.ok} class:err={!message.ok} role="status">
			{message.ok ? '✓' : '⚠'} {message.text}
		</p>
	{/if}
</div>

<style>
	h2 {
		margin: 0 0 0.4rem;
	}
	.lead {
		color: var(--muted);
		margin: 0 0 1.5rem;
		max-width: 640px;
		font-size: 0.92rem;
	}
	.panel {
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 1.5rem;
		max-width: 560px;
		display: flex;
		flex-direction: column;
		gap: 1.1rem;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}
	.field span {
		font-size: 0.85rem;
		font-weight: 600;
	}
	.field small {
		color: var(--muted);
		font-size: 0.78rem;
	}
	select,
	.field input {
		background: var(--bg);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.6rem 0.75rem;
		font: inherit;
	}
	.toggle {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		font-size: 0.9rem;
	}
	.actions {
		display: flex;
		gap: 0.6rem;
	}
	button {
		border: 1px solid var(--border);
		background: var(--panel-2);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.6rem 1.2rem;
		font-weight: 600;
	}
	button.primary {
		background: var(--accent);
		color: #07121f;
		border: none;
	}
	button:disabled {
		opacity: 0.55;
	}
	.msg {
		margin: 0;
		padding: 0.65rem 0.85rem;
		border-radius: var(--radius);
		font-size: 0.88rem;
	}
	.msg.ok {
		background: rgba(54, 211, 153, 0.12);
		border: 1px solid var(--accent-2);
		color: var(--accent-2);
	}
	.msg.err {
		background: rgba(248, 114, 114, 0.12);
		border: 1px solid var(--danger);
		color: var(--danger);
	}
</style>
