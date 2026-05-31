<script lang="ts">
	import { marketCap } from '$lib/format';
	import type { EtfHolding, EtfProfile } from '$lib/types';

	let { data } = $props();

	const etf = $derived<EtfProfile>(data.etf);

	function expense(ratio: number): string {
		return Number.isFinite(ratio) ? `${ratio.toFixed(2)}%` : '—';
	}

	function weightPct(weight: number): string {
		return Number.isFinite(weight) ? `${weight.toFixed(2)}%` : '—';
	}

	// Holdings sorted by weight (descending); negatives clamped for tile sizing.
	const holdings = $derived<EtfHolding[]>(
		[...(etf.holdings ?? [])].sort((a, b) => (b.weight ?? 0) - (a.weight ?? 0))
	);

	const totalWeight = $derived(
		Math.max(1, holdings.reduce((s, h) => s + Math.max(0, h.weight ?? 0), 0))
	);

	function tileFlex(weight: number): number {
		return Math.max(0.02, Math.max(0, weight ?? 0) / totalWeight);
	}
</script>

<svelte:head>
	<title>{etf.symbol} · FINVIZ Elite+</title>
</svelte:head>

<a class="back" href="/etf">← ETFs</a>

<header class="head">
	<div>
		<h2>{etf.symbol}</h2>
		<p>{etf.name}{etf.category ? ` · ${etf.category}` : ''}</p>
	</div>
</header>

<div class="stats">
	<div class="stat">
		<span class="k">Expense Ratio</span>
		<span class="v">{expense(etf.expense_ratio)}</span>
	</div>
	<div class="stat">
		<span class="k">AUM</span>
		<span class="v">{marketCap(etf.aum)}</span>
	</div>
	<div class="stat">
		<span class="k">Category</span>
		<span class="v">{etf.category || '—'}</span>
	</div>
	<div class="stat">
		<span class="k">Holdings</span>
		<span class="v">{holdings.length}</span>
	</div>
</div>

{#if holdings.length}
	<section class="block">
		<h3>Holdings weight</h3>
		<div class="treemap">
			{#each holdings as h (h.symbol)}
				<a
					class="tile"
					href={`/symbol/${encodeURIComponent(h.symbol)}`}
					style="flex-grow: {tileFlex(h.weight)}"
					title={`${h.symbol} · ${h.name} · ${weightPct(h.weight)}`}
				>
					<span class="sym">{h.symbol}</span>
					<span class="wt">{weightPct(h.weight)}</span>
				</a>
			{/each}
		</div>
	</section>

	<section class="block">
		<h3>Holdings</h3>
		<div class="table-wrap">
			<table>
				<thead>
					<tr>
						<th>Symbol</th>
						<th>Name</th>
						<th class="num">Weight</th>
					</tr>
				</thead>
				<tbody>
					{#each holdings as h (h.symbol)}
						<tr>
							<td class="mono">
								<a href={`/symbol/${encodeURIComponent(h.symbol)}`}>{h.symbol}</a>
							</td>
							<td>{h.name ?? '—'}</td>
							<td class="num">{weightPct(h.weight)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</section>
{:else}
	<p class="empty">No holdings data.</p>
{/if}

<style>
	.back {
		display: inline-block;
		margin-bottom: 1rem;
		color: var(--muted);
		text-decoration: none;
		font-size: 0.9rem;
	}
	.back:hover {
		color: var(--text);
	}
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
		font-size: 1.6rem;
		font-family: var(--mono);
	}
	.head p {
		margin: 0.2rem 0 0;
		color: var(--muted);
		font-size: 0.85rem;
	}
	.stats {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
		gap: 0.75rem;
		margin-bottom: 1.5rem;
	}
	.stat {
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 0.75rem 0.9rem;
	}
	.stat .k {
		display: block;
		color: var(--muted);
		font-size: 0.75rem;
		margin-bottom: 0.25rem;
	}
	.stat .v {
		font-size: 1.05rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.block {
		margin-top: 1.75rem;
	}
	.block h3 {
		margin: 0 0 0.75rem;
		font-size: 0.8rem;
		color: var(--muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}
	.treemap {
		display: flex;
		flex-wrap: wrap;
		gap: 3px;
		align-content: flex-start;
	}
	.tile {
		flex-basis: 0;
		min-width: 64px;
		min-height: 54px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-decoration: none;
		color: #fff;
		background: var(--accent);
		border-radius: 3px;
		padding: 4px;
		overflow: hidden;
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.45);
	}
	.tile:hover {
		outline: 2px solid #fff;
		outline-offset: -2px;
	}
	.tile .sym {
		font-size: 0.78rem;
		font-weight: 700;
		font-family: var(--mono);
		white-space: nowrap;
	}
	.tile .wt {
		font-size: 0.68rem;
		font-variant-numeric: tabular-nums;
	}
	.table-wrap {
		overflow-x: auto;
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.85rem;
	}
	th,
	td {
		text-align: left;
		padding: 0.5rem 0.7rem;
		border-bottom: 1px solid var(--border);
		white-space: nowrap;
	}
	th {
		color: var(--muted);
		font-size: 0.72rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.4px;
	}
	td {
		font-variant-numeric: tabular-nums;
	}
	td.mono a {
		font-family: var(--mono);
		color: var(--accent);
		text-decoration: none;
		font-weight: 600;
	}
	td.mono a:hover {
		text-decoration: underline;
	}
	th.num,
	td.num {
		text-align: right;
	}
	.empty {
		text-align: center;
		color: var(--muted);
		padding: 2rem;
	}
</style>
