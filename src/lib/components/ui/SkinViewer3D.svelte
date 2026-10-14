<script lang="ts">
	import { SkinViewer } from 'skinview3d';
	import { appearance } from '$lib/state/appearanceState.svelte';

	/**
	 * Visor 3D de skin compartido — Entrega C (Home), sección 1/7 del
	 * encargo. Antes de esto, el mismo ciclo de vida de skinview3d (crear
	 * canvas, cargar textura, auto-rotate, dispose) estaba duplicado casi
	 * igual en WelcomeState.svelte y SkinManagerModal.svelte: misma
	 * librería, mismo patrón onMount/crear → onDestroy/dispose, solo con
	 * autoRotateSpeed/zoom levemente distintos entre los dos (0.6/0.9 acá,
	 * 0.8/0.85 allá). Este componente centraliza esa lógica una sola vez;
	 * cada caller sigue pasando su propio autoRotateSpeed/zoom/tamaño, así
	 * que ningún visual de los dos sitios cambia.
	 *
	 * Ciclo de vida (sección 10.4 del encargo):
	 *  - el canvas y el `SkinViewer` se crean una sola vez, cuando el
	 *    elemento existe (primer `$effect`).
	 *  - cambios de `skinUrl`/`capeUrl` (nueva cuenta, skin actualizado,
	 *    capa elegida) solo piden la textura de nuevo, sin recrear el
	 *    viewer.
	 *  - si `loadSkin`/`loadCape` rechaza (textura caída, sin red — ver
	 *    `network.svelte.ts`), se avisa por `onSkinError`/`onCapeError` en
	 *    vez de dejar un canvas mudo; el caller decide qué mostrar.
	 *  - dispose() corre tanto si el componente se destruye (navegación
	 *    fuera, Home deja de estar montada porque AppShell la reemplaza por
	 *    InstanceDetail) como si width/height cambian (se recrea limpio).
	 */
	let {
		skinUrl = null,
		capeUrl = null,
		width,
		height,
		autoRotate = true,
		autoRotateSpeed = 0.7,
		zoom = 0.9,
		onSkinError,
		onCapeError
	}: {
		skinUrl?: string | null;
		capeUrl?: string | null;
		width: number;
		height: number;
		autoRotate?: boolean;
		autoRotateSpeed?: number;
		zoom?: number;
		onSkinError?: (error: unknown) => void;
		onCapeError?: (error: unknown) => void;
	} = $props();

	let canvasEl: HTMLCanvasElement | undefined = $state();
	let viewer: SkinViewer | undefined;

	// Perfil Lite / "reducir movimiento" → personaje estático en vez de
	// auto-rotate corriendo siempre (sección 8 de esta entrega: alcanza con
	// esto acá, el idle/lighting elaborado de Experience es Entrega D).
	const effectiveAutoRotate = $derived(
		autoRotate && appearance.quality !== 'lite' && !appearance.reduceMotion
	);

	$effect(() => {
		if (!canvasEl) return;
		const created = new SkinViewer({ canvas: canvasEl, width, height });
		viewer = created;
		return () => {
			created.dispose();
			if (viewer === created) viewer = undefined;
		};
	});

	$effect(() => {
		if (!viewer) return;
		viewer.autoRotate = effectiveAutoRotate;
		viewer.autoRotateSpeed = autoRotateSpeed;
		viewer.zoom = zoom;
	});

	$effect(() => {
		if (!viewer) return;
		if (!skinUrl) {
			viewer.loadSkin(null);
			return;
		}
		Promise.resolve(viewer.loadSkin(skinUrl)).catch((err) => onSkinError?.(err));
	});

	$effect(() => {
		if (!viewer) return;
		if (!capeUrl) {
			viewer.loadCape(null);
			return;
		}
		Promise.resolve(viewer.loadCape(capeUrl)).catch((err) => onCapeError?.(err));
	});
</script>

<canvas bind:this={canvasEl} aria-hidden="true"></canvas>

<style>
	canvas {
		display: block;
		cursor: grab;
	}
	canvas:active {
		cursor: grabbing;
	}
</style>
