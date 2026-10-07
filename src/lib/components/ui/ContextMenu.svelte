<script lang="ts">
	import { onMount } from 'svelte';

	export interface ContextMenuItem {
		id: string;
		label: string;
		icon?: typeof import('lucide-svelte').Boxes;
		danger?: boolean;
		/** Devuelve una promesa: el ítem muestra un spinner mientras corre y
		 * "✓" un momento al terminar, sin cerrar el menú de golpe — pensado
		 * para acciones de fondo (crear acceso directo, exportar) que no
		 * tienen otro lugar donde mostrar su resultado. El valor resuelto
		 * (si lo hay) no se usa, así que cualquier promesa sirve. */
		run: () => unknown;
	}

	let {
		x,
		y,
		items,
		onClose
	}: {
		x: number;
		y: number;
		items: ContextMenuItem[];
		onClose: () => void;
	} = $props();

	let busy = $state<string | null>(null);
	let done = $state<string | null>(null);
	let menuEl = $state<HTMLDivElement | undefined>(undefined);

	// Si el menú se abrió muy cerca del borde de la ventana, se corrige la
	// posición después de montar (ya con el tamaño real) para que no quede
	// cortado — un menú contextual que se sale de la pantalla es inusable.
	let adjustedX = $state(x);
	let adjustedY = $state(y);
	onMount(() => {
		if (!menuEl) return;
		const rect = menuEl.getBoundingClientRect();
		const margin = 8;
		if (x + rect.width > window.innerWidth - margin) {
			adjustedX = Math.max(margin, window.innerWidth - rect.width - margin);
		}
		if (y + rect.height > window.innerHeight - margin) {
			adjustedY = Math.max(margin, window.innerHeight - rect.height - margin);
		}
	});

	async function runItem(item: ContextMenuItem) {
		if (busy) return;
		busy = item.id;
		try {
			await item.run();
			done = item.id;
			setTimeout(onClose, 500);
		} catch {
			// El error real ya lo maneja/loguea la acción en sí (lib/api
			// propaga el mensaje de Rust) — acá no hay dónde mostrarlo con
			// detalle, así que solo se cierra sin fingir que salió bien.
			onClose();
		} finally {
			busy = null;
		}
	}
</script>

<svelte:window
	onkeydown={(e) => e.key === 'Escape' && onClose()}
	onclick={onClose}
	oncontextmenu={(e) => {
		// Si el "contextmenu" nativo del click que ABRIÓ este menú termina
		// disparando igual (además del mousedown que ya lo abrió, ver
		// Sidebar.svelte), el handler que lo originó ya llamó
		// preventDefault() — si se cierra acá también, el menú aparece y
		// se cierra en el mismo gesto. Solo cerrar si es un click derecho
		// en otro lado (no manejado por nadie más).
		if (e.defaultPrevented) return;
		e.preventDefault();
		onClose();
	}}
/>

<div
	bind:this={menuEl}
	class="context-menu"
	style="left: {adjustedX}px; top: {adjustedY}px;"
	onclick={(e) => e.stopPropagation()}
	role="menu"
	tabindex="-1"
>
	{#each items as item (item.id)}
		<button
			type="button"
			class="menu-item"
			class:danger={item.danger}
			onclick={() => runItem(item)}
		>
			{#if item.icon}
				<item.icon size={14} />
			{/if}
			<span>{item.label}</span>
			{#if busy === item.id}
				<span class="menu-status">…</span>
			{:else if done === item.id}
				<span class="menu-status">✓</span>
			{/if}
		</button>
	{/each}
</div>

<style>
	.context-menu {
		position: fixed;
		z-index: 200;
		min-width: 200px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		box-shadow: var(--shadow-lg);
		padding: 4px;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.menu-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 7px 10px;
		border: none;
		background: transparent;
		color: var(--text-primary);
		font-size: 0.8rem;
		text-align: left;
		border-radius: var(--border-radius-sm);
		cursor: pointer;
	}

	.menu-item:hover {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
	}

	.menu-item.danger {
		color: var(--color-error);
	}

	.menu-status {
		margin-left: auto;
		color: var(--text-muted);
		font-size: 0.72rem;
	}
</style>
