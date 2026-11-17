import { invoke } from '$lib/api/invoke';
import { listen } from '@tauri-apps/api/event';
import { applyVisualAttributes, VISUAL_APPLIED_EVENT } from '$lib/visual/preferences.svelte';
import { scenes, persist } from '$lib/scenes/preferences.svelte';
import { DEFAULT_SCENE } from '$lib/scenes/model';
import {
	contributionId,
	enabledAddons,
	findScene,
	findTheme,
	THEME_TOKENS,
	themeContrastIssues,
	readableThemeAccent,
	themeRawAliases,
	contrastRatio,
	readableSemanticColors,
	type AddonCatalog
} from './model';
export const ADDON_THEME_KEY = 'tfl-addon-theme-v1';
export const addonState = $state({
	catalog: { safeMode: false, addons: [] } as AddonCatalog,
	loaded: false,
	error: '',
	theme: ''
});
const previous = new Map<string, string>();
function resolvedThemeDefaults(root: HTMLElement, keys: readonly string[] = THEME_TOKENS) {
	const probe = document.createElement('span');
	probe.style.cssText = 'position:fixed;visibility:hidden;pointer-events:none';
	root.append(probe);
	const context = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
	try {
		return Object.fromEntries(
			keys.map((key) => {
				probe.style.color = `var(${key})`;
				const color = getComputedStyle(probe).color;
				if (!context) return [key, color];
				context.clearRect(0, 0, 1, 1);
				context.fillStyle = color;
				context.fillRect(0, 0, 1, 1);
				const [r, g, b, a] = context.getImageData(0, 0, 1, 1).data;
				return [key, `rgba(${r},${g},${b},${a / 255})`];
			})
		);
	} finally {
		probe.remove();
	}
}
function restoreTheme() {
	if (typeof document === 'undefined') return;
	for (const [key, value] of previous) {
		if (value) document.documentElement.style.setProperty(key, value);
		else document.documentElement.style.removeProperty(key);
	}
	previous.clear();
	// Rebuilding host defaults is internal to reapplication; do not notify recursively.
	applyVisualAttributes(false);
}
export function applyAddonTheme() {
	restoreTheme();
	if (typeof document === 'undefined') return true;
	const found = findTheme(addonState.catalog, addonState.theme);
	if (!found) return true;
	const root = document.documentElement;
	const defaults = resolvedThemeDefaults(root);
	const issues = themeContrastIssues(found.theme.tokens, defaults);
	if (issues.length) {
		addonState.error = `Tema sin contraste suficiente (4.5:1): ${issues.join(', ')}.`;
		return false;
	}
	for (const [key, value] of Object.entries(found.theme.tokens)) {
		previous.set(key, root.style.getPropertyValue(key));
		root.style.setProperty(key, value);
	}
	// Keep host material derivations while applying the addon's explicit raw palette.
	for (const [key, value] of Object.entries(themeRawAliases(found.theme.tokens))) {
		previous.set(key, root.style.getPropertyValue(key));
		root.style.setProperty(key, value);
	}
	// Link/focus text must follow the installed palette even when the host appearance is light.
	previous.set('--text-accent', root.style.getPropertyValue('--text-accent'));
	root.style.setProperty('--text-accent', readableThemeAccent(found.theme.tokens, defaults));
	const accent = found.theme.tokens['--accent'];
	if (accent)
		for (const [key, value] of Object.entries({
			'--accent-rgb': [1, 3, 5].map((n) => parseInt(accent.slice(n, n + 2), 16)).join(', '),
			'--accent-hover': `color-mix(in srgb, ${accent} 85%, white)`
		})) {
			previous.set(key, root.style.getPropertyValue(key));
			root.style.setProperty(key, value);
		}
	const surfaces = [
		'--surface-canvas',
		'--surface-workspace',
		'--surface-card',
		'--surface-elevated',
		'--surface-interactive'
	];
	const effective = resolvedThemeDefaults(root, [
		...surfaces,
		'--text-primary',
		'--text-secondary',
		'--text-muted',
		'--text-accent'
	]);
	const surfaceIssues = surfaces.flatMap((surface) =>
		['--text-primary', '--text-secondary', '--text-muted']
			.filter((text) => contrastRatio(effective[text], effective[surface]) < 4.5)
			.map((text) => `${text} / ${surface}`)
	);
	if (surfaceIssues.length) {
		restoreTheme();
		addonState.error = `Tema sin contraste suficiente en el material actual (4.5:1): ${surfaceIssues.join(', ')}.`;
		return false;
	}
	if (
		surfaces.some((surface) => contrastRatio(effective['--text-accent'], effective[surface]) < 4.5)
	)
		root.style.setProperty('--text-accent', effective['--text-primary']);
	const semanticKeys = ['--color-error', '--color-success', '--color-warning'];
	const semanticDefaults = resolvedThemeDefaults(root, semanticKeys);
	for (const [key, value] of Object.entries(
		readableSemanticColors(
			surfaces.map((surface) => effective[surface]),
			effective['--text-primary'],
			semanticDefaults
		)
	)) {
		previous.set(key, root.style.getPropertyValue(key));
		root.style.setProperty(key, value);
	}
	return true;
}
export function selectAddonTheme(id: string) {
	addonState.theme = id && findTheme(addonState.catalog, id) ? id : '';
	addonState.error = '';
	if (!applyAddonTheme()) addonState.theme = '';
	try {
		localStorage.setItem(ADDON_THEME_KEY, addonState.theme);
	} catch {
		addonState.error = 'No se pudo guardar el tema del addon.';
	}
}
export function acceptCatalog(catalog: AddonCatalog) {
	addonState.catalog = catalog;
	addonState.loaded = true;
	addonState.error = '';
	if (addonState.theme && !findTheme(catalog, addonState.theme)) selectAddonTheme('');
	else if (!applyAddonTheme()) {
		addonState.theme = '';
		try {
			localStorage.setItem(ADDON_THEME_KEY, '');
		} catch {
			/* theme remains at safe default */
		}
	}
	let changed = false;
	for (const config of [scenes.global, ...Object.values(scenes.instances)]) {
		if (config.preset.startsWith('addon:') && !findScene(catalog, config.preset)) {
			config.preset = DEFAULT_SCENE.preset;
			changed = true;
		}
	}
	if (changed) persist();
}
export async function refreshAddons() {
	try {
		await catalogOperation('addons_list', {});
	} catch (error) {
		addonState.error = String(error);
	}
}
export async function changeAddons(command: string, args: Record<string, unknown>) {
	await catalogOperation(command, args);
}
let pendingCatalog: Promise<void> = Promise.resolve();
let lifecycle = 0;
function catalogOperation(command: string, args: Record<string, unknown>) {
	const generation = lifecycle;
	const next = pendingCatalog.then(async () => {
		const catalog = await invoke<AddonCatalog>(command, args);
		if (generation === lifecycle) acceptCatalog(catalog);
	});
	pendingCatalog = next.catch(() => {
		/* keep subsequent recovery operations available */
	});
	return next;
}
export function addonThemes() {
	return enabledAddons(addonState.catalog).flatMap((addon) =>
		addon.manifest.themes.map((theme) => ({
			id: contributionId(addon.manifest.id, theme.id),
			name: theme.name,
			packageName: addon.manifest.name
		}))
	);
}
export function addonSceneOptions() {
	return enabledAddons(addonState.catalog).flatMap((addon) =>
		addon.manifest.scenes.map((scene) => ({
			id: contributionId(addon.manifest.id, scene.id),
			name: [scene.name, scene.name] as [string, string],
			description: [addon.manifest.name, addon.manifest.name] as [string, string],
			colors: [scene.background, scene.background, scene.background] as [string, string, string]
		}))
	);
}
export function resolveAddonScene(id: string) {
	return findScene(addonState.catalog, id);
}
export function startAddons() {
	lifecycle++;
	let stopped = false;
	let unlisten: (() => void) | undefined;
	try {
		addonState.theme = localStorage.getItem(ADDON_THEME_KEY) ?? '';
	} catch {
		/* default theme */
	}
	void listen('addons-changed', () => {
		void refreshAddons();
	})
		.then((stop) => {
			if (stopped) stop();
			else {
				unlisten = stop;
				void refreshAddons();
			}
		})
		.catch(() => {
			if (!stopped) void refreshAddons();
		});
	const storage = (e: StorageEvent) => {
		if (e.key === ADDON_THEME_KEY || e.key === null) {
			try {
				addonState.theme = localStorage.getItem(ADDON_THEME_KEY) ?? '';
			} catch {
				addonState.theme = '';
			}
			applyAddonTheme();
		}
	};
	window.addEventListener('storage', storage);
	const visualApplied = () => {
		if (addonState.loaded && addonState.theme) selectAddonTheme(addonState.theme);
	};
	window.addEventListener(VISUAL_APPLIED_EVENT, visualApplied);
	const colorAttributes = [
		'data-material',
		'data-quality',
		'data-theme',
		'data-surface',
		'data-adaptive',
		'data-wallpaper'
	];
	const colorSignature = () =>
		JSON.stringify(colorAttributes.map((key) => document.documentElement.getAttribute(key)));
	let previousAppearance = colorSignature();
	const appearanceObserver = new MutationObserver(() => {
		const next = colorSignature();
		if (next === previousAppearance) return;
		previousAppearance = next;
		if (addonState.loaded && addonState.theme) selectAddonTheme(addonState.theme);
	});
	appearanceObserver.observe(document.documentElement, {
		attributes: true,
		attributeFilter: colorAttributes
	});
	return () => {
		stopped = true;
		lifecycle++;
		unlisten?.();
		window.removeEventListener('storage', storage);
		window.removeEventListener(VISUAL_APPLIED_EVENT, visualApplied);
		appearanceObserver.disconnect();
		restoreTheme();
	};
}
