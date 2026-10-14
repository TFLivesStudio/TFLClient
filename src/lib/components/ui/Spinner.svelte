<script lang="ts">
	import { Loader2 } from 'lucide-svelte';
	import { appearance } from '$lib/state/appearanceState.svelte';

	/** Spinner compartido — Entrega A, sección 5.1 del encargo: con
	 * reduceMotion activo (siempre cierto en perfil Lite, ver
	 * appearanceState) muestra el ícono QUIETO en vez de girando — una
	 * representación estática en vez de solo depender de que el guard
	 * global de CSS aplaste la duración a ~0, que haría que gire a
	 * velocidad absurda en vez de quedarse fijo. */
	let {
		size = 'md',
		label
	}: {
		size?: 'sm' | 'md' | 'lg';
		/** Si se pasa, el spinner es accesible (`role="status"` + aria-label).
		 * Omitila cuando el spinner es puramente decorativo (ej. al lado de
		 * un label que ya dice "Cargando…"). */
		label?: string;
	} = $props();

	const px = $derived(
		size === 'sm'
			? 'var(--icon-size-sm)'
			: size === 'lg'
				? 'var(--icon-size-lg)'
				: 'var(--icon-size-md)'
	);
</script>

<span
	class="tfl-spinner"
	class:static={appearance.reduceMotion}
	style={`width: ${px}; height: ${px};`}
	role={label ? 'status' : 'presentation'}
	aria-label={label}
>
	<Loader2 size="100%" aria-hidden="true" />
</span>

<style>
	.tfl-spinner {
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.tfl-spinner :global(svg) {
		animation: tfl-spinner-spin 0.8s linear infinite;
	}

	.tfl-spinner.static :global(svg) {
		animation: none;
	}

	@keyframes tfl-spinner-spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
