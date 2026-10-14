<script lang="ts">
	import type { Snippet } from 'svelte';

	/** Estado vacío compartido — Entrega A. `icon`/`action` son snippets
	 * opcionales para no imponer un set fijo de íconos/botones; el caller
	 * trae su propio ícono de lucide-svelte y, si quiere, un <Button>. */
	let {
		icon,
		title,
		description,
		action
	}: {
		icon?: Snippet;
		title: string;
		description?: string;
		action?: Snippet;
	} = $props();
</script>

<div class="tfl-empty-state">
	{#if icon}
		<div class="tfl-empty-icon">{@render icon()}</div>
	{/if}
	<p class="tfl-empty-title">{title}</p>
	{#if description}
		<p class="tfl-empty-description">{description}</p>
	{/if}
	{#if action}
		<div class="tfl-empty-action">{@render action()}</div>
	{/if}
</div>

<style>
	.tfl-empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 40px 24px;
		text-align: center;
	}

	.tfl-empty-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 6px;
		color: var(--text-muted);
	}

	.tfl-empty-title {
		font-size: var(--text-subtitle);
		line-height: var(--line-height-subtitle);
		font-weight: 700;
		color: var(--text-primary);
	}

	.tfl-empty-description {
		max-width: 360px;
		font-size: var(--text-small);
		line-height: var(--line-height-small);
		color: var(--text-secondary);
	}

	.tfl-empty-action {
		margin-top: 10px;
	}
</style>
