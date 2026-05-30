<script lang="ts">
	import { price, trend } from '$lib/format';

	let { data } = $props();
	const s = $derived(data.summary);

	function money(n: number): string {
		return `$${n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
	}
	function signed(n: number): string {
		return `${n >= 0 ? '+' : '-'}${money(Math.abs(n))}`;
	}
</script>

<svelte:head>
	<title>Portfolio · FINVIZ Elite+</title>
</svelte:head>

<div class="cards">
	<div class="card">
		<span class="k">Market Value</span>
		<span class="v">{money(s.market_value)}</span>
	</div>
	<div class="card">
		<span class="k">Cost Basis</span>
		<span class="v">{money(s.cost_basis)}</span>
	</div>
	<div class="card {trend(s.unrealized_pnl)}">
		<span class="k">Unrealized P&amp;L</span>
		<span class="v">{signed(s.unrealized_pnl)}</span>
	</div>
	<div class="card {trend(s.unrealized_pnl)}">
		<span class="k">Return</span>
		<span class="v">{s.unrealized_pnl_pct >= 0 ? '+' : ''}{s.unrealized_pnl_pct.toFixed(2)}%</span>
	</div>
</div>

<div class="wrap">
	<table>
		<thead>
			<tr>
				<th>Symbol</th>
				<th class="r">Qty</th>
				<th class="r">Avg Cost</th>
				<th class="r">Last</th>
				<th class="r">Market Value</th>
				<th class="r">Unrealized P&amp;L</th>
				<th class="r">Return</th>
			</tr>
		</thead>
		<tbody>
			{#each s.positions as p (p.symbol)}
				<tr>
					<td class="sym"><a href="/symbol/{p.symbol}">{p.symbol}</a></td>
					<td class="r">{p.quantity}</td>
					<td class="r">{price(p.avg_price)}</td>
					<td class="r">{price(p.last_price)}</td>
					<td class="r">{money(p.market_value)}</td>
					<td class="r {trend(p.unrealized_pnl)}">{signed(p.unrealized_pnl)}</td>
					<td class="r {trend(p.unrealized_pnl)}">{p.unrealized_pnl_pct >= 0 ? '+' : ''}{p.unrealized_pnl_pct.toFixed(2)}%</td>
				</tr>
			{:else}
				<tr><td colspan="7" class="empty">No positions.</td></tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	.cards {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
		gap: 0.85rem;
		margin-bottom: 1.5rem;
	}
	.card {
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 1rem;
	}
	.card .k {
		display: block;
		color: var(--muted);
		font-size: 0.78rem;
		margin-bottom: 0.35rem;
	}
	.card .v {
		font-size: 1.3rem;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}
	.card.up .v {
		color: var(--accent-2);
	}
	.card.down .v {
		color: var(--danger);
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
		font-size: 0.88rem;
	}
	th,
	td {
		padding: 0.6rem 0.85rem;
		border-bottom: 1px solid var(--border);
		white-space: nowrap;
		text-align: left;
	}
	th {
		background: var(--panel-2);
		color: var(--muted);
		font-weight: 600;
	}
	.r {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}
	.sym a {
		color: var(--accent);
		font-weight: 700;
		font-family: var(--mono);
		text-decoration: none;
	}
	.up {
		color: var(--accent-2);
	}
	.down {
		color: var(--danger);
	}
	.empty {
		text-align: center;
		color: var(--muted);
		padding: 2rem;
	}
</style>
