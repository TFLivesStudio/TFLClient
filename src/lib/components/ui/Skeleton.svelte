<script lang="ts">
	import { appearance } from '$lib/state/appearanceState.svelte';

	/** Bloque de carga — Entrega A. El shimmer animado solo corre cuando
	 * tiene sentido visualmente y en rendimiento: ni en Lite ni con blur
	 * desactivado (`appearance.noBlur`, perfil Lite también lo activa) —
	 * condición exacta pedida en la sección 3/5 del encargo para este
	 * componente. Sin shimmer, el bloque queda como un relleno estático
	 * (sigue comunicando "esto está cargando" por su forma, solo sin
	 * movimiento). */
	let {
		variant = 'rect',
		width = '100%',
		height,
		radius
	}: {
		variant?: 'rect' | 'circle' | 'text';
		width?: string | number;
		height?: string | number;
		radius?: string;
	} = $props();

	const shimmer = $derived(!appearance.noBlur && appearance.quality !== 'lite');

	function toCss(v: string | number | undefined): string | undefined {
		if (v === undefined) return undefined;
		return typeof v === 'number' ? `${v}px` : v;
	}

	const resolvedWidth = $derived(toCss(width) ?? '100%');
	const resolvedHeight = $derived(
		toCss(height) ?? (variant === 'text' ? '0.9em' : variant === 'circle' ? resolvedWidth : '16px')
	);
	const resolvedRadius = $derived(
		radius ??
			(variant === 'circle'
				? '50%'
				: variant === 'text'
					? 'var(--border-radius-sm)'
					: 'var(--border-radius-sm)')
	);
</script>

<span
	class="tfl-skeleton"
	class:shimmer
	style={`width: ${resolvedWidth}; height: ${resolvedHeight}; border-radius: ${resolvedRadius};`}
	aria-hidden="true"
></span>

<style>
	.tfl-skeleton {
		display: inline-block;
		background: var(--border);
	}

	.tfl-skeleton.shimmer {
		background: linear-gradient(
			90deg,
			var(--border) 25%,
			color-mix(in srgb, var(--accent) 22%, var(--border)) 50%,
			var(--border) 75%
		);
		background-size: 200% 100%;
		animation: tfl-skeleton-shimmer 1.6s ease-in-out infinite;
	}

	@keyframes tfl-skeleton-shimmer {
		0% {
			background-position: 200% 0;
		}
		100% {
			background-position: -200% 0;
		}
	}
</style>
