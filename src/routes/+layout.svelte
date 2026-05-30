<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';

	let { children } = $props();

	const links = [
		{ href: '/', label: 'Screener' },
		{ href: '/portfolio', label: 'Portfolio' },
		{ href: '/watchlists', label: 'Watchlists' }
	];

	function isActive(href: string): boolean {
		return href === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(href);
	}
</script>

<div class="shell">
	<header>
		<div class="brand">
			<span class="logo">▦</span>
			<div>
				<h1>FINVIZ <span>Elite+</span></h1>
				<p>Rust + Axum · SvelteKit screener</p>
			</div>
		</div>
		<nav>
			{#each links as link (link.href)}
				<a href={link.href} class:active={isActive(link.href)}>{link.label}</a>
			{/each}
		</nav>
	</header>

	<main>
		{@render children()}
	</main>

	<footer>
		Synthetic demo data · powered by the <code>finviz-api</code> Rust backend
	</footer>
</div>

<style>
	.shell {
		max-width: 1280px;
		margin: 0 auto;
		padding: 0 1.25rem 3rem;
	}
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1.25rem 0;
		border-bottom: 1px solid var(--border);
		margin-bottom: 1.5rem;
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 0.85rem;
	}
	.logo {
		font-size: 1.9rem;
		color: var(--accent);
		line-height: 1;
	}
	h1 {
		margin: 0;
		font-size: 1.25rem;
		letter-spacing: 0.5px;
	}
	h1 span {
		color: var(--accent-2);
	}
	.brand p {
		margin: 0.15rem 0 0;
		font-size: 0.75rem;
		color: var(--muted);
	}
	nav a {
		text-decoration: none;
		color: var(--muted);
		font-weight: 600;
		font-size: 0.9rem;
		padding: 0.4rem 0.75rem;
		border-radius: var(--radius);
	}
	nav a.active {
		color: var(--text);
		background: var(--panel-2);
	}
	footer {
		margin-top: 2.5rem;
		padding-top: 1rem;
		border-top: 1px solid var(--border);
		color: var(--muted);
		font-size: 0.8rem;
	}
	footer code {
		font-family: var(--mono);
		color: var(--accent);
	}
</style>
