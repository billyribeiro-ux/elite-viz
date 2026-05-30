<script lang="ts">
	import type { FieldInfo, Preset, SortOrder } from '$lib/types';

	let {
		query = $bindable(''),
		sort = $bindable('market_cap'),
		order = $bindable('desc' as SortOrder),
		presets,
		fields,
		loading = false,
		onRun
	}: {
		query: string;
		sort: string;
		order: SortOrder;
		presets: Preset[];
		fields: FieldInfo[];
		loading?: boolean;
		onRun: () => void;
	} = $props();

	function submit(event: Event) {
		event.preventDefault();
		onRun();
	}

	function applyPreset(preset: Preset) {
		query = preset.query;
		onRun();
	}

	function toggleOrder() {
		order = order === 'asc' ? 'desc' : 'asc';
		onRun();
	}
</script>

<section class="bar">
	<form onsubmit={submit}>
		<input
			type="text"
			bind:value={query}
			placeholder='price > 100 and pe < 30 and sector = "Technology"'
			spellcheck="false"
			autocomplete="off"
			aria-label="Screener filter expression"
		/>
		<div class="controls">
			<label>
				Sort
				<select bind:value={sort} onchange={onRun}>
					{#each fields as field (field.name)}
						<option value={field.name}>{field.name}</option>
					{/each}
				</select>
			</label>
			<button type="button" class="order" onclick={toggleOrder} title="Toggle sort order">
				{order === 'asc' ? '▲ asc' : '▼ desc'}
			</button>
			<button type="submit" class="run" disabled={loading}>
				{loading ? 'Running…' : 'Run screen'}
			</button>
		</div>
	</form>

	<div class="presets">
		{#each presets as preset (preset.id)}
			<button type="button" onclick={() => applyPreset(preset)} title={preset.query}>
				{preset.label}
			</button>
		{/each}
	</div>
</section>

<style>
	.bar {
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 1rem;
		margin-bottom: 1.25rem;
	}
	form {
		display: flex;
		gap: 0.75rem;
		flex-wrap: wrap;
		align-items: center;
	}
	input {
		flex: 1 1 360px;
		min-width: 0;
		background: var(--bg);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.65rem 0.8rem;
		font-family: var(--mono);
		font-size: 0.9rem;
	}
	.controls {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}
	label {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		color: var(--muted);
		font-size: 0.8rem;
	}
	select,
	.order {
		background: var(--panel-2);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: var(--radius);
		padding: 0.5rem 0.6rem;
	}
	.run {
		background: var(--accent);
		border: none;
		color: #07121f;
		font-weight: 700;
		border-radius: var(--radius);
		padding: 0.55rem 1rem;
	}
	.run:disabled {
		opacity: 0.6;
		cursor: progress;
	}
	.presets {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
		margin-top: 0.85rem;
	}
	.presets button {
		background: transparent;
		border: 1px solid var(--border);
		color: var(--muted);
		border-radius: 999px;
		padding: 0.35rem 0.8rem;
		font-size: 0.8rem;
	}
	.presets button:hover {
		color: var(--text);
		border-color: var(--accent);
	}
</style>
