<script lang="ts">
	import { untrack } from 'svelte';
	import { env } from '$env/dynamic/public';
	import {
		deleteSavedScreen,
		exportScreenerCsv,
		getSavedScreens,
		runScreen,
		saveScreen
	} from '$lib/api';
	import QueryBar from '$lib/components/QueryBar.svelte';
	import ResultsTable from '$lib/components/ResultsTable.svelte';
	import type { SavedScreen, ScreenerRow, SortOrder } from '$lib/types';

	let { data } = $props();

	// Seed local, user-mutable state once from the server-loaded snapshot.
	let query = $state(untrack(() => data.initial.query));
	let sort = $state('market_cap');
	let order = $state<SortOrder>('desc');
	let rows = $state<ScreenerRow[]>(untrack(() => data.initial.rows));
	let total = $state(untrack(() => data.initial.total));
	let matched = $state(untrack(() => data.initial.matched));
	let loading = $state(false);
	let error = $state<string | null>(null);
	let live = $state(false);

	// Saved screens.
	let saved = $state<SavedScreen[]>(untrack(() => data.saved));
	let exporting = $state(false);
	let saving = $state(false);
	let showSaveInput = $state(false);
	let newName = $state('');
	// Id of the saved screen currently loaded, to badge its match count.
	let activeSavedId = $state<string | null>(null);

	async function run() {
		loading = true;
		error = null;
		try {
			const res = await runScreen({ query, sort, order, limit: 100 });
			rows = res.rows;
			total = res.total;
			matched = res.matched;
			query = res.query;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	// Any manual edit / preset / sort change detaches from the saved screen.
	function clearActive() {
		activeSavedId = null;
	}

	async function refreshSaved() {
		try {
			saved = await getSavedScreens();
		} catch {
			/* keep existing list on transient failure */
		}
	}

	function loadSaved(s: SavedScreen) {
		query = s.query;
		if (s.sort) sort = s.sort;
		if (s.order) order = s.order;
		activeSavedId = s.id;
		run();
	}

	async function commitSave() {
		const name = newName.trim();
		if (!name) return;
		saving = true;
		error = null;
		try {
			const created = await saveScreen({ name, query, sort, order });
			newName = '';
			showSaveInput = false;
			await refreshSaved();
			activeSavedId = created.id;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	async function removeSaved(s: SavedScreen) {
		error = null;
		try {
			await deleteSavedScreen(s.id);
			if (activeSavedId === s.id) activeSavedId = null;
			await refreshSaved();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function doExport() {
		exporting = true;
		error = null;
		try {
			await exportScreenerCsv({ query, sort, order, limit: 100 });
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			exporting = false;
		}
	}

	function changeSort(field: string) {
		clearActive();
		if (sort === field) {
			order = order === 'asc' ? 'desc' : 'asc';
		} else {
			sort = field;
			order = 'desc';
		}
		run();
	}

	// QueryBar triggers this on submit / preset / sort-select; treat as a manual run.
	function manualRun() {
		clearActive();
		run();
	}

	// When live, stream sorted-by-change results from the backend. Re-subscribes
	// whenever the query changes; tears down on toggle-off or unmount.
	$effect(() => {
		if (!live) return;
		const q = query;
		const wsBase =
			env.PUBLIC_WS_URL?.replace('/ws/quotes', '/ws/screener-updates') ||
			`${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.hostname}:8080/ws/screener-updates`;
		const socket = new WebSocket(`${wsBase}?query=${encodeURIComponent(q)}&limit=100`);
		socket.onmessage = (event) => {
			const msg = JSON.parse(event.data);
			if (msg.error) {
				error = msg.error;
				return;
			}
			rows = msg.rows;
			total = msg.total;
			matched = msg.matched;
			sort = 'change_pct';
			order = 'desc';
		};
		return () => socket.close();
	});
</script>

<svelte:head>
	<title>FINVIZ Elite+ · Screener</title>
</svelte:head>

<QueryBar
	bind:query
	bind:sort
	bind:order
	{loading}
	presets={data.presets}
	fields={data.fields}
	onRun={manualRun}
/>

<section class="saved" aria-label="Saved screens">
	<div class="saved-list">
		<span class="saved-label">Saved</span>
		{#each saved as s (s.id)}
			{@const active = activeSavedId === s.id}
			<span class="chip" class:active>
				<button type="button" class="chip-load" title={s.query} onclick={() => loadSaved(s)}>
					{s.name}
					{#if active && matched > 0}
						<span class="chip-badge" title="{matched} matches">★ {matched}</span>
					{/if}
				</button>
				<button
					type="button"
					class="chip-del"
					aria-label="Delete saved screen {s.name}"
					title="Delete"
					onclick={() => removeSaved(s)}>✕</button
				>
			</span>
		{:else}
			<span class="saved-empty">No saved screens yet.</span>
		{/each}
	</div>
	<div class="saved-actions">
		{#if showSaveInput}
			<form
				class="save-form"
				onsubmit={(e) => {
					e.preventDefault();
					commitSave();
				}}
			>
				<input
					type="text"
					bind:value={newName}
					placeholder="Name this screen"
					aria-label="Saved screen name"
					autocomplete="off"
					spellcheck="false"
				/>
				<button type="submit" class="save-go" disabled={saving || !newName.trim()}>
					{saving ? 'Saving…' : 'Save'}
				</button>
				<button
					type="button"
					class="save-cancel"
					onclick={() => {
						showSaveInput = false;
						newName = '';
					}}>Cancel</button
				>
			</form>
		{:else}
			<button type="button" class="save-current" onclick={() => (showSaveInput = true)}>
				＋ Save current
			</button>
		{/if}
		<button type="button" class="export" onclick={doExport} disabled={exporting}>
			{exporting ? 'Exporting…' : '⭳ Export CSV'}
		</button>
	</div>
</section>

<div class="stats">
	<span><strong>{matched}</strong> matches</span>
	<span class="sep">/</span>
	<span>{total} symbols</span>
	<label class="live" class:on={live}>
		<input type="checkbox" bind:checked={live} />
		<span class="led"></span> Live
	</label>
	{#if query.trim()}
		<code>{query}</code>
	{/if}
</div>

{#if error}
	<p class="error" role="alert">⚠ {error}</p>
{/if}

<ResultsTable {rows} {sort} {order} onSort={changeSort} />

<style>
	.stats {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		margin-bottom: 0.85rem;
		color: var(--muted);
		font-size: 0.9rem;
	}
	.stats strong {
		color: var(--text);
	}
	.stats .sep {
		opacity: 0.5;
	}
	.stats code {
		font-family: var(--mono);
		font-size: 0.8rem;
		color: var(--accent);
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 0.15rem 0.5rem;
		margin-left: auto;
	}
	.live {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		font-size: 0.82rem;
		cursor: pointer;
		user-select: none;
	}
	.live .led {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--border);
	}
	.live.on .led {
		background: var(--accent-2);
		box-shadow: 0 0 6px var(--accent-2);
		animation: blink 1.2s infinite;
	}
	.live.on {
		color: var(--accent-2);
	}
	@keyframes blink {
		50% {
			opacity: 0.4;
		}
	}
	.error {
		background: rgba(248, 114, 114, 0.12);
		border: 1px solid var(--danger);
		color: var(--danger);
		border-radius: var(--radius);
		padding: 0.7rem 0.9rem;
		margin-bottom: 1rem;
	}

	.saved {
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-wrap: wrap;
		gap: 0.75rem;
		margin-bottom: 0.9rem;
	}
	.saved-list {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 0.4rem;
	}
	.saved-label {
		color: var(--muted);
		font-size: 0.78rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin-right: 0.2rem;
	}
	.saved-empty {
		color: var(--muted);
		font-size: 0.82rem;
		opacity: 0.7;
	}
	.chip {
		display: inline-flex;
		align-items: stretch;
		border: 1px solid var(--border);
		border-radius: 999px;
		overflow: hidden;
		background: var(--panel);
	}
	.chip.active {
		border-color: var(--accent);
	}
	.chip-load {
		background: transparent;
		border: none;
		color: var(--text);
		font-size: 0.8rem;
		padding: 0.3rem 0.7rem;
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
		cursor: pointer;
	}
	.chip-load:hover {
		color: var(--accent);
	}
	.chip-badge {
		color: var(--accent-2);
		font-size: 0.72rem;
		font-variant-numeric: tabular-nums;
	}
	.chip-del {
		background: transparent;
		border: none;
		border-left: 1px solid var(--border);
		color: var(--muted);
		font-size: 0.7rem;
		padding: 0 0.55rem;
		cursor: pointer;
	}
	.chip-del:hover {
		color: var(--danger);
	}
	.saved-actions {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
	.save-form {
		display: flex;
		align-items: center;
		gap: 0.4rem;
	}
	.save-form input {
		background: var(--bg);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.4rem 0.6rem;
		font-size: 0.82rem;
	}
	.save-current,
	.save-cancel {
		background: transparent;
		border: 1px solid var(--border);
		color: var(--muted);
		border-radius: 999px;
		padding: 0.35rem 0.8rem;
		font-size: 0.8rem;
		cursor: pointer;
	}
	.save-current:hover,
	.save-cancel:hover {
		color: var(--text);
		border-color: var(--accent);
	}
	.save-go,
	.export {
		background: var(--accent);
		border: none;
		color: #07121f;
		font-weight: 700;
		border-radius: 999px;
		padding: 0.4rem 0.9rem;
		font-size: 0.8rem;
		cursor: pointer;
	}
	.export {
		background: var(--panel-2);
		color: var(--text);
		border: 1px solid var(--border);
		font-weight: 600;
	}
	.export:hover:not(:disabled) {
		border-color: var(--accent);
		color: var(--accent);
	}
	.save-go:disabled,
	.export:disabled {
		opacity: 0.6;
		cursor: progress;
	}
</style>
