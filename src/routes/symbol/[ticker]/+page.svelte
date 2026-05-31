<script lang="ts">
	import { onMount } from 'svelte';
	import { env } from '$env/dynamic/public';
	import Chart from '$lib/components/Chart.svelte';
	import {
		compactNumber,
		marketCap,
		optional,
		patternLabel,
		percent,
		price,
		shortDate,
		trend
	} from '$lib/format';
	import type { IndicatorPoint, OptionContract, QuoteTick } from '$lib/types';

	let { data } = $props();

	let live = $state<QuoteTick | null>(null);

	const livePrice = $derived(live?.price ?? data.quote.price);
	const liveChangePct = $derived(live?.change_pct ?? data.quote.change_pct);
	const liveChange = $derived(live?.change ?? data.quote.change);

	const closes = $derived(data.bars.map((b) => ({ ts: b.ts, value: b.close })));

	// Indicator overlay selection. Only offer what the backend actually returned.
	type Overlay = 'none' | 'sma' | 'ema' | 'bbands';
	const hasSma = $derived((data.sma?.points?.length ?? 0) > 0);
	const hasEma = $derived((data.ema?.points?.length ?? 0) > 0);
	const hasBbands = $derived((data.bbands?.length ?? 0) > 0);
	const hasRsi = $derived((data.rsi?.points?.length ?? 0) > 0);

	let overlay = $state<Overlay>('sma');

	const overlayOptions = $derived(
		[
			{ value: 'none' as Overlay, label: 'None', show: true },
			{ value: 'sma' as Overlay, label: `SMA(${data.sma?.period ?? 20})`, show: hasSma },
			{ value: 'ema' as Overlay, label: `EMA(${data.ema?.period ?? 20})`, show: hasEma },
			{ value: 'bbands' as Overlay, label: 'Bollinger', show: hasBbands }
		].filter((o) => o.show)
	);

	// The line overlaid on the price chart (single-line overlays).
	const overlayPoints = $derived<IndicatorPoint[]>(
		overlay === 'sma'
			? (data.sma?.points ?? [])
			: overlay === 'ema'
				? (data.ema?.points ?? [])
				: []
	);
	const overlayLabel = $derived(
		overlay === 'sma'
			? `SMA(${data.sma?.period ?? 20})`
			: overlay === 'ema'
				? `EMA(${data.ema?.period ?? 20})`
				: ''
	);

	// Bollinger bands rendered as three series sharing the price chart's domain.
	const bbMiddle = $derived<IndicatorPoint[]>(
		(data.bbands ?? []).map((p) => ({ ts: p.ts, value: p.middle }))
	);

	const rsiPoints = $derived<IndicatorPoint[]>(data.rsi?.points ?? []);
	const latestRsi = $derived(rsiPoints.length ? rsiPoints[rsiPoints.length - 1].value : null);

	// Detected chart patterns — best-effort enrichment, sorted by confidence desc.
	const patterns = $derived(
		[...(data.patterns ?? [])].sort((a, b) => (b.confidence ?? 0) - (a.confidence ?? 0))
	);

	function confidencePct(c: number): number {
		if (!Number.isFinite(c)) return 0;
		return Math.max(0, Math.min(100, Math.round(c * 100)));
	}

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

	// Enrichment tabs — only surface a tab when it has data.
	type Tab = 'news' | 'insider' | 'ratings' | 'options';
	const news = $derived(data.news ?? []);
	const insider = $derived(data.insider ?? []);
	const ratings = $derived(data.ratings ?? []);
	const chain = $derived(data.options ?? null);
	const hasNews = $derived(news.length > 0);
	const hasInsider = $derived(insider.length > 0);
	const hasRatings = $derived(ratings.length > 0);
	const hasOptions = $derived((chain?.expiries?.length ?? 0) > 0 && (chain?.contracts?.length ?? 0) > 0);

	const tabs = $derived(
		[
			{ value: 'news' as Tab, label: 'News', show: hasNews },
			{ value: 'insider' as Tab, label: 'Insider', show: hasInsider },
			{ value: 'ratings' as Tab, label: 'Ratings', show: hasRatings },
			{ value: 'options' as Tab, label: 'Options', show: hasOptions }
		].filter((t) => t.show)
	);

	let selectedTab = $state<Tab>('news');

	// ---- options chain --------------------------------------------------------
	let selectedExpiry = $state<string>('');

	// Default the expiry to the first available; reset when it's no longer valid.
	const expiry = $derived(
		chain?.expiries?.includes(selectedExpiry) ? selectedExpiry : (chain?.expiries?.[0] ?? '')
	);

	const underlying = $derived(chain?.underlying_price ?? livePrice);

	type StrikeRow = {
		strike: number;
		call: OptionContract | null;
		put: OptionContract | null;
		nearMoney: boolean;
	};

	// Build a per-strike row for the selected expiry, pairing calls and puts.
	const strikeRows = $derived.by<StrikeRow[]>(() => {
		if (!chain) return [];
		const rows = new Map<number, StrikeRow>();
		for (const c of chain.contracts) {
			if (c.expiry !== expiry) continue;
			let row = rows.get(c.strike);
			if (!row) {
				row = { strike: c.strike, call: null, put: null, nearMoney: false };
				rows.set(c.strike, row);
			}
			if (c.kind === 'call') row.call = c;
			else if (c.kind === 'put') row.put = c;
		}
		const list = [...rows.values()].sort((a, b) => a.strike - b.strike);
		// Mark the single strike closest to the underlying as near-the-money.
		if (list.length && Number.isFinite(underlying)) {
			let bestIdx = 0;
			let bestDiff = Infinity;
			for (let i = 0; i < list.length; i++) {
				const diff = Math.abs(list[i].strike - underlying);
				if (diff < bestDiff) {
					bestDiff = diff;
					bestIdx = i;
				}
			}
			list[bestIdx].nearMoney = true;
		}
		return list;
	});

	function ivPct(v: number): string {
		return Number.isFinite(v) ? `${(v * 100).toFixed(1)}%` : '—';
	}

	function num(v: number | null | undefined): string {
		return v != null && Number.isFinite(v) ? compactNumber(v) : '—';
	}

	function px(v: number | null | undefined): string {
		return v != null && Number.isFinite(v) ? price(v) : '—';
	}

	function deltaFmt(v: number | null | undefined): string {
		return v != null && Number.isFinite(v) ? v.toFixed(2) : '—';
	}

	// Keep the selection valid as availability changes.
	const activeTab = $derived(
		tabs.some((t) => t.value === selectedTab) ? selectedTab : (tabs[0]?.value ?? null)
	);

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

