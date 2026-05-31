<script lang="ts">
	import { marketCap } from '$lib/format';
	import type { EtfProfile } from '$lib/types';

	let { data } = $props();

	const etfs = $derived<EtfProfile[]>(data.etfs ?? []);
	const loadError = $derived<string | null>(data.error ?? null);

	function expense(ratio: number): string {
		return Number.isFinite(ratio) ? `${ratio.toFixed(2)}%` : '—';
	}
</script>

<svelte:head>
	<title>FINVIZ Elite+ · ETFs</title>
</svelte:head>

<header class="head">
	<div>
		<h2>ETFs</h2>
		<p>Exchange-traded funds · expense ratio, AUM and holdings.</p>
	</div>
</header>

{#if loadError}
	<p class="error" role="alert">⚠ {loadError}</p>
{/if}

{#if etfs.length}
	<div class="grid">
		{#each etfs as etf (etf.symbol)}
			<a class="card" href={`/etf/${encodeURIComponent(etf.symbol)}`}>
				<div class="card-head">
					<span class="sym">{etf.symbol}</span>
					{#if etf.category}<span class="cat">{etf.category}</span>{/if}
				</div>
				<p class="name" title={etf.name}>{etf.name}</p>
				<dl class="metrics">
					<div>
						<dt>Expense</dt>
						<dd>{expense(etf.expense_ratio)}</dd>
					</div>
					<div>
						<dt>AUM</dt>
						<dd>{marketCap(etf.aum)}</dd>
					</div>
					<div>
						<dt>Holdings</dt>
						<dd>{etf.holdings?.length ?? 0}</dd>
					</div>
				</dl>
			</a>
		{/each}
	</div>
{:else}
	<p class="empty">No ETFs available.</p>
{/if}

<style>
	.head {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		flex-wrap: wrap;
		gap: 1rem;
		margin-bottom: 1.25rem;
	}
	h2 {
		margin: 0;
		font-size: 1.4rem;
	}
	.head p {
		margin: 0.2rem 0 0;
		color: var(--muted);
		font-size: 0.85rem;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 1rem;
	}
	.card {
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 1rem;
		text-decoration: none;
		color: var(--text);
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}
	.card:hover {
		border-color: var(--accent);
	}
	.card-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.5rem;
	}
	.sym {
		font-family: var(--mono);
		font-weight: 700;
		font-size: 1.1rem;
	}
	.cat {
		color: var(--muted);
		font-size: 0.7rem;
		text-transform: uppercase;
		letter-spacing: 0.4px;
		white-space: nowrap;
	}
	.name {
		margin: 0;
		color: var(--muted);
		font-size: 0.85rem;
		line-height: 1.3;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.metrics {
		display: flex;
		justify-content: space-between;
		gap: 0.5rem;
		margin: 0.25rem 0 0;
	}
	.metrics div {
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
	}
	.metrics dt {
		color: var(--muted);
		font-size: 0.7rem;
		text-transform: uppercase;
		letter-spacing: 0.4px;
	}
	.metrics dd {
		margin: 0;
		font-weight: 600;
		font-size: 0.9rem;
		font-variant-numeric: tabular-nums;
	}
	.empty {
		text-align: center;
		color: var(--muted);
		padding: 2rem;
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
