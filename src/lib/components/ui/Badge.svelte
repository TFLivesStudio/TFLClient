<script lang="ts">
	import type { Snippet } from 'svelte';

	/** Badge semántico — Entrega A. success/error/warning usan los tokens
	 * `--color-*` existentes; no hay un `--color-info` en global.css todavía,
	 * así que "info" usa `--accent` (es, de hecho, el color de marca/acento
	 * de la app — un uso razonable como "informativo" sin inventar un token
	 * de color nuevo fuera de lo pedido en la sección 1). "neutral" usa la
	 * escala de texto/borde neutra. */
	let {
		variant = 'neutral',
		children
	}: {
		variant?: 'success' | 'error' | 'warning' | 'info' | 'neutral';
		children: Snippet;
	} = $props();
</script>

<span class="tfl-badge {variant}">{@render children()}</span>

<style>
	.tfl-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: 999px;
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		font-weight: 600;
		white-space: nowrap;
	}

	.tfl-badge.success {
		background: color-mix(in srgb, var(--color-success) 16%, transparent);
		color: var(--color-success);
	}
	.tfl-badge.error {
		background: rgba(var(--color-error-rgb), 0.16);
		color: var(--color-error);
	}
	.tfl-badge.warning {
		background: color-mix(in srgb, var(--color-warning) 18%, transparent);
		color: var(--color-warning);
	}
	.tfl-badge.info {
		background: color-mix(in srgb, var(--accent) 16%, transparent);
		color: var(--accent);
	}
	.tfl-badge.neutral {
		background: var(--surface-interactive);
		color: var(--text-secondary);
	}
</style>
