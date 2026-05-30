<script lang="ts">
	import type { IndicatorPoint } from '$lib/types';

	let {
		series,
		overlay = [],
		height = 260,
		label = 'Close',
		overlayLabel = ''
	}: {
		series: IndicatorPoint[];
		overlay?: IndicatorPoint[];
		height?: number;
		label?: string;
		overlayLabel?: string;
	} = $props();

	const W = 1000; // internal viewBox width; scales to container

	// Domain is computed from both series so the overlay shares the same axes.
	const bounds = $derived.by(() => {
		const all = [...series, ...overlay];
		const ts = all.map((p) => p.ts);
		const vals = all.map((p) => p.value);
		const tMin = Math.min(...ts);
		const tMax = Math.max(...ts);
		let vMin = Math.min(...vals);
		let vMax = Math.max(...vals);
		if (vMin === vMax) {
			vMin -= 1;
			vMax += 1;
		}
		const pad = (vMax - vMin) * 0.08;
		return { tMin, tMax, vMin: vMin - pad, vMax: vMax + pad };
	});

	function x(ts: number): number {
		const { tMin, tMax } = bounds;
		return tMax === tMin ? 0 : ((ts - tMin) / (tMax - tMin)) * W;
	}
	function y(value: number): number {
		const { vMin, vMax } = bounds;
		return height - ((value - vMin) / (vMax - vMin)) * height;
	}

	function path(points: IndicatorPoint[]): string {
		return points.map((p, i) => `${i === 0 ? 'M' : 'L'}${x(p.ts).toFixed(2)} ${y(p.value).toFixed(2)}`).join(' ');
	}

	const areaPath = $derived(
		series.length
			? `${path(series)} L${x(series[series.length - 1].ts).toFixed(2)} ${height} L${x(series[0].ts).toFixed(2)} ${height} Z`
			: ''
	);
</script>

<figure>
	<svg viewBox={`0 0 ${W} ${height}`} preserveAspectRatio="none" role="img" aria-label="{label} chart">
		<defs>
			<linearGradient id="fill" x1="0" x2="0" y1="0" y2="1">
				<stop offset="0%" stop-color="var(--accent)" stop-opacity="0.28" />
				<stop offset="100%" stop-color="var(--accent)" stop-opacity="0" />
			</linearGradient>
		</defs>
		{#if series.length}
			<path d={areaPath} fill="url(#fill)" stroke="none" />
			<path d={path(series)} fill="none" stroke="var(--accent)" stroke-width="2" vector-effect="non-scaling-stroke" />
		{/if}
		{#if overlay.length}
			<path
				d={path(overlay)}
				fill="none"
				stroke="var(--warn)"
				stroke-width="1.5"
				stroke-dasharray="4 3"
				vector-effect="non-scaling-stroke"
			/>
		{/if}
	</svg>
	<figcaption>
		<span class="key accent">— {label}</span>
		{#if overlay.length && overlayLabel}
			<span class="key warn">-- {overlayLabel}</span>
		{/if}
		<span class="range">{bounds.vMin.toFixed(2)} – {bounds.vMax.toFixed(2)}</span>
	</figcaption>
</figure>

<style>
	figure {
		margin: 0;
	}
	svg {
		width: 100%;
		height: auto;
		display: block;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: var(--radius);
	}
	figcaption {
		display: flex;
		gap: 1rem;
		align-items: center;
		margin-top: 0.5rem;
		font-size: 0.78rem;
		color: var(--muted);
	}
	.key.accent {
		color: var(--accent);
	}
	.key.warn {
		color: var(--warn);
	}
	.range {
		margin-left: auto;
		font-variant-numeric: tabular-nums;
	}
</style>
