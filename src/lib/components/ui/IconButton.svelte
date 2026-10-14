<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Loader2 } from 'lucide-svelte';

	/** Wrapper de un solo ícono — Entrega A. `label` es obligatoria porque
	 * un botón solo-ícono sin aria-label es invisible para lectores de
	 * pantalla; no hay fallback silencioso a propósito. El contenido entre
	 * tags es el ícono (lucide-svelte, ej. `<IconButton label="Cerrar"><X size={16} /></IconButton>`). */
	let {
		label,
		variant = 'ghost',
		size = 'normal',
		type = 'button',
		disabled = false,
		loading = false,
		onclick,
		children,
		...rest
	}: {
		label: string;
		variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
		size?: 'compact' | 'normal' | 'prominent';
		type?: 'button' | 'submit' | 'reset';
		disabled?: boolean;
		loading?: boolean;
		onclick?: (e: MouseEvent) => void;
		children: Snippet;
		[key: string]: unknown;
	} = $props();
</script>

<button
	{type}
	class="tfl-icon-btn {variant} {size}"
	aria-label={label}
	title={label}
	disabled={disabled || loading}
	aria-busy={loading}
	{onclick}
	{...rest}
>
	{#if loading}
		<Loader2 size={16} class="tfl-icon-btn-spin" aria-hidden="true" />
	{:else}
		{@render children()}
	{/if}
</button>

<style>
	.tfl-icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		border: 1px solid transparent;
		border-radius: var(--border-radius-sm);
		cursor: pointer;
		transition:
			background-color var(--duration-micro) var(--easing-standard),
			border-color var(--duration-micro) var(--easing-standard),
			color var(--duration-micro) var(--easing-standard),
			opacity var(--duration-micro) var(--easing-standard);
	}

	.tfl-icon-btn:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	.tfl-icon-btn.compact {
		width: var(--control-size-compact);
		height: var(--control-size-compact);
	}
	.tfl-icon-btn.normal {
		width: var(--control-size-normal);
		height: var(--control-size-normal);
	}
	.tfl-icon-btn.prominent {
		width: var(--control-size-prominent);
		height: var(--control-size-prominent);
	}

	.tfl-icon-btn.primary {
		background: var(--accent);
		color: var(--accent-text);
	}
	.tfl-icon-btn.primary:not(:disabled):hover {
		background: var(--accent-hover);
	}

	.tfl-icon-btn.secondary {
		background: var(--surface-interactive);
		border-color: var(--border);
		color: var(--text-primary);
	}
	.tfl-icon-btn.secondary:not(:disabled):hover {
		border-color: var(--accent);
	}

	.tfl-icon-btn.ghost {
		background: transparent;
		color: var(--text-secondary);
	}
	.tfl-icon-btn.ghost:not(:disabled):hover {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
		color: var(--text-primary);
	}

	.tfl-icon-btn.danger {
		background: transparent;
		color: var(--color-error);
	}
	.tfl-icon-btn.danger:not(:disabled):hover {
		background: rgba(var(--color-error-rgb), 0.15);
	}

	:global(.tfl-icon-btn-spin) {
		animation: tfl-icon-btn-spin 0.8s linear infinite;
	}

	@keyframes tfl-icon-btn-spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
