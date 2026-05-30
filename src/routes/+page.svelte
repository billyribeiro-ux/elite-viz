<script lang="ts">
	import { untrack } from 'svelte';
	import { runScreen } from '$lib/api';
	import QueryBar from '$lib/components/QueryBar.svelte';
	import ResultsTable from '$lib/components/ResultsTable.svelte';
	import type { ScreenerRow, SortOrder } from '$lib/types';

	let { data } = $props();

	// Seed local, user-mutable state once from the server-loaded snapshot.
	let query = $state(untrack(() => data.initial.query));
	let sort = $state('market_cap');
	let order = $state<SortOrder>('desc');
	let rows = $state<ScreenerRow[]>(untrack(() => data.initial.rows));
	let total = $state(untrack(() => data.initial.total));
	let matched = $state(untrack(() => data.initial.matched));
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function run() {
		loading = true;
		error = null;
		try {
			const res = await runScreen({ query, sort, order, limit: 100 });
			rows = res.rows;
			total = res.total;
			matched = res.matched;
			query = res.query;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function changeSort(field: string) {
		if (sort === field) {
			order = order === 'asc' ? 'desc' : 'asc';
		} else {
			sort = field;
			order = 'desc';
		}
		run();
	}
</script>

<svelte:head>
	<title>FINVIZ Elite+ · Screener</title>
</svelte:head>

<QueryBar bind:query bind:sort bind:order {loading} presets={data.presets} fields={data.fields} onRun={run} />

<div class="stats">
	<span><strong>{matched}</strong> matches</span>
	<span class="sep">/</span>
	<span>{total} symbols</span>
	<label class="live" class:on={live}>
		<input type="checkbox" bind:checked={live} />
		<span class="led"></span> Live
	</label>
	{#if query.trim()}
		<code>{query}</code>
	{/if}
</div>

{#if error}
	<p class="error" role="alert">⚠ {error}</p>
{/if}

<ResultsTable {rows} {sort} {order} onSort={changeSort} />

<style>
	.stats {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		margin-bottom: 0.85rem;
		color: var(--muted);
		font-size: 0.9rem;
	}
	.stats strong {
		color: var(--text);
	}
	.stats .sep {
		opacity: 0.5;
	}
	.stats code {
		font-family: var(--mono);
		font-size: 0.8rem;
		color: var(--accent);
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 0.15rem 0.5rem;
		margin-left: auto;
	}
	.live {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		font-size: 0.82rem;
		cursor: pointer;
		user-select: none;
	}
	.live .led {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--border);
	}
	.live.on .led {
		background: var(--accent-2);
		box-shadow: 0 0 6px var(--accent-2);
		animation: blink 1.2s infinite;
	}
	.live.on {
		color: var(--accent-2);
	}
	@keyframes blink {
		50% {
			opacity: 0.4;
		}
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
