<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Loader2 } from 'lucide-svelte';

	/** Botón compartido — Entrega A (Visual Foundation). Usa los tokens de
	 * global.css (sección 1): tamaños de `--control-size-*`, tipografía de
	 * `--text-small`/`--text-body`, motion de `--duration-micro`. Ningún
	 * color/tamaño hardcodeado. */
	let {
		variant = 'primary',
		size = 'normal',
		type = 'button',
		disabled = false,
		loading = false,
		icon,
		children,
		onclick,
		...rest
	}: {
		variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
		size?: 'compact' | 'normal' | 'prominent';
		type?: 'button' | 'submit' | 'reset';
		disabled?: boolean;
		/** Spinner en vez del ícono/contenido mientras una acción corre; el
		 * botón queda deshabilitado mientras tanto. */
		loading?: boolean;
		/** Ícono opcional a la izquierda del label (ej. `{#snippet icon()}<Plus size={16} />{/snippet}`). */
		icon?: Snippet;
		children?: Snippet;
		onclick?: (e: MouseEvent) => void;
		[key: string]: unknown;
	} = $props();
</script>

<button
	{type}
	class="tfl-btn {variant} {size}"
	class:loading
	disabled={disabled || loading}
	aria-busy={loading}
	{onclick}
	{...rest}
>
	{#if loading}
		<Loader2 size={16} class="tfl-btn-spin" aria-hidden="true" />
	{:else if icon}
		{@render icon()}
	{/if}
	{#if children}
		<span class="tfl-btn-label">{@render children()}</span>
	{/if}
</button>

<style>
	.tfl-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		border: 1px solid transparent;
		border-radius: var(--border-radius-sm);
		font-family: inherit;
		font-weight: 600;
		cursor: pointer;
		white-space: nowrap;
		transition:
			background-color var(--duration-micro) var(--easing-standard),
			border-color var(--duration-micro) var(--easing-standard),
			color var(--duration-micro) var(--easing-standard),
			opacity var(--duration-micro) var(--easing-standard);
	}

	.tfl-btn:disabled {
		cursor: not-allowed;
		opacity: 0.55;
	}

	/* Tamaños — altura de `--control-size-*`, padding/tipografía acorde a
	   cada uno (compact usa la escala chica, prominent un poco más de aire
	   horizontal). */
	.tfl-btn.compact {
		height: var(--control-size-compact);
		padding: 0 12px;
		font-size: var(--text-small);
		line-height: var(--line-height-small);
	}
	.tfl-btn.normal {
		height: var(--control-size-normal);
		padding: 0 16px;
		font-size: var(--text-body);
		line-height: var(--line-height-body);
	}
	.tfl-btn.prominent {
		height: var(--control-size-prominent);
		padding: 0 22px;
		font-size: var(--text-body);
		line-height: var(--line-height-body);
	}

	/* Variantes de color. */
	.tfl-btn.primary {
		background: var(--accent);
		color: var(--accent-text);
	}
	.tfl-btn.primary:not(:disabled):hover {
		background: var(--accent-hover);
	}

	.tfl-btn.secondary {
		background: var(--surface-interactive);
		border-color: var(--border);
		color: var(--text-primary);
	}
	.tfl-btn.secondary:not(:disabled):hover {
		border-color: var(--accent);
	}

	.tfl-btn.ghost {
		background: transparent;
		color: var(--text-primary);
	}
	.tfl-btn.ghost:not(:disabled):hover {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
	}

	.tfl-btn.danger {
		background: var(--color-error);
		color: #ffffff;
	}
	.tfl-btn.danger:not(:disabled):hover {
		background: color-mix(in srgb, var(--color-error) 85%, white);
	}

	.tfl-btn-label {
		overflow: hidden;
		text-overflow: ellipsis;
	}

	:global(.tfl-btn-spin) {
		animation: tfl-btn-spin 0.8s linear infinite;
	}

	@keyframes tfl-btn-spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
