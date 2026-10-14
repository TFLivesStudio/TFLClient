<script lang="ts">
	import type { Snippet } from 'svelte';

	/** Contenido flotante anclado a un trigger — Entrega A. El trigger
	 * recibe `toggle` como parámetro de snippet, así que el caller decide
	 * qué elemento abre/cierra el popover (un botón, un ícono, lo que
	 * sea) sin que este componente le imponga markup. Cierra solo al
	 * click afuera (igual que ContextMenu) o Escape. */
	let {
		trigger,
		children,
		placement = 'bottom',
		open = $bindable(false)
	}: {
		trigger: Snippet<[{ toggle: () => void }]>;
		children: Snippet;
		placement?: 'top' | 'bottom' | 'left' | 'right';
		open?: boolean;
	} = $props();

	let wrapEl = $state<HTMLDivElement | undefined>(undefined);

	function toggle() {
		open = !open;
	}

	function handleOutsideClick(e: MouseEvent) {
		if (open && wrapEl && !wrapEl.contains(e.target as Node)) open = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') open = false;
	}
</script>

<svelte:window onclick={handleOutsideClick} onkeydown={handleKeydown} />

<div class="tfl-popover-wrap" bind:this={wrapEl}>
	{@render trigger({ toggle })}
	{#if open}
		<!-- Sin onclick/stopPropagation: el cierre por click-afuera ya lo
			 decide `handleOutsideClick` arriba comprobando
			 `wrapEl.contains(e.target)`, así que un click DENTRO de este
			 div nunca llega a cerrarlo — agregar stopPropagation acá
			 además cortaría el burbujeo de teclado hacia el handler de
			 Escape en :svelte:window para cualquier cosa enfocada dentro. -->
		<div class="tfl-popover-content pos-{placement}">
			{@render children()}
		</div>
	{/if}
</div>

<style>
	.tfl-popover-wrap {
		position: relative;
		display: inline-flex;
	}

	.tfl-popover-content {
		position: absolute;
		z-index: var(--z-dropdown);
		min-width: 200px;
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		box-shadow: var(--shadow-lg);
		padding: 10px;
		animation: tfl-popover-in var(--duration-micro) var(--easing-decelerate);
	}

	.tfl-popover-content.pos-top {
		bottom: calc(100% + 8px);
		left: 0;
	}
	.tfl-popover-content.pos-bottom {
		top: calc(100% + 8px);
		left: 0;
	}
	.tfl-popover-content.pos-left {
		right: calc(100% + 8px);
		top: 0;
	}
	.tfl-popover-content.pos-right {
		left: calc(100% + 8px);
		top: 0;
	}

	@keyframes tfl-popover-in {
		from {
			opacity: 0;
			transform: translateY(-2px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
