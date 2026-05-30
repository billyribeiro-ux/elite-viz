<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { createWatchlist, deleteWatchlist } from '$lib/api';

	let { data } = $props();

	let name = $state('');
	let symbolsText = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);

	async function create(event: Event) {
		event.preventDefault();
		busy = true;
		error = null;
		try {
			const symbols = symbolsText
				.split(',')
				.map((s) => s.trim())
				.filter(Boolean);
			await createWatchlist({ name, symbols });
			name = '';
			symbolsText = '';
			await invalidateAll();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	async function remove(id: string) {
		busy = true;
		error = null;
		try {
			await deleteWatchlist(id);
			await invalidateAll();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head>
	<title>Watchlists · FINVIZ Elite+</title>
</svelte:head>

<form class="create" onsubmit={create}>
	<input bind:value={name} placeholder="Watchlist name" aria-label="Watchlist name" />
	<input
		bind:value={symbolsText}
		placeholder="Symbols, comma-separated (e.g. AAPL, MSFT)"
		aria-label="Symbols"
	/>
	<button type="submit" disabled={busy || !name.trim()}>Create</button>
</form>

{#if error}
	<p class="error" role="alert">⚠ {error}</p>
{/if}

<div class="grid">
	{#each data.watchlists as wl (wl.id)}
		<section class="wl">
			<header>
				<h3>{wl.name}</h3>
				<button class="del" onclick={() => remove(wl.id)} disabled={busy} title="Delete">✕</button>
			</header>
			<div class="symbols">
				{#each wl.symbols as sym (sym)}
					<a class="chip" href="/symbol/{sym}">{sym}</a>
				{:else}
					<span class="muted">No symbols</span>
				{/each}
			</div>
		</section>
	{:else}
		<p class="muted">No watchlists yet — create one above.</p>
	{/each}
</div>

<style>
	.create {
		display: flex;
		gap: 0.6rem;
		flex-wrap: wrap;
		margin-bottom: 1.25rem;
	}
	.create input {
		flex: 1 1 200px;
		background: var(--bg);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.6rem 0.75rem;
	}
	.create button {
		background: var(--accent);
		color: #07121f;
		border: none;
		font-weight: 700;
		border-radius: var(--radius);
		padding: 0.6rem 1.1rem;
	}
	.create button:disabled {
		opacity: 0.5;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
		gap: 1rem;
	}
	.wl {
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 1rem;
	}
	.wl header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.75rem;
	}
	h3 {
		margin: 0;
		font-size: 1rem;
	}
	.del {
		background: transparent;
		border: none;
		color: var(--muted);
		font-size: 0.9rem;
	}
	.del:hover {
		color: var(--danger);
	}
	.symbols {
		display: flex;
		flex-wrap: wrap;
		gap: 0.4rem;
	}
	.chip {
		background: var(--panel-2);
		border: 1px solid var(--border);
		border-radius: 999px;
		padding: 0.25rem 0.7rem;
		font-family: var(--mono);
		font-size: 0.8rem;
		color: var(--accent);
		text-decoration: none;
	}
	.chip:hover {
		border-color: var(--accent);
	}
	.muted {
		color: var(--muted);
	}
	.error {
		background: rgba(248, 114, 114, 0.12);
		border: 1px solid var(--danger);
		color: var(--danger);
		border-radius: var(--radius);
		padding: 0.7rem 0.9rem;
		margin-bottom: 1rem;
	}
</style>
