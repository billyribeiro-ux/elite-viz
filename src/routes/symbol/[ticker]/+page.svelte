<script lang="ts">
	import { onMount } from 'svelte';
	import { env } from '$env/dynamic/public';
	import Chart from '$lib/components/Chart.svelte';
	import { compactNumber, marketCap, optional, percent, price, trend } from '$lib/format';
	import type { QuoteTick } from '$lib/types';

	let { data } = $props();

	let live = $state<QuoteTick | null>(null);

	const livePrice = $derived(live?.price ?? data.quote.price);
	const liveChangePct = $derived(live?.change_pct ?? data.quote.change_pct);
	const liveChange = $derived(live?.change ?? data.quote.change);

	const closes = $derived(data.bars.map((b) => ({ ts: b.ts, value: b.close })));

	const stats = $derived([
		{ label: 'Market Cap', value: marketCap(data.fundamentals.market_cap) },
		{ label: 'P/E', value: optional(data.fundamentals.pe) },
		{ label: 'EPS', value: optional(data.fundamentals.eps) },
		{ label: 'Beta', value: optional(data.fundamentals.beta) },
		{ label: 'Div Yield', value: data.fundamentals.dividend_yield === null ? '—' : `${data.fundamentals.dividend_yield.toFixed(2)}%` },
		{ label: 'Volume', value: compactNumber(data.quote.volume) },
		{ label: 'Day High', value: price(data.quote.day_high) },
		{ label: 'Day Low', value: price(data.quote.day_low) },
		{ label: 'Prev Close', value: price(data.quote.prev_close) }
	]);

	onMount(() => {
		const fallback = `${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.hostname}:8080/ws/quotes`;
		const base = env.PUBLIC_WS_URL || fallback;
		let socket: WebSocket | null = null;
		try {
			socket = new WebSocket(`${base}?symbols=${encodeURIComponent(data.symbol)}`);
			socket.onmessage = (event) => {
				const ticks = JSON.parse(event.data) as QuoteTick[];
				const match = ticks.find((t) => t.symbol === data.symbol);
				if (match) live = match;
			};
		} catch {
			// Live updates are best-effort; the static quote remains.
		}
		return () => socket?.close();
	});
</script>

<svelte:head>
	<title>{data.symbol} · FINVIZ Elite+</title>
</svelte:head>

<a class="back" href="/">← Screener</a>

<header class="quote">
	<div>
		<h2>{data.symbol} {#if live}<span class="dot" title="live">●</span>{/if}</h2>
		<p>{data.instrument?.name ?? ''}{data.instrument ? ` · ${data.instrument.sector}` : ''}</p>
	</div>
	<div class="price {trend(liveChange)}">
		<span class="big">{price(livePrice)}</span>
		<span class="chg">{percent(liveChangePct)} ({liveChange >= 0 ? '+' : ''}{liveChange.toFixed(2)})</span>
	</div>
</header>

<Chart series={closes} overlay={data.sma.points} label="Close" overlayLabel={`SMA(${data.sma.period})`} />

<div class="stats">
	{#each stats as stat (stat.label)}
		<div class="stat">
			<span class="k">{stat.label}</span>
			<span class="v">{stat.value}</span>
		</div>
	{/each}
</div>

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
	.quote {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		margin-bottom: 1.25rem;
		flex-wrap: wrap;
		gap: 1rem;
	}
	h2 {
		margin: 0;
		font-size: 1.6rem;
		font-family: var(--mono);
	}
	.dot {
		color: var(--accent-2);
		font-size: 0.7rem;
		vertical-align: middle;
		animation: pulse 1.4s infinite;
	}
	@keyframes pulse {
		50% {
			opacity: 0.3;
		}
	}
	.quote p {
		margin: 0.2rem 0 0;
		color: var(--muted);
		font-size: 0.85rem;
	}
	.price {
		text-align: right;
	}
	.price .big {
		font-size: 1.8rem;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}
	.price .chg {
		display: block;
		font-size: 0.9rem;
		margin-top: 0.2rem;
	}
	.price.up {
		color: var(--accent-2);
	}
	.price.down {
		color: var(--danger);
	}
	.stats {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
		gap: 0.75rem;
		margin-top: 1.5rem;
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
</style>
