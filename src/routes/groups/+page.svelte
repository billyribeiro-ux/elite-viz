<script lang="ts">
	import { untrack } from 'svelte';
	import { exportGroupsCsv, getGroups } from '$lib/api';
	import { compactNumber, marketCap, optional, percent, trend } from '$lib/format';
	import type { GroupBy, GroupRow, SortOrder } from '$lib/types';

	let { data } = $props();

	let by = $state<GroupBy>(untrack(() => data.by));
	let groups = $state<GroupRow[]>(untrack(() => data.groups));
	let loading = $state(false);
	let error = $state<string | null>(untrack(() => data.error));
	let sort = $state<keyof GroupRow>('total_market_cap');
	let order = $state<SortOrder>('desc');
	let exporting = $state(false);
	let metric = $state<'avg_perf_week' | 'avg_perf_month' | 'avg_perf_year' | 'total_market_cap'>(
		'total_market_cap'
	);

	const groupOptions: { value: GroupBy; label: string }[] = [
		{ value: 'sector', label: 'Sector' },
		{ value: 'industry', label: 'Industry' },
		{ value: 'country', label: 'Country' }
	];

	const metricOptions: { value: typeof metric; label: string }[] = [
		{ value: 'total_market_cap', label: 'Total Mkt Cap' },
		{ value: 'avg_perf_week', label: 'Perf Week' },
		{ value: 'avg_perf_month', label: 'Perf Month' },
		{ value: 'avg_perf_year', label: 'Perf Year' }
	];

	type Col = { key: keyof GroupRow; label: string; align: 'left' | 'right' };
	const columns: Col[] = [
		{ key: 'name', label: 'Group', align: 'left' },
		{ key: 'count', label: '#', align: 'right' },
		{ key: 'avg_change_pct', label: 'Avg Change %', align: 'right' },
		{ key: 'avg_pe', label: 'Avg P/E', align: 'right' },
		{ key: 'total_market_cap', label: 'Total Mkt Cap', align: 'right' },
		{ key: 'avg_perf_week', label: 'Perf Week', align: 'right' },
		{ key: 'avg_perf_month', label: 'Perf Month', align: 'right' },
		{ key: 'avg_perf_year', label: 'Perf Year', align: 'right' }
	];

	const sortedGroups = $derived.by(() => {
		const dir = order === 'asc' ? 1 : -1;
		return [...groups].sort((a, b) => {
			const av = a[sort];
			const bv = b[sort];
			if (typeof av === 'number' && typeof bv === 'number') return (av - bv) * dir;
			return String(av).localeCompare(String(bv)) * dir;
		});
	});

	// Largest absolute metric value, for scaling the bar visualization.
	const maxMetric = $derived(
		Math.max(1, ...groups.map((g) => Math.abs(g[metric])))
	);

	async function reload() {
		loading = true;
		error = null;
		try {
			groups = await getGroups(by);
		} catch (e) {
			groups = [];
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function changeSort(key: keyof GroupRow) {
		if (sort === key) {
			order = order === 'asc' ? 'desc' : 'asc';
		} else {
			sort = key;
			order = key === 'name' ? 'asc' : 'desc';
		}
	}

	function indicator(key: keyof GroupRow): string {
		if (sort !== key) return '';
		return order === 'asc' ? ' ▲' : ' ▼';
	}

	function barLabel(g: GroupRow): string {
		return metric === 'total_market_cap' ? marketCap(g[metric]) : percent(g[metric]);
	}

	function isPerf(): boolean {
		return metric !== 'total_market_cap';
	}

	async function doExport() {
		exporting = true;
		error = null;
		try {
			await exportGroupsCsv(by);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			exporting = false;
		}
	}
</script>

<svelte:head>
	<title>FINVIZ Elite+ · Groups</title>
</svelte:head>

<header class="head">
	<div>
		<h2>Groups</h2>
		<p>Aggregated screener rows by {by}.</p>
	</div>
	<div class="controls">
		<label>
			Group by
			<select
				bind:value={by}
				onchange={reload}
				disabled={loading}
			>
				{#each groupOptions as opt (opt.value)}
					<option value={opt.value}>{opt.label}</option>
				{/each}
			</select>
		</label>
		<label>
			Bar metric
			<select bind:value={metric}>
				{#each metricOptions as opt (opt.value)}
					<option value={opt.value}>{opt.label}</option>
				{/each}
			</select>
		</label>
		<button type="button" class="export" onclick={doExport} disabled={exporting || loading}>
			{exporting ? 'Exporting…' : '⭳ Export CSV'}
		</button>
	</div>
</header>

{#if error}
	<p class="error" role="alert">⚠ {error}</p>
{/if}

<div class="bars">
	{#each sortedGroups as g (g.name)}
		<div class="bar-row">
			<span class="bar-name" title={g.name}>{g.name}</span>
			<div class="bar-track">
				<div
					class="bar-fill {isPerf() ? trend(g[metric]) : 'neutral'}"
					style="width: {(Math.abs(g[metric]) / maxMetric) * 100}%"
				></div>
			</div>
			<span class="bar-val {isPerf() ? trend(g[metric]) : ''}">{barLabel(g)}</span>
		</div>
	{:else}
		<p class="empty">No groups to display.</p>
	{/each}
</div>

<div class="wrap">
	<table>
		<thead>
			<tr>
				{#each columns as col (col.key)}
					<th
						class:right={col.align === 'right'}
						class:active={sort === col.key}
						onclick={() => changeSort(col.key)}
					>
						{col.label}{indicator(col.key)}
					</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each sortedGroups as g (g.name)}
				<tr>
					<td class="name">{g.name}</td>
					<td class="right">{g.count}</td>
					<td class="right {trend(g.avg_change_pct)}">{percent(g.avg_change_pct)}</td>
					<td class="right">{optional(g.avg_pe)}</td>
					<td class="right">{marketCap(g.total_market_cap)}</td>
					<td class="right {trend(g.avg_perf_week)}">{percent(g.avg_perf_week)}</td>
					<td class="right {trend(g.avg_perf_month)}">{percent(g.avg_perf_month)}</td>
					<td class="right {trend(g.avg_perf_year)}">{percent(g.avg_perf_year)}</td>
				</tr>
			{:else}
				<tr>
					<td colspan={columns.length} class="empty">No groups for this view.</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<p class="foot">{compactNumber(groups.length)} groups · sorted by {sort}</p>

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
	.controls {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
		align-items: flex-end;
	}
	.export {
		background: var(--panel-2);
		color: var(--text);
		border: 1px solid var(--border);
		border-radius: 999px;
		padding: 0.4rem 0.9rem;
		font-size: 0.8rem;
		font-weight: 600;
		cursor: pointer;
	}
	.export:hover:not(:disabled) {
		border-color: var(--accent);
		color: var(--accent);
	}
	.export:disabled {
		opacity: 0.6;
		cursor: progress;
	}
	.controls label {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		font-size: 0.75rem;
		color: var(--muted);
	}
	select {
		background: var(--panel);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.4rem 0.6rem;
		font-size: 0.85rem;
	}
	.bars {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
		margin-bottom: 1.5rem;
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 0.9rem 1rem;
	}
	.bar-row {
		display: grid;
		grid-template-columns: 160px 1fr 110px;
		align-items: center;
		gap: 0.75rem;
		font-size: 0.82rem;
	}
	.bar-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--muted);
	}
	.bar-track {
		background: var(--panel-2);
		border-radius: 4px;
		height: 14px;
		overflow: hidden;
	}
	.bar-fill {
		height: 100%;
		border-radius: 4px;
		transition: width 0.3s ease;
	}
	.bar-fill.neutral {
		background: var(--accent);
	}
	.bar-fill.up {
		background: var(--accent-2);
	}
	.bar-fill.down {
		background: var(--danger);
	}
	.bar-fill.flat {
		background: var(--muted);
	}
	.bar-val {
		text-align: right;
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
	.name {
		font-weight: 600;
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
	.error {
		background: rgba(248, 114, 114, 0.12);
		border: 1px solid var(--danger);
		color: var(--danger);
		border-radius: var(--radius);
		padding: 0.7rem 0.9rem;
		margin-bottom: 1rem;
	}
	.foot {
		margin-top: 0.85rem;
		color: var(--muted);
		font-size: 0.8rem;
	}
</style>
