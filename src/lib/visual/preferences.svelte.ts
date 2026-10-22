import { persistAuthority } from '$lib/integrations/preferences';
import {
	DEFAULT_VISUAL,
	accentText,
	parsePreferences,
	type VisualPreferences,
	type InstanceVisual
} from './policy';
const KEY = 'tfl-visual-refresh-v1';
export const visual = $state<VisualPreferences>(structuredClone(DEFAULT_VISUAL));
export const preferenceStatus = $state({ error: false });
export function applyVisualAttributes() {
	if (typeof document === 'undefined') return;
	const root = document.documentElement;
	root.dataset.visual = visual.style;
	if (visual.customAccent) {
		root.style.setProperty('--accent', visual.customAccent);
		root.style.setProperty('--accent-text', accentText(visual.customAccent));
		root.style.setProperty(
			'--accent-hover',
			`color-mix(in srgb, ${visual.customAccent} 85%, white)`
		);
		root.style.setProperty(
			'--accent-rgb',
			[1, 3, 5].map((i) => parseInt(visual.customAccent.slice(i, i + 2), 16)).join(', ')
		);
	} else {
		for (const key of ['--accent', '--accent-text', '--accent-hover', '--accent-rgb'])
			root.style.removeProperty(key);
	}
	root.dataset.pixelType = visual.pixelType;
	root.dataset.material = visual.material;
	root.toggleAttribute('data-adaptive', visual.adaptive);
}
export function restoreVisualPreferences() {
	try {
		Object.assign(visual, parsePreferences(JSON.parse(localStorage.getItem(KEY) ?? '{}')));
	} catch {
		Object.assign(visual, structuredClone(DEFAULT_VISUAL));
	}
	applyVisualAttributes();
}
export function saveVisualPreferences() {
	applyVisualAttributes();
	try {
		localStorage.setItem(KEY, JSON.stringify($state.snapshot(visual)));
		persistAuthority('visual', $state.snapshot(visual));
		preferenceStatus.error = false;
	} catch {
		preferenceStatus.error = true;
	}
}
export function setVisual<K extends keyof VisualPreferences>(key: K, value: VisualPreferences[K]) {
	visual[key] = value;
	saveVisualPreferences();
}
export function instanceVisual(id: string): InstanceVisual {
	return visual.instances[id] ?? { favorite: false, group: '', artwork: 'auto' };
}
export function setInstanceVisual(id: string, patch: Partial<InstanceVisual>) {
	visual.instances[id] = { ...instanceVisual(id), ...patch };
	saveVisualPreferences();
}
export function addGroup(name: string) {
	const group = name.trim().slice(0, 40);
	if (!group || visual.groups.includes(group) || visual.groups.length >= 30) return;
	visual.groups.push(group);
	saveVisualPreferences();
}
export function removeGroup(group: string) {
	visual.groups = visual.groups.filter((g) => g !== group);
	for (const meta of Object.values(visual.instances)) if (meta.group === group) meta.group = '';
	saveVisualPreferences();
}
