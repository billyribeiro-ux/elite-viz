<script lang="ts">
	import { compactNumber, marketCap, optional, percent, price, trend } from '$lib/format';
	import type { ScreenerRow, SortOrder } from '$lib/types';

	let {
		rows,
		sort,
		order,
		onSort
	}: {
		rows: ScreenerRow[];
		sort: string;
		order: SortOrder;
		onSort: (field: string) => void;
	} = $props();

	type Align = 'left' | 'right';
	const columns: { key: string; label: string; align: Align }[] = [
		{ key: 'symbol', label: 'Symbol', align: 'left' },
		{ key: 'name', label: 'Company', align: 'left' },
		{ key: 'sector', label: 'Sector', align: 'left' },
		{ key: 'price', label: 'Price', align: 'right' },
		{ key: 'change_pct', label: 'Change %', align: 'right' },
		{ key: 'volume', label: 'Volume', align: 'right' },
		{ key: 'market_cap', label: 'Mkt Cap', align: 'right' },
		{ key: 'pe', label: 'P/E', align: 'right' },
		{ key: 'beta', label: 'Beta', align: 'right' }
	];

	function indicator(key: string): string {
		if (sort !== key) return '';
		return order === 'asc' ? ' ▲' : ' ▼';
	}
</script>

<div class="wrap">
	<table>
		<thead>
			<tr>
				{#each columns as col (col.key)}
					<th
						class:right={col.align === 'right'}
						class:active={sort === col.key}
						onclick={() => onSort(col.key)}
					>
						{col.label}{indicator(col.key)}
					</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each rows as row (row.symbol)}
				<tr>
					<td class="sym"><a href="/symbol/{row.symbol}">{row.symbol}</a></td>
					<td class="name">{row.name}</td>
					<td class="muted">{row.sector}</td>
					<td class="right">{price(row.price)}</td>
					<td class="right {trend(row.change_pct)}">{percent(row.change_pct)}</td>
					<td class="right">{compactNumber(row.volume)}</td>
					<td class="right">{marketCap(row.market_cap)}</td>
					<td class="right">{optional(row.pe)}</td>
					<td class="right">{optional(row.beta)}</td>
				</tr>
			{:else}
				<tr>
					<td colspan={columns.length} class="empty">No matches for this filter.</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	.wrap {
		border: 1px solid var(--border);
		border-radius: var(--radius);
		overflow: auto;
		background: var(--panel);
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.88rem;
	}
	th,
	td {
		padding: 0.6rem 0.85rem;
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
		cursor: pointer;
		user-select: none;
	}
	th.right,
	td.right {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}
	th.active {
		color: var(--text);
	}
	tbody tr:hover {
		background: var(--panel-2);
	}
	.sym a {
		font-weight: 700;
		color: var(--accent);
		font-family: var(--mono);
		text-decoration: none;
	}
	.sym a:hover {
		text-decoration: underline;
	}
	.name {
		max-width: 220px;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.muted {
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
		padding: 2rem;
	}
</style>
