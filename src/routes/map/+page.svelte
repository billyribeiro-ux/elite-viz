<script lang="ts">
	import { marketCap, percent } from '$lib/format';
	import type { ScreenerRow } from '$lib/types';

	let { data } = $props();

	const rows = $derived<ScreenerRow[]>(data.rows ?? []);
	const error = $derived<string | null>(data.error ?? null);

	type Window = 'change_pct' | 'perf_week' | 'perf_month' | 'perf_year';
	let window = $state<Window>('change_pct');

	const windowOptions: { value: Window; label: string }[] = [
		{ value: 'change_pct', label: 'Day' },
		{ value: 'perf_week', label: 'Week' },
		{ value: 'perf_month', label: 'Month' },
		{ value: 'perf_year', label: 'Year' }
	];

	type Sector = {
		name: string;
		total: number;
		tiles: { row: ScreenerRow; weight: number }[];
	};

	// Group rows by sector; weight each tile by market cap (clamped to >=0).
	const sectors = $derived.by<Sector[]>(() => {
		const map = new Map<string, ScreenerRow[]>();
		for (const r of rows) {
			const key = r.sector || 'Other';
			const list = map.get(key);
			if (list) list.push(r);
			else map.set(key, [r]);
		}
		const result: Sector[] = [];
		for (const [name, list] of map) {
			const tiles = list
				.map((row) => ({ row, weight: Math.max(0, row.market_cap) }))
				.sort((a, b) => b.weight - a.weight);
			const total = tiles.reduce((s, t) => s + t.weight, 0);
			result.push({ name, total, tiles });
		}
		return result.sort((a, b) => b.total - a.total);
	});

	const grandTotal = $derived(Math.max(1, sectors.reduce((s, sec) => s + sec.total, 0)));

	// Color clamp: ±5% maps to full red/green saturation.
	function colorFor(v: number): string {
		const clamp = Math.max(-1, Math.min(1, v / 5));
		if (clamp >= 0) {
			// neutral grey -> green
			const t = clamp;
			const r = Math.round(54 + (54 - 54) * t);
			const g = Math.round(58 + (211 - 58) * t);
			const b = Math.round(72 + (153 - 72) * t);
			return `rgb(${r}, ${g}, ${b})`;
		}
		// neutral grey -> red
		const t = -clamp;
		const r = Math.round(54 + (248 - 54) * t);
		const g = Math.round(58 + (114 - 58) * t);
		const b = Math.round(72 + (114 - 72) * t);
		return `rgb(${r}, ${g}, ${b})`;
	}

	function metric(row: ScreenerRow): number {
		const v = row[window];
		return typeof v === 'number' ? v : 0;
	}

	// Sector panels are sized by their share of total market cap.
	function sectorFlex(sec: Sector): number {
		return Math.max(0.04, sec.total / grandTotal);
	}

	// Tiles within a sector are sized by their share of that sector.
	function tileFlex(sec: Sector, weight: number): number {
		return Math.max(0.02, weight / Math.max(1, sec.total));
	}
</script>

<svelte:head>
	<title>FINVIZ Elite+ · Map</title>
</svelte:head>

<header class="head">
	<div>
		<h2>Map</h2>
		<p>Market heatmap · tile size ∝ market cap · color ∝ performance.</p>
	</div>
	<div class="controls">
		<label>
			Performance
			<select bind:value={window}>
				{#each windowOptions as opt (opt.value)}
					<option value={opt.value}>{opt.label}</option>
				{/each}
			</select>
		</label>
		<div class="legend" aria-hidden="true">
			<span style="background: {colorFor(-5)}"></span>
			<span style="background: {colorFor(-2)}"></span>
			<span style="background: {colorFor(0)}"></span>
			<span style="background: {colorFor(2)}"></span>
			<span style="background: {colorFor(5)}"></span>
			<small>−5% &nbsp; → &nbsp; +5%</small>
		</div>
	</div>
</header>

{#if error}
	<p class="error" role="alert">⚠ {error}</p>
{/if}

{#if sectors.length}
	<div class="map">
		{#each sectors as sec (sec.name)}
			<section class="sector" style="flex-grow: {sectorFlex(sec)}">
				<h3 title={sec.name}>{sec.name} <span>{marketCap(sec.total)}</span></h3>
				<div class="tiles">
					{#each sec.tiles as t (t.row.symbol)}
						<a
							class="tile"
							href={`/symbol/${encodeURIComponent(t.row.symbol)}`}
							style="flex-grow: {tileFlex(sec, t.weight)}; background: {colorFor(metric(t.row))}"
							title={`${t.row.symbol} · ${t.row.name} · ${percent(metric(t.row))}`}
						>
							<span class="sym">{t.row.symbol}</span>
							<span class="pct">{percent(metric(t.row))}</span>
						</a>
					{/each}
				</div>
			</section>
		{/each}
	</div>
{:else}
	<p class="empty">No rows to map.</p>
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
	.controls {
		display: flex;
		gap: 1.25rem;
		flex-wrap: wrap;
		align-items: flex-end;
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
	.legend {
		display: flex;
		align-items: center;
		gap: 0;
	}
	.legend span {
		width: 22px;
		height: 14px;
		display: inline-block;
	}
	.legend span:first-child {
		border-radius: 3px 0 0 3px;
	}
	.legend small {
		margin-left: 0.5rem;
		color: var(--muted);
		font-size: 0.7rem;
		white-space: nowrap;
	}
	.map {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		align-content: flex-start;
	}
	.sector {
		flex-basis: 240px;
		min-width: 180px;
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 0.5rem;
		display: flex;
		flex-direction: column;
	}
	.sector h3 {
		margin: 0 0 0.4rem;
		font-size: 0.78rem;
		color: var(--muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		display: flex;
		justify-content: space-between;
		gap: 0.5rem;
		overflow: hidden;
		white-space: nowrap;
	}
	.sector h3 span {
		font-weight: 400;
		text-transform: none;
		letter-spacing: 0;
	}
	.tiles {
		display: flex;
		flex-wrap: wrap;
		gap: 2px;
		flex: 1;
		min-height: 120px;
	}
	.tile {
		flex-basis: 0;
		min-width: 44px;
		min-height: 38px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-decoration: none;
		color: #fff;
		border-radius: 3px;
		padding: 2px;
		overflow: hidden;
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.45);
	}
	.tile:hover {
		outline: 2px solid #fff;
		outline-offset: -2px;
	}
	.tile .sym {
		font-size: 0.72rem;
		font-weight: 700;
		font-family: var(--mono);
		white-space: nowrap;
	}
	.tile .pct {
		font-size: 0.64rem;
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
