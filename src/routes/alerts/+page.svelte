<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { createAlert, deleteAlert } from '$lib/api';

	let { data } = $props();

	let symbol = $state('');
	let query = $state('');
	let note = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);

	async function add(event: Event) {
		event.preventDefault();
		busy = true;
		error = null;
		try {
			await createAlert({ symbol, query, note: note || undefined });
			symbol = '';
			query = '';
			note = '';
			await invalidateAll();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	async function remove(id: string) {
		busy = true;
		error = null;
		try {
			await deleteAlert(id);
			await invalidateAll();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head>
	<title>Alerts · FINVIZ Elite+</title>
</svelte:head>

<h2>Alerts</h2>
<p class="lead">
	An alert is a screener expression evaluated against one symbol — e.g.
	<code>price &gt; 250</code> or <code>change_pct &lt; -3</code>. Triggered alerts
	are highlighted.
</p>

<form class="create" onsubmit={add}>
	<input class="sym" bind:value={symbol} placeholder="Symbol" aria-label="Symbol" />
	<input class="q" bind:value={query} placeholder="Condition, e.g. price > 250" aria-label="Condition" spellcheck="false" />
	<input class="note" bind:value={note} placeholder="Note (optional)" aria-label="Note" />
	<button type="submit" disabled={busy || !symbol.trim() || !query.trim()}>Add</button>
</form>

{#if error}
	<p class="error" role="alert">⚠ {error}</p>
{/if}

<div class="wrap">
	<table>
		<thead>
			<tr>
				<th>Status</th>
				<th>Symbol</th>
				<th>Condition</th>
				<th>Note</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each data.alerts as a (a.id)}
				<tr class:triggered={a.triggered}>
					<td>
						<span class="badge" class:on={a.triggered}>{a.triggered ? 'TRIGGERED' : 'armed'}</span>
					</td>
					<td class="sym"><a href="/symbol/{a.symbol}">{a.symbol}</a></td>
					<td class="mono">{a.query}</td>
					<td class="muted">{a.note}</td>
					<td class="r"><button class="del" onclick={() => remove(a.id)} disabled={busy}>✕</button></td>
				</tr>
			{:else}
				<tr><td colspan="5" class="empty">No alerts yet.</td></tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	h2 {
		margin: 0 0 0.4rem;
	}
	.lead {
		color: var(--muted);
		margin: 0 0 1.5rem;
		font-size: 0.9rem;
	}
	.lead code {
		font-family: var(--mono);
		color: var(--accent);
	}
	.create {
		display: flex;
		gap: 0.6rem;
		flex-wrap: wrap;
		margin-bottom: 1.25rem;
	}
	.create input {
		background: var(--bg);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.6rem 0.75rem;
		font: inherit;
	}
	.create .sym {
		flex: 0 0 110px;
		font-family: var(--mono);
		text-transform: uppercase;
	}
	.create .q {
		flex: 1 1 240px;
		font-family: var(--mono);
	}
	.create .note {
		flex: 1 1 160px;
	}
	.create button {
		background: var(--accent);
		color: #07121f;
		border: none;
		font-weight: 700;
		border-radius: var(--radius);
		padding: 0.6rem 1.1rem;
	}
	.create button:disabled {
		opacity: 0.5;
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
		border-bottom: 1px solid var(--border);
		white-space: nowrap;
	}
	th {
		background: var(--panel-2);
		color: var(--muted);
		font-weight: 600;
	}
	tr.triggered {
		background: rgba(251, 189, 35, 0.08);
	}
	.badge {
		font-size: 0.72rem;
		font-weight: 700;
		color: var(--muted);
		border: 1px solid var(--border);
		border-radius: 999px;
		padding: 0.15rem 0.55rem;
	}
	.badge.on {
		color: var(--warn);
		border-color: var(--warn);
	}
	.sym a {
		color: var(--accent);
		font-weight: 700;
		font-family: var(--mono);
		text-decoration: none;
	}
	.mono {
		font-family: var(--mono);
	}
	.muted {
		color: var(--muted);
	}
	.r {
		text-align: right;
	}
	.del {
		background: none;
		border: none;
		color: var(--muted);
	}
	.del:hover {
		color: var(--danger);
	}
	.empty {
		text-align: center;
		color: var(--muted);
		padding: 2rem;
	}
	.error {
		color: var(--danger);
		margin-bottom: 1rem;
	}
</style>
