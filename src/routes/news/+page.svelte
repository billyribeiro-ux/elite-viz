<script lang="ts">
	import { relativeDate, shortDate } from '$lib/format';

	let { data } = $props();

	let selectedCategory = $state('All');

	const categories = $derived([
		'All',
		...Array.from(
			new Set(data.items.map((n) => n.category).filter((c): c is string => !!c))
		).sort()
	]);

	const filtered = $derived(
		selectedCategory === 'All'
			? data.items
			: data.items.filter((n) => n.category === selectedCategory)
	);
</script>

<svelte:head>
	<title>News · FINVIZ Elite+</title>
</svelte:head>

<header class="head">
	<h2>Market News</h2>
	<p>Latest headlines across the market feed.</p>
</header>

{#if data.error}
	<p class="notice error">Could not load news: {data.error}</p>
{/if}

{#if categories.length > 1}
	<div class="filters" role="group" aria-label="Filter by category">
		{#each categories as cat (cat)}
			<button
				type="button"
				class:active={selectedCategory === cat}
				onclick={() => (selectedCategory = cat)}
			>
				{cat}
			</button>
		{/each}
	</div>
{/if}

{#if filtered.length === 0}
	<p class="notice">
		{#if data.items.length === 0}
			No news available right now.
		{:else}
			No headlines in “{selectedCategory}”.
		{/if}
	</p>
{:else}
	<ul class="feed">
		{#each filtered as item (item.id)}
			<li class="row">
				<time class="date" title={shortDate(item.ts)}>{relativeDate(item.ts)}</time>
				<div class="body">
					<div class="meta">
						{#if item.category}<span class="chip">{item.category}</span>{/if}
						{#if item.source}<span class="source">{item.source}</span>{/if}
					</div>
					{#if item.symbol}
						<a class="headline" href="/symbol/{item.symbol}">{item.headline}</a>
						<a class="ticker" href="/symbol/{item.symbol}">{item.symbol}</a>
					{:else if item.url}
						<a class="headline" href={item.url} target="_blank" rel="noopener noreferrer">
							{item.headline}
						</a>
					{:else}
						<span class="headline">{item.headline}</span>
					{/if}
				</div>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.head {
		margin-bottom: 1.25rem;
	}
	.head h2 {
		margin: 0;
		font-size: 1.4rem;
	}
	.head p {
		margin: 0.25rem 0 0;
		color: var(--muted);
		font-size: 0.85rem;
	}
	.notice {
		color: var(--muted);
		font-size: 0.9rem;
		padding: 1rem 0;
	}
	.notice.error {
		color: var(--danger);
	}
	.filters {
		display: flex;
		gap: 0.4rem;
		margin-bottom: 1rem;
		flex-wrap: wrap;
	}
	.filters button {
		background: var(--panel);
		border: 1px solid var(--border);
		color: var(--muted);
		border-radius: var(--radius);
		padding: 0.3rem 0.7rem;
		font-size: 0.8rem;
		font-weight: 600;
		cursor: pointer;
	}
	.filters button:hover {
		color: var(--text);
		border-color: var(--accent);
	}
	.filters button.active {
		color: var(--text);
		background: var(--panel-2);
		border-color: var(--accent);
	}
	.feed {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
	}
	.row {
		display: flex;
		gap: 1rem;
		padding: 0.85rem 0;
		border-bottom: 1px solid var(--border);
	}
	.date {
		flex: 0 0 5.5rem;
		color: var(--muted);
		font-size: 0.78rem;
		font-variant-numeric: tabular-nums;
		padding-top: 0.1rem;
	}
	.body {
		flex: 1 1 auto;
		min-width: 0;
	}
	.meta {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 0.3rem;
		flex-wrap: wrap;
	}
	.chip {
		background: var(--panel-2);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 0.1rem 0.5rem;
		font-size: 0.7rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.4px;
		color: var(--accent);
	}
	.source {
		color: var(--muted);
		font-size: 0.75rem;
	}
	.headline {
		display: inline;
		color: var(--text);
		text-decoration: none;
		font-weight: 600;
		font-size: 0.95rem;
		line-height: 1.35;
	}
	a.headline:hover {
		color: var(--accent);
	}
	.ticker {
		margin-left: 0.5rem;
		color: var(--accent-2);
		text-decoration: none;
		font-family: var(--mono);
		font-size: 0.78rem;
		font-weight: 600;
	}
	.ticker:hover {
		text-decoration: underline;
	}
</style>
