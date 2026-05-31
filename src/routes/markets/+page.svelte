<script lang="ts">
	import { percent, price, trend } from '$lib/format';
	import type { MarketAsset } from '$lib/types';

	let { data } = $props();

	type Board = 'futures' | 'forex' | 'crypto';

	let selected = $state<Board>('futures');

	const boards: { value: Board; label: string }[] = [
		{ value: 'futures', label: 'Futures' },
		{ value: 'forex', label: 'Forex' },
		{ value: 'crypto', label: 'Crypto' }
	];

	/** Rows for the currently-selected board. */
	const rows = $derived<MarketAsset[]>(data[selected] ?? []);

	/**
	 * Forex rates are usually sub-10 and need finer precision; show 4 decimals
	 * for small numbers and fall back to the shared `price()` helper otherwise.
	 */
	function fmtPrice(board: Board, n: number): string {
		if (board === 'forex' && Math.abs(n) < 10) return n.toFixed(4);
		return price(n);
	}

	/** Stable, sorted list of group names present in the selected board. */
	const groupNames = $derived.by(() => {
		const seen = new Set<string>();
		for (const r of rows) seen.add(r.group || 'Other');
		return [...seen].sort((a, b) => a.localeCompare(b));
	});

	/** Rows bucketed by their `group` field, in `groupNames` order. */
	const grouped = $derived.by(() => {
		const map = new Map<string, MarketAsset[]>();
		for (const name of groupNames) map.set(name, []);
		for (const r of rows) {
			const key = r.group || 'Other';
			(map.get(key) ?? map.set(key, []).get(key)!).push(r);
		}
		return [...map.entries()];
	});

	/** Tile background tint based on change_pct magnitude (heat strip). */
	function heatStyle(n: number): string {
		const mag = Math.min(1, Math.abs(n) / 5);
		const alpha = (0.12 + mag * 0.5).toFixed(3);
		const color = n > 0 ? '52, 211, 153' : n < 0 ? '248, 114, 114' : '148, 163, 184';
		return `background: rgba(${color}, ${alpha});`;
	}
</script>

<svelte:head>
	<title>FINVIZ Elite+ · Markets</title>
</svelte:head>

<header class="head">
	<div>
		<h2>Markets</h2>
		<p>Live futures, forex and crypto boards, grouped by category.</p>
	</div>
	<div class="tabs" role="tablist" aria-label="Market board">
		{#each boards as b (b.value)}
			<button
				type="button"
				role="tab"
				aria-selected={selected === b.value}
				class:active={selected === b.value}
				onclick={() => (selected = b.value)}
			>
				{b.label}
			</button>
		{/each}
	</div>
</header>

{#if rows.length === 0}
	<p class="empty" role="status">No {selected} data available right now.</p>
{:else}
	<div class="heat" aria-hidden="true">
		{#each rows as r (r.symbol)}
			<span class="tile {trend(r.change_pct)}" style={heatStyle(r.change_pct)} title={r.name}>
				<span class="tile-sym">{r.symbol}</span>
				<span class="tile-pct">{percent(r.change_pct)}</span>
			</span>
		{/each}
	</div>

	<div class="wrap">
		<table>
			<thead>
				<tr>
					<th class="left">Symbol</th>
					<th class="left">Name</th>
					<th class="right">Price</th>
					<th class="right">Change %</th>
					<th class="right">Perf Week</th>
					<th class="right">Perf Month</th>
				</tr>
			</thead>
			<tbody>
				{#each grouped as [name, items] (name)}
					<tr class="group-row">
						<td colspan={6}>{name} <span class="count">· {items.length}</span></td>
					</tr>
					{#each items as r (r.symbol)}
						<tr>
							<td class="sym">{r.symbol}</td>
							<td class="name">{r.name}</td>
							<td class="right">{fmtPrice(selected, r.price)}</td>
							<td class="right {trend(r.change_pct)}">{percent(r.change_pct)}</td>
							<td class="right {trend(r.perf_week)}">{percent(r.perf_week)}</td>
							<td class="right {trend(r.perf_month)}">{percent(r.perf_month)}</td>
						</tr>
					{/each}
				{/each}
			</tbody>
		</table>
	</div>

	<p class="foot">{rows.length} instruments · {groupNames.length} groups</p>
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
	.tabs {
		display: inline-flex;
		gap: 0.25rem;
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: 999px;
		padding: 0.25rem;
	}
	.tabs button {
		background: transparent;
		border: none;
		color: var(--muted);
		font-weight: 600;
		font-size: 0.85rem;
		padding: 0.4rem 0.95rem;
		border-radius: 999px;
		cursor: pointer;
	}
	.tabs button:hover {
		color: var(--text);
	}
	.tabs button.active {
		background: var(--panel-2);
		color: var(--text);
	}
	.heat {
		display: flex;
		flex-wrap: wrap;
		gap: 0.3rem;
		margin-bottom: 1.25rem;
	}
	.tile {
		display: flex;
		flex-direction: column;
		gap: 0.1rem;
		min-width: 76px;
		padding: 0.35rem 0.5rem;
		border-radius: var(--radius);
		border: 1px solid var(--border);
		font-size: 0.72rem;
		line-height: 1.2;
	}
	.tile-sym {
		font-weight: 700;
	}
	.tile-pct {
		font-variant-numeric: tabular-nums;
	}
	.wrap {
		border: 1px solid var(--border);
		border-radius: var(--radius);
		overflow: auto;
		background: var(--panel);
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.85rem;
	}
	th,
	td {
		padding: 0.45rem 0.85rem;
		text-align: left;
		white-space: nowrap;
		border-bottom: 1px solid var(--border);
	}
	th {
		position: sticky;
		top: 0;
		background: var(--panel-2);
		color: var(--muted);
		font-weight: 600;
		user-select: none;
	}
	th.right,
	td.right {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}
	.group-row td {
		background: var(--panel-2);
		font-weight: 700;
		font-size: 0.78rem;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--text);
	}
	.group-row .count {
		color: var(--muted);
		font-weight: 600;
	}
	tbody tr:not(.group-row):hover {
		background: var(--panel-2);
	}
	.sym {
		font-weight: 700;
	}
	.name {
		color: var(--muted);
	}
	.up {
		color: var(--accent-2);
	}
	.down {
		color: var(--danger);
	}
	.flat {
		color: var(--muted);
	}
	.empty {
		text-align: center;
		color: var(--muted);
		padding: 3rem 2rem;
		border: 1px dashed var(--border);
		border-radius: var(--radius);
		background: var(--panel);
	}
	.foot {
		margin-top: 0.85rem;
		color: var(--muted);
		font-size: 0.8rem;
	}
</style>