{#if overlayOptions.length > 1}
	<div class="indicators" role="group" aria-label="Chart overlays">
		{#each overlayOptions as opt (opt.value)}
			<button
				type="button"
				class:active={overlay === opt.value}
				onclick={() => (overlay = opt.value)}
			>
				{opt.label}
			</button>
		{/each}
	</div>
{/if}

<Chart
	series={closes}
	overlay={overlayPoints}
	overlayLabel={overlay === 'bbands' ? '' : overlayLabel}
	bands={overlay === 'bbands' && hasBbands
		? {
				upper: data.bbands.map((p) => ({ ts: p.ts, value: p.upper })),
				lower: data.bbands.map((p) => ({ ts: p.ts, value: p.lower }))
			}
		: null}
	label="Close"
/>

{#if overlay === 'bbands' && bbMiddle.length}
	<p class="caption">Bollinger middle = SMA basis; envelope shaded.</p>
{/if}

{#if patterns.length > 0}
	<section class="patterns" aria-label="Detected patterns">
		<h3>Detected patterns</h3>
		<ul class="pattern-list">
			{#each patterns as p, i (p.kind + '-' + p.start_ts + '-' + i)}
				{@const conf = confidencePct(p.confidence)}
				<li class="pattern">
					<div class="pattern-head">
						<span class="pattern-kind">{patternLabel(p.kind)}</span>
						<span class="pattern-conf" title="Confidence">
							<span class="conf-bar" aria-hidden="true">
								<span class="conf-fill" style="width: {conf}%"></span>
							</span>
							<span class="conf-pct">{conf}%</span>
						</span>
					</div>
					<span class="pattern-range">{shortDate(p.start_ts)} – {shortDate(p.end_ts)}</span>
					{#if p.description}
						<p class="pattern-desc">{p.description}</p>
					{/if}
				</li>
			{/each}
		</ul>
	</section>
{/if}

{#if hasRsi}
	<section class="rsi">
		<div class="rsi-head">
			<span class="rsi-title">RSI(14)</span>
			{#if latestRsi !== null}
				<span
					class="rsi-val"
					class:over={latestRsi >= 70}
					class:under={latestRsi <= 30}
				>
					{latestRsi.toFixed(1)}
				</span>
			{/if}
		</div>
		<Chart series={rsiPoints} height={90} label="RSI" />
	</section>
{/if}

<div class="stats">
	{#each stats as stat (stat.label)}
		<div class="stat">
			<span class="k">{stat.label}</span>
			<span class="v">{stat.value}</span>
		</div>
	{/each}
</div>

{#if tabs.length > 0}
	<section class="enrich">
		<div class="tabs" role="tablist" aria-label="Symbol detail">
			{#each tabs as tab (tab.value)}
				<button
					type="button"
					role="tab"
					aria-selected={activeTab === tab.value}
					class:active={activeTab === tab.value}
					onclick={() => (selectedTab = tab.value)}
				>
					{tab.label}
				</button>
			{/each}
		</div>

		{#if activeTab === 'news'}
			<ul class="news-list">
				{#each news as item (item.id)}
					<li>
						<time class="cell-date">{shortDate(item.ts)}</time>
						<div class="news-body">
							{#if item.url}
								<a href={item.url} target="_blank" rel="noopener noreferrer">{item.headline}</a>
							{:else}
								<span>{item.headline}</span>
							{/if}
							{#if item.source}<span class="news-source">{item.source}</span>{/if}
						</div>
					</li>
				{/each}
			</ul>
		{:else if activeTab === 'insider'}
			<div class="table-wrap">
				<table>
					<thead>
						<tr>
							<th>Date</th>
							<th>Insider</th>
							<th>Relation</th>
							<th>Type</th>
							<th class="num">Shares</th>
							<th class="num">Price</th>
							<th class="num">Value</th>
						</tr>
					</thead>
					<tbody>
						{#each insider as t, i (t.ts + '-' + i)}
							<tr>
								<td>{shortDate(t.ts)}</td>
								<td>{t.insider_name ?? '—'}</td>
								<td>{t.relation ?? '—'}</td>
								<td class:buy={t.transaction === 'Buy'} class:sell={t.transaction === 'Sell'}>
									{t.transaction ?? '—'}
								</td>
								<td class="num">{t.shares != null ? compactNumber(t.shares) : '—'}</td>
								<td class="num">{t.price != null ? price(t.price) : '—'}</td>
								<td class="num">{t.value != null ? `$${compactNumber(t.value)}` : '—'}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{:else if activeTab === 'ratings'}
			<div class="table-wrap">
				<table>
					<thead>
						<tr>
							<th>Date</th>
							<th>Firm</th>
							<th>Action</th>
							<th>Rating</th>
							<th class="num">Price Target</th>
						</tr>
					</thead>
					<tbody>
						{#each ratings as r, i (r.ts + '-' + i)}
							<tr>
								<td>{shortDate(r.ts)}</td>
								<td>{r.firm ?? '—'}</td>
								<td>{r.action ?? '—'}</td>
								<td>{r.rating ?? '—'}</td>
								<td class="num">{r.price_target != null ? price(r.price_target) : '—'}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{:else if activeTab === 'options' && chain}
			<div class="opt-controls">
				<label>
					Expiry
					<select value={expiry} onchange={(e) => (selectedExpiry = e.currentTarget.value)}>
						{#each chain.expiries as ex (ex)}
							<option value={ex}>{shortDate(Date.parse(ex) / 1000)}</option>
						{/each}
					</select>
				</label>
				<span class="opt-underlying">Underlying {px(underlying)}</span>
			</div>
			{#if strikeRows.length}
				<div class="table-wrap">
					<table class="opt-table">
						<thead>
							<tr>
								<th colspan="7" class="side-head call-head">Calls</th>
								<th class="strike-head" rowspan="2">Strike</th>
								<th colspan="7" class="side-head put-head">Puts</th>
							</tr>
							<tr>
								<th class="num">Bid</th>
								<th class="num">Ask</th>
								<th class="num">Last</th>
								<th class="num">Vol</th>
								<th class="num">OI</th>
								<th class="num">IV</th>
								<th class="num">Δ</th>
								<th class="num">Bid</th>
								<th class="num">Ask</th>
								<th class="num">Last</th>
								<th class="num">Vol</th>
								<th class="num">OI</th>
								<th class="num">IV</th>
								<th class="num">Δ</th>
							</tr>
						</thead>
						<tbody>
							{#each strikeRows as row (row.strike)}
								<tr class:atm={row.nearMoney}>
									<td class="num call-cell">{px(row.call?.bid)}</td>
									<td class="num call-cell">{px(row.call?.ask)}</td>
									<td class="num call-cell">{px(row.call?.last)}</td>
									<td class="num call-cell">{num(row.call?.volume)}</td>
									<td class="num call-cell">{num(row.call?.open_interest)}</td>
									<td class="num call-cell">{row.call ? ivPct(row.call.implied_vol) : '—'}</td>
									<td class="num call-cell">{deltaFmt(row.call?.delta)}</td>
									<td class="num strike-cell">{px(row.strike)}</td>
									<td class="num put-cell">{px(row.put?.bid)}</td>
									<td class="num put-cell">{px(row.put?.ask)}</td>
									<td class="num put-cell">{px(row.put?.last)}</td>
									<td class="num put-cell">{num(row.put?.volume)}</td>
									<td class="num put-cell">{num(row.put?.open_interest)}</td>
									<td class="num put-cell">{row.put ? ivPct(row.put.implied_vol) : '—'}</td>
									<td class="num put-cell">{deltaFmt(row.put?.delta)}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<p class="empty">No contracts for this expiry.</p>
			{/if}
		{/if}
	</section>
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
	.indicators {
		display: flex;
		gap: 0.4rem;
		margin-bottom: 0.6rem;
		flex-wrap: wrap;
	}
	.indicators button {
		background: var(--panel);
		border: 1px solid var(--border);
		color: var(--muted);
		border-radius: var(--radius);
		padding: 0.3rem 0.7rem;
		font-size: 0.8rem;
		font-weight: 600;
		cursor: pointer;
	}
	.indicators button:hover {
		color: var(--text);
		border-color: var(--accent);
	}
	.indicators button.active {
		color: var(--text);
		background: var(--panel-2);
		border-color: var(--accent);
	}
	.caption {
		margin: 0.4rem 0 0;
		color: var(--muted);
		font-size: 0.75rem;
	}
	.patterns {
		margin-top: 1.25rem;
	}
	.patterns h3 {
		margin: 0 0 0.6rem;
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--muted);
		text-transform: uppercase;
		letter-spacing: 0.4px;
	}
	.pattern-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 0.6rem;
	}
	.pattern {
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 0.7rem 0.8rem;
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}
	.pattern-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.6rem;
	}
	.pattern-kind {
		font-weight: 600;
		font-size: 0.9rem;
		color: var(--text);
	}
	.pattern-conf {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		flex: 0 0 auto;
	}
	.conf-bar {
		width: 48px;
		height: 5px;
		border-radius: 3px;
		background: var(--panel-2);
		overflow: hidden;
	}
	.conf-fill {
		display: block;
		height: 100%;
		background: var(--accent);
	}
	.conf-pct {
		font-size: 0.78rem;
		font-variant-numeric: tabular-nums;
		color: var(--muted);
	}
	.pattern-range {
		font-size: 0.75rem;
		color: var(--muted);
		font-variant-numeric: tabular-nums;
	}
	.pattern-desc {
		margin: 0;
		font-size: 0.82rem;
		line-height: 1.4;
		color: var(--text);
	}
	.rsi {
		margin-top: 1rem;
	}
	.rsi-head {
		display: flex;
		align-items: baseline;
		gap: 0.6rem;
		margin-bottom: 0.35rem;
	}
	.rsi-title {
		font-size: 0.8rem;
		color: var(--muted);
		font-weight: 600;
	}
	.rsi-val {
		font-size: 0.85rem;
		font-variant-numeric: tabular-nums;
		color: var(--text);
	}
	.rsi-val.over {
		color: var(--danger);
	}
	.rsi-val.under {
		color: var(--accent-2);
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
	.enrich {
		margin-top: 1.75rem;
	}
	.tabs {
		display: flex;
		gap: 0.4rem;
		margin-bottom: 1rem;
		flex-wrap: wrap;
		border-bottom: 1px solid var(--border);
		padding-bottom: 0.6rem;
	}
	.tabs button {
		background: var(--panel);
		border: 1px solid var(--border);
		color: var(--muted);
		border-radius: var(--radius);
		padding: 0.35rem 0.8rem;
		font-size: 0.85rem;
		font-weight: 600;
		cursor: pointer;
	}
	.tabs button:hover {
		color: var(--text);
		border-color: var(--accent);
	}
	.tabs button.active {
		color: var(--text);
		background: var(--panel-2);
		border-color: var(--accent);
	}
	.news-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
	}
	.news-list li {
		display: flex;
		gap: 1rem;
		padding: 0.6rem 0;
		border-bottom: 1px solid var(--border);
	}
	.cell-date {
		flex: 0 0 5.5rem;
		color: var(--muted);
		font-size: 0.78rem;
		font-variant-numeric: tabular-nums;
		padding-top: 0.1rem;
	}
	.news-body {
		flex: 1 1 auto;
		min-width: 0;
	}
	.news-body a,
	.news-body span:first-child {
		color: var(--text);
		text-decoration: none;
		font-weight: 600;
		font-size: 0.9rem;
		line-height: 1.35;
	}
	.news-body a:hover {
		color: var(--accent);
	}
	.news-source {
		display: block;
		margin-top: 0.2rem;
		color: var(--muted);
		font-size: 0.75rem;
		font-weight: 400;
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
	th.num,
	td.num {
		text-align: right;
	}
	td.buy {
		color: var(--accent-2);
		font-weight: 600;
	}
	td.sell {
		color: var(--danger);
		font-weight: 600;
	}
	.opt-controls {
		display: flex;
		align-items: flex-end;
		gap: 1.25rem;
		margin-bottom: 0.9rem;
		flex-wrap: wrap;
	}
	.opt-controls label {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		font-size: 0.75rem;
		color: var(--muted);
	}
	.opt-controls select {
		background: var(--panel);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.4rem 0.6rem;
		font-size: 0.85rem;
	}
	.opt-underlying {
		font-size: 0.8rem;
		color: var(--muted);
		font-variant-numeric: tabular-nums;
	}
	.opt-table {
		font-size: 0.78rem;
	}
	.opt-table th,
	.opt-table td {
		padding: 0.35rem 0.55rem;
	}
	.opt-table .side-head {
		text-align: center;
		font-size: 0.7rem;
		letter-spacing: 0.6px;
	}
	.opt-table .call-head {
		color: var(--accent-2);
	}
	.opt-table .put-head {
		color: var(--danger);
	}
	.opt-table .strike-head {
		text-align: center;
		background: var(--panel-2);
	}
	.opt-table .strike-cell {
		text-align: center;
		font-weight: 700;
		background: var(--panel-2);
		font-family: var(--mono);
	}
	.opt-table tr.atm {
		background: rgba(45, 211, 153, 0.1);
	}
	.opt-table tr.atm .strike-cell {
		background: rgba(45, 211, 153, 0.25);
		color: var(--accent-2);
	}
	.empty {
		text-align: center;
		color: var(--muted);
		padding: 1.5rem;
	}
</style>
