<script lang="ts">
	import type { Snippet } from 'svelte';

	/** Tooltip genérico posicionable — Entrega A. A diferencia de
	 * `InfoTooltip` (ícono de ayuda fijo, burbuja abajo), este envuelve
	 * CUALQUIER trigger vía `children` y soporta las 4 posiciones. No se
	 * tocó `InfoTooltip.svelte` en esta entrega (fuera del alcance pedido:
	 * "no toques ningún otro archivo fuera de lo descrito"); queda como
	 * trabajo de una entrega futura unificar su lógica de hover/focus con
	 * la de acá si se quiere deduplicar del todo. */
	let {
		content,
		position = 'top',
		disabled = false,
		children
	}: {
		content: string | Snippet;
		position?: 'top' | 'bottom' | 'left' | 'right';
		disabled?: boolean;
		children: Snippet;
	} = $props();

	let open = $state(false);
</script>

<span
	class="tfl-tooltip-wrap"
	role="group"
	onmouseenter={() => !disabled && (open = true)}
	onmouseleave={() => (open = false)}
	onfocusin={() => !disabled && (open = true)}
	onfocusout={() => (open = false)}
>
	{@render children()}
	{#if open && !disabled}
		<span class="tfl-tooltip-bubble pos-{position}" role="tooltip">
			{#if typeof content === 'string'}
				{content}
			{:else}
				{@render content()}
			{/if}
		</span>
	{/if}
</span>

<style>
	.tfl-tooltip-wrap {
		position: relative;
		display: inline-flex;
	}

	.tfl-tooltip-bubble {
		position: absolute;
		z-index: var(--z-tooltip);
		width: max-content;
		max-width: 260px;
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-card);
		box-shadow: var(--shadow-md);
		color: var(--text-secondary);
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		font-weight: 400;
		text-align: left;
		pointer-events: none;
		animation: tfl-tooltip-in var(--duration-micro) var(--easing-decelerate);
	}

	.tfl-tooltip-bubble.pos-top {
		bottom: calc(100% + 8px);
		left: 50%;
		transform: translateX(-50%);
	}
	.tfl-tooltip-bubble.pos-bottom {
		top: calc(100% + 8px);
		left: 50%;
		transform: translateX(-50%);
	}
	.tfl-tooltip-bubble.pos-left {
		right: calc(100% + 8px);
		top: 50%;
		transform: translateY(-50%);
	}
	.tfl-tooltip-bubble.pos-right {
		left: calc(100% + 8px);
		top: 50%;
		transform: translateY(-50%);
	}

	@keyframes tfl-tooltip-in {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}
</style>
