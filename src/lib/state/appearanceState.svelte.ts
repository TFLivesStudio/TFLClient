import { convertFileSrc } from '@tauri-apps/api/core';
import { getCustomWallpaperPath } from '$lib/api';
import { appState } from '$lib/state/state.svelte';

// SettingsPanel cambia data-ambience directo en <html> (localStorage +
// setAttribute), no hay ningún store — para saber cuándo mostrar las
// partículas hace falta observar el atributo (ver `observeAppearance`).
export const appearance = $state<{ ambience: string; reduceMotion: boolean }>({
	ambience:
		typeof document !== 'undefined'
			? (document.documentElement.getAttribute('data-ambience') ?? 'aurora')
			: 'aurora',
	reduceMotion: false
});

/** Sincroniza `appearance` con los atributos de <html> y la mantiene al día. */
export function observeAppearance(): MutationObserver {
	const root = document.documentElement;
	const sync = () => {
		appearance.ambience = root.getAttribute('data-ambience') ?? 'aurora';
		appearance.reduceMotion = root.hasAttribute('data-reduce-motion');
	};
	sync();
	const observer = new MutationObserver(sync);
	observer.observe(root, {
		attributes: true,
		attributeFilter: ['data-ambience', 'data-reduce-motion']
	});
	return observer;
}

export async function restoreAppearance() {
	const root = document.documentElement;
	const preferences = [
		['tfl-accent', 'data-accent', 'orange'],
		['tfl-surface', 'data-surface', 'obsidian'],
		['tfl-ambience', 'data-ambience', 'aurora'],
		['tfl-density', 'data-density', 'comfortable'],
		['tfl-wallpaper', 'data-wallpaper', 'none'],
		['tfl-card-style', 'data-card-style', 'rich'],
		['tfl-font', 'data-font', 'system']
	] as const;
	for (const [storageKey, attribute, defaultValue] of preferences) {
		let value = localStorage.getItem(storageKey) ?? defaultValue;
		// OLED no tiene variante clara. Estados viejos (de antes del fix
		// de exclusión mutua OLED/claro) pueden tener surface='oled' con
		// theme='light' guardado a la vez — se normaliza acá una sola
		// vez, en vez de arrastrar el estado inválido en cada arranque.
		if (
			attribute === 'data-surface' &&
			value === 'oled' &&
			root.getAttribute('data-theme') === 'light'
		) {
			value = defaultValue;
			localStorage.setItem(storageKey, defaultValue);
		}
		if (value === defaultValue) root.removeAttribute(attribute);
		else root.setAttribute(attribute, value);
	}
	// El wallpaper propio no tiene regla fija en la hoja de estilos (la
	// URL es dinámica) — SettingsPanel lo setea inline al elegirlo, pero
	// eso no sobrevive un reinicio. Hay que reconstruirlo acá.
	if (localStorage.getItem('tfl-wallpaper') === 'custom') {
		try {
			const path = await getCustomWallpaperPath();
			if (path) root.style.setProperty('--wallpaper-bg', `url("${convertFileSrc(path)}")`);
		} catch {
			// sin wallpaper propio disponible, se queda con el fondo vacío
		}
	}
}

export function applyQualityVisuals(settings: NonNullable<typeof appState.settings>) {
	const root = document.documentElement;
	root.setAttribute('data-quality', settings.quality_profile.toLowerCase());
	root.toggleAttribute('data-reduce-motion', settings.quality_profile === 'Lite');
	root.toggleAttribute('data-no-blur', settings.disable_blur_effects);
	// Único consumidor real de disable_infinite_animations (antes se
	// guardaba en el backend pero ninguna CSS lo leía — por eso Balanced
	// y Experience se veían idénticos). Con el atributo ausente, la capa
	// de resplandor pasivo de Experience puede correr (ver global.css).
	root.toggleAttribute('data-no-infinite-fx', settings.disable_infinite_animations);
}
