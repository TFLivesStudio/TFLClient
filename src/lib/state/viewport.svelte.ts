import { appState } from './state.svelte';

/**
 * Ancho reactivo de la ventana — Entrega B (Shell responsive). No existía
 * ningún `matchMedia`/`ResizeObserver`/breakpoint en todo el repo antes de
 * esto (la única excepción, un `@media` local a WelcomeState.svelte, no es
 * de esta entrega). La ventana entera es el contenedor que importa acá
 * (TitleBar + Sidebar + main deciden su layout en función de ella), así que
 * alcanza con `window.innerWidth` + un listener de resize debounced — no
 * hace falta observar un nodo en particular.
 *
 * Dos conceptos DISTINTOS que no hay que confundir al tocar este archivo
 * (sección 4 del encargo lo pide explícito):
 *
 *  - "manual": `appState.settings.sidebar_collapsed` — la preferencia del
 *    usuario, persistida en el backend (ver Sidebar.svelte / toggleCollapsed
 *    / updateSettings). SOLO cambia cuando el usuario clickea el toggle de
 *    la sidebar. Este módulo nunca la escribe.
 *  - "forzado": `viewport.width < SIDEBAR_FORCE_COMPACT_WIDTH` — la ventana
 *    no tiene espacio para la sidebar completa en este momento. Es un
 *    override puramente VISUAL: nunca toca `appState.settings`. Si la
 *    ventana se agranda de nuevo por encima del umbral, la sidebar vuelve a
 *    mostrarse tal como la preferencia manual indique, sin quedar "pegada"
 *    en compacto.
 *
 * `isSidebarEffectiveCollapsed()` combina ambos — es la función que
 * Sidebar.svelte Y TitleBar.svelte consultan para decidir su layout, para
 * que los dos SIEMPRE coincidan (evita el desalineamiento marca/sidebar de
 * la sección 6.2 del encargo cuando cambia el ancho de cualquiera de las
 * dos formas posibles).
 */

// Espejo de --sidebar-width/--titlebar-height en src/styles/global.css (no
// se puede tocar ese archivo en esta entrega — alcance explícito: solo
// AppShell/TitleBar/Sidebar/este módulo). Si alguno de los dos valores
// cambia alguna vez, hay que actualizar el otro a mano.
export const SIDEBAR_EXPANDED_WIDTH = 272;

// Ancho del riel solo-íconos. Antes vivía hardcodeado y duplicado a mano
// dentro de Sidebar.svelte; ahora es un único valor que Sidebar.svelte Y
// TitleBar.svelte importan de acá, así el ancho de la marca en la barra de
// título y el ancho real de la sidebar nunca pueden quedar desincronizados.
export const SIDEBAR_COLLAPSED_WIDTH = 76;

// Debajo de este ancho de ventana ya no entran cómodos la sidebar completa
// (272px) más un panel principal usable — se fuerza el modo compacto sin
// tocar la preferencia manual. La ventana mínima soportada por Tauri es
// 800x600 (ver src-tauri/tauri.conf.json): a 800px de ancho, con la sidebar
// forzada a 76px, el panel principal todavía tiene ~700px, cómodo; es
// bastante más abajo del umbral elegido acá.
export const SIDEBAR_FORCE_COMPACT_WIDTH = 960;

export const viewport = $state<{ width: number }>({
	width: typeof window !== 'undefined' ? window.innerWidth : 1280
});

let initialized = false;
let resizeTimer: ReturnType<typeof setTimeout> | undefined;

/** Se llama una sola vez desde el onMount de AppShell.svelte (patrón igual
 * a initNetworkListener/observeAppearance: idempotente, no hace nada si ya
 * se inicializó o si no hay `window` todavía). */
export function initViewportListener() {
	if (initialized || typeof window === 'undefined') return;
	initialized = true;
	viewport.width = window.innerWidth;
	const onResize = () => {
		clearTimeout(resizeTimer);
		resizeTimer = setTimeout(() => {
			viewport.width = window.innerWidth;
		}, 120);
	};
	window.addEventListener('resize', onResize);
}

/** Preferencia manual OR falta de espacio — ver comentario largo arriba.
 * Única función que Sidebar.svelte/TitleBar.svelte deberían consultar para
 * decidir si el layout va colapsado. */
export function isSidebarEffectiveCollapsed(): boolean {
	return (
		appState.settings?.sidebar_collapsed === true || viewport.width < SIDEBAR_FORCE_COMPACT_WIDTH
	);
}

/** true solo cuando el colapso actual es por falta de espacio (la
 * preferencia manual sigue siendo "expandida") — Sidebar.svelte la usa para
 * deshabilitar su propio toggle manual en ese caso en vez de dejar que un
 * click grabe sin querer una preferencia que el usuario no pidió (el click
 * no tendría efecto visible hasta agrandar la ventana de nuevo). */
export function isSidebarForcedCompact(): boolean {
	return (
		appState.settings?.sidebar_collapsed !== true && viewport.width < SIDEBAR_FORCE_COMPACT_WIDTH
	);
}
