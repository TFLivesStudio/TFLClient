<script lang="ts">
	import { appearance } from '$lib/state/appearanceState.svelte';

	/** Barra de progreso compartida — Entrega A. `value` (0-100) para
	 * progreso determinado, `indeterminate` para desconocido. En perfil
	 * Lite (o con reduceMotion) la indeterminada se muestra como un bloque
	 * ESTÁTICO en vez de barrido infinito (sección 5.1 del encargo:
	 * preferir representación estática cuando sea viable), en vez de
	 * depender solo de que el guard global de CSS aplaste la duración. */
	let {
		value = 0,
		indeterminate = false
	}: {
		value?: number;
		indeterminate?: boolean;
	} = $props();

	const clamped = $derived(Math.min(100, Math.max(0, value)));
	const animated = $derived(
		indeterminate && appearance.quality !== 'lite' && !appearance.reduceMotion
	);
</script>

<div
	class="tfl-progress"
	role="progressbar"
	aria-valuemin="0"
	aria-valuemax="100"
	aria-valuenow={indeterminate ? undefined : clamped}
>
	<div class="tfl-progress-track">
		{#if indeterminate}
			<div class="tfl-progress-fill indeterminate" class:animated></div>
		{:else}
			<div class="tfl-progress-fill" style={`width: ${clamped}%`}></div>
		{/if}
	</div>
</div>

<style>
	.tfl-progress-track {
		height: 5px;
		border-radius: 4px;
		background: var(--border);
		overflow: hidden;
	}

	.tfl-progress-fill {
		height: 100%;
		background: var(--accent);
		border-radius: 4px;
		transition: width var(--duration-panel) var(--easing-standard);
	}

	.tfl-progress-fill.indeterminate {
		width: 35%;
	}

	.tfl-progress-fill.indeterminate.animated {
		animation: tfl-progress-sweep 1.2s ease-in-out infinite;
	}

	@keyframes tfl-progress-sweep {
		0% {
			transform: translateX(-100%);
		}
		100% {
			transform: translateX(285%);
		}
	}
</style>
