<script lang="ts">
	import type { Snippet } from 'svelte';
	import { AlertTriangle } from 'lucide-svelte';
	import { t } from '$lib/i18n/index.svelte';
	import Button from './Button.svelte';

	/** Estado de error compartido — Entrega A. Mismo layout que
	 * `EmptyState` con semántica de error (ícono/color de `--color-error`)
	 * y una acción de reintentar opcional: si se pasa `onRetry` sin
	 * `retryLabel`, usa `t('ui.retry')` en vez de un string hardcodeado. */
	let {
		icon,
		title,
		description,
		onRetry,
		retryLabel
	}: {
		icon?: Snippet;
		title: string;
		description?: string;
		onRetry?: () => void;
		retryLabel?: string;
	} = $props();
</script>

<div class="tfl-error-state">
	<div class="tfl-error-icon">
		{#if icon}
			{@render icon()}
		{:else}
			<AlertTriangle size={28} />
		{/if}
	</div>
	<p class="tfl-error-title">{title}</p>
	{#if description}
		<p class="tfl-error-description">{description}</p>
	{/if}
	{#if onRetry}
		<div class="tfl-error-action">
			<Button variant="secondary" onclick={onRetry}>{retryLabel ?? t('ui.retry')}</Button>
		</div>
	{/if}
</div>

<style>
	.tfl-error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 40px 24px;
		text-align: center;
	}

	.tfl-error-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 6px;
		color: var(--color-error);
	}

	.tfl-error-title {
		font-size: var(--text-subtitle);
		line-height: var(--line-height-subtitle);
		font-weight: 700;
		color: var(--text-primary);
	}

	.tfl-error-description {
		max-width: 360px;
		font-size: var(--text-small);
		line-height: var(--line-height-small);
		color: var(--text-secondary);
	}

	.tfl-error-action {
		margin-top: 10px;
	}
</style>
