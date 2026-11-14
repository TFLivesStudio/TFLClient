import { persistAuthority } from '$lib/integrations/preferences';
import {
	DEFAULT_SCENES,
	parseScenes,
	parseScene,
	type SceneConfig,
	type ScenePreferences
} from './model';
export const SCENES_KEY = 'tfl-scenes-v1';
export const SCENES_V1_BACKUP_KEY = 'tfl-scenes-v1-before-global-backgrounds';
export const scenes = $state<ScenePreferences>(structuredClone(DEFAULT_SCENES));
export const sceneStatus = $state({ error: false });
export function restoreScenes() {
	try {
		const raw = localStorage.getItem(SCENES_KEY);
		const parsed = JSON.parse(raw ?? '{}');
		Object.assign(scenes, parseScenes(parsed));
		if (raw && (parsed?.version === 1 || parsed?.version === undefined)) {
			// Never delete the previous representation; authority writes remain revision checked.
			try {
				if (!localStorage.getItem(SCENES_V1_BACKUP_KEY))
					localStorage.setItem(SCENES_V1_BACKUP_KEY, raw);
				localStorage.setItem(SCENES_KEY, JSON.stringify($state.snapshot(scenes)));
			} catch {
				sceneStatus.error = true;
			}
		}
	} catch {
		Object.assign(scenes, structuredClone(DEFAULT_SCENES));
	}
}
export function watchScenes() {
	restoreScenes();
	const change = (e: StorageEvent) => {
		if (e.key === SCENES_KEY || e.key === null) restoreScenes();
	};
	window.addEventListener('storage', change);
	return () => window.removeEventListener('storage', change);
}
export function saveScene(config: SceneConfig, instance: string | null = null) {
	if (instance && ['__proto__', 'constructor', 'prototype'].includes(instance)) return;
	if (instance) scenes.instances[instance] = parseScene(config);
	else scenes.global = parseScene(config);
	persist();
}
export function inheritScene(instance: string) {
	delete scenes.instances[instance];
	persist();
}
export function persist() {
	try {
		localStorage.setItem(SCENES_KEY, JSON.stringify($state.snapshot(scenes)));
		persistAuthority('scenes', $state.snapshot(scenes));
		sceneStatus.error = false;
	} catch {
		sceneStatus.error = true;
	}
}
