<script lang="ts">
	import Chart from '$lib/components/Chart.svelte';
	import { price, percent, trend } from '$lib/format';
	import { runBacktest } from '$lib/api';
	import type {
		BacktestResult,
		BacktestStrategy,
		IndicatorPoint,
		RuleParam,
		RuleSpec,
		StrategyEntry
	} from '$lib/types';

	let { data } = $props();

	const rules = $derived<RuleSpec[]>(data.rules ?? []);

	let symbol = $state('AAPL');
	let selectedKind = $state('');
	// Param values keyed by rule param name. Reset whenever the kind changes.
	let paramValues = $state<Record<string, number | boolean>>({});
	let timeExit = $state<string>('');
	let stopLoss = $state<string>('');
	let limit = $state<string>('');

	let busy = $state(false);
	let error = $state<string | null>(null);
	let result = $state<BacktestResult | null>(null);

	const selectedRule = $derived<RuleSpec | undefined>(
		rules.find((r) => r.kind === selectedKind)
	);
	const params = $derived<RuleParam[]>(selectedRule?.params ?? []);

	// Default the selected kind to the first catalog entry once rules arrive.
	$effect(() => {
		if (!selectedKind && rules.length) {
			selectKind(rules[0].kind);
		}
	});

	function defaultFor(p: RuleParam): number | boolean {
		if (p.type === 'bool') return typeof p.default === 'boolean' ? p.default : false;
		return typeof p.default === 'number' ? p.default : 0;
	}

	function selectKind(kind: string): void {
		selectedKind = kind;
		const rule = rules.find((r) => r.kind === kind);
		const next: Record<string, number | boolean> = {};
		for (const p of rule?.params ?? []) {
			next[p.name] = defaultFor(p);
		}
		paramValues = next;
	}

	function onKindChange(event: Event): void {
		selectKind((event.currentTarget as HTMLSelectElement).value);
	}

	function paramLabel(p: RuleParam): string {
		return p.label ?? p.name.replace(/_/g, ' ');
	}

	const equitySeries = $derived<IndicatorPoint[]>(
		(result?.equity_curve ?? []).map((p) => ({ ts: p.ts, value: p.equity }))
	);

	/** Epoch-seconds → YYYY-MM-DD. */
	function fmtDate(ts: number): string {
		if (!Number.isFinite(ts)) return '—';
		const d = new Date(ts * 1000);
		return Number.isNaN(d.getTime()) ? '—' : d.toISOString().slice(0, 10);
	}

	function num(v: unknown): number {
		return typeof v === 'number' && Number.isFinite(v) ? v : 0;
	}

	function buildStrategy(): BacktestStrategy {
		const entry: StrategyEntry = { kind: selectedKind };
		for (const p of params) {
			const val = paramValues[p.name];
			entry[p.name] = val ?? defaultFor(p);
		}
		const strategy: BacktestStrategy = { entry };
		const te = Number.parseInt(timeExit, 10);
		if (Number.isFinite(te) && te > 0) strategy.time_exit = te;
		const sl = Number.parseFloat(stopLoss);
		if (Number.isFinite(sl) && sl > 0) strategy.stop_loss_pct = sl;
		return strategy;
	}

	async function run(event: Event): Promise<void> {
		event.preventDefault();
		if (!symbol.trim() || !selectedKind) return;
		busy = true;
		error = null;
		try {
			const lim = Number.parseInt(limit, 10);
			result = await runBacktest({
				symbol: symbol.trim().toUpperCase(),
				strategy: buildStrategy(),
				limit: Number.isFinite(lim) && lim > 0 ? lim : undefined
			});
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			result = null;
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head>
	<title>Backtest · FINVIZ Elite+</title>
</svelte:head>

<h2>Backtest</h2>
<p class="lead">
	Build an entry rule, set optional exits, and replay it over historical bars to
	measure returns, drawdown, and risk-adjusted performance.
</p>

{#if data.rulesError}
	<p class="error" role="alert">⚠ Could not load rule catalog: {data.rulesError}</p>
{/if}

<form class="builder" onsubmit={run}>
	<div class="field">
		<label for="bt-symbol">Symbol</label>
		<input
			id="bt-symbol"
			class="sym"
			bind:value={symbol}
			placeholder="AAPL"
			spellcheck="false"
			autocomplete="off"
		/>
	</div>

	<div class="field">
		<label for="bt-kind">Entry rule</label>
		<select id="bt-kind" value={selectedKind} onchange={onKindChange} disabled={!rules.length}>
			{#if !rules.length}
				<option value="">No rules available</option>
			{/if}
			{#each rules as rule (rule.kind)}
				<option value={rule.kind}>{rule.label ?? rule.kind}</option>
			{/each}
		</select>
	</div>

	{#each params as p (p.name)}
		<div class="field">
			<label for="bt-param-{p.name}">{paramLabel(p)}</label>
			{#if p.type === 'bool'}
				<label class="check">
					<input
						id="bt-param-{p.name}"
						type="checkbox"
						checked={paramValues[p.name] === true}
						onchange={(e) => (paramValues[p.name] = e.currentTarget.checked)}
					/>
					<span>{paramValues[p.name] === true ? 'true' : 'false'}</span>
				</label>
			{:else}
				<input
					id="bt-param-{p.name}"
					type="number"
					step={p.type === 'int' ? '1' : 'any'}
					value={num(paramValues[p.name])}
					oninput={(e) =>
						(paramValues[p.name] =
							p.type === 'int'
								? Math.trunc(e.currentTarget.valueAsNumber)
								: e.currentTarget.valueAsNumber)}
				/>
			{/if}
		</div>
	{/each}

	<div class="field">
		<label for="bt-time-exit">Time exit (bars)</label>
		<input id="bt-time-exit" type="number" min="0" step="1" bind:value={timeExit} placeholder="optional" />
	</div>

	<div class="field">
		<label for="bt-stop-loss">Stop loss (%)</label>
		<input id="bt-stop-loss" type="number" min="0" step="any" bind:value={stopLoss} placeholder="optional" />
	</div>

	<div class="field">
		<label for="bt-limit">Bars limit</label>
		<input id="bt-limit" type="number" min="0" step="1" bind:value={limit} placeholder="optional" />
	</div>

	<div class="actions">
		<button type="submit" disabled={busy || !symbol.trim() || !selectedKind}>
			{busy ? 'Running…' : 'Run backtest'}
		</button>
	</div>
</form>

{#if error}
	<p class="error" role="alert">⚠ {error}</p>
{/if}

{#if result}
	<div class="cards">
		<div class="card {trend(num(result.total_return_pct))}">
			<span class="k">Total Return</span>
			<span class="v">{percent(num(result.total_return_pct))}</span>
		</div>
		<div class="card {trend(num(result.avg_return_pct))}">
			<span class="k">Avg Return</span>
			<span class="v">{percent(num(result.avg_return_pct))}</span>
		</div>
		<div class="card">
			<span class="k">Win Rate</span>
			<span class="v">{(num(result.win_rate) * 100).toFixed(1)}%</span>
		</div>
		<div class="card down">
			<span class="k">Max Drawdown</span>
			<span class="v">{percent(-Math.abs(num(result.max_drawdown_pct)))}</span>
		</div>
		<div class="card {trend(num(result.sharpe))}">
			<span class="k">Sharpe</span>
			<span class="v">{num(result.sharpe).toFixed(2)}</span>
		</div>
		<div class="card {trend(num(result.calmar))}">
			<span class="k">Calmar</span>
			<span class="v">{num(result.calmar).toFixed(2)}</span>
		</div>
	</div>

	{#if equitySeries.length}
		<section class="block">
			<h3>Equity curve</h3>
			<Chart series={equitySeries} label="Equity" height={260} />
		</section>
	{/if}

	<section class="block">
		<h3>Trades <span class="count">({result.trades?.length ?? 0})</span></h3>
		<div class="wrap">
			<table>
				<thead>
					<tr>
						<th>Entry</th>
						<th>Exit</th>
						<th class="r">Entry Price</th>
						<th class="r">Exit Price</th>
						<th class="r">Return</th>
						<th class="r">Bars</th>
					</tr>
				</thead>
				<tbody>
					{#each result.trades ?? [] as t, i (i)}
						<tr>
							<td class="mono">{fmtDate(t.entry_ts)}</td>
							<td class="mono">{fmtDate(t.exit_ts)}</td>
							<td class="r">{price(num(t.entry_price))}</td>
							<td class="r">{price(num(t.exit_price))}</td>
							<td class="r {trend(num(t.return_pct))}">{percent(num(t.return_pct))}</td>
							<td class="r">{num(t.bars_held)}</td>
						</tr>
					{:else}
						<tr><td colspan="6" class="empty">No trades for this strategy.</td></tr>
					{/each}
				</tbody>
			</table>
		</div>
	</section>
{:else if !busy}
	<p class="hint">Configure a strategy above and run a backtest to see results.</p>
{/if}

<style>
	h2 {
		margin: 0 0 0.4rem;
	}
	.lead {
		color: var(--muted);
		margin: 0 0 1.5rem;
		font-size: 0.9rem;
		max-width: 60ch;
	}
	.builder {
		display: flex;
		flex-wrap: wrap;
		gap: 0.9rem;
		align-items: flex-end;
		padding: 1rem;
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		margin-bottom: 1.25rem;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}
	.field label {
		color: var(--muted);
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: capitalize;
	}
	.field input,
	.field select {
		background: var(--bg);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.55rem 0.7rem;
		font: inherit;
		min-width: 130px;
	}
	.field input[type='number'] {
		width: 130px;
	}
	.field .sym {
		font-family: var(--mono);
		text-transform: uppercase;
	}
	.check {
		flex-direction: row;
		align-items: center;
		gap: 0.45rem;
		text-transform: none;
		color: var(--text);
		font-size: 0.85rem;
		padding: 0.3rem 0;
	}
	.check input {
		min-width: 0;
		width: auto;
	}
	.actions {
		margin-left: auto;
	}
	.actions button {
		background: var(--accent);
		color: #07121f;
		border: none;
		font-weight: 700;
		border-radius: var(--radius);
		padding: 0.6rem 1.3rem;
		cursor: pointer;
	}
	.actions button:disabled {
		opacity: 0.5;
		cursor: default;
	}
	.cards {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
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
	.block {
		margin-bottom: 1.75rem;
	}
	.block h3 {
		margin: 0 0 0.75rem;
		font-size: 1rem;
	}
	.block h3 .count {
		color: var(--muted);
		font-weight: 400;
		font-size: 0.85rem;
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
	.mono {
		font-family: var(--mono);
	}
	td.up {
		color: var(--accent-2);
	}
	td.down {
		color: var(--danger);
	}
	.empty {
		text-align: center;
		color: var(--muted);
		padding: 2rem;
	}
	.hint {
		color: var(--muted);
		font-size: 0.9rem;
	}
	.error {
		color: var(--danger);
		margin-bottom: 1rem;
	}
</style>
