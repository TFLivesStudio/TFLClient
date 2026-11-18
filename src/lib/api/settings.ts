// Ajustes, RAM recomendada y fondo personalizado
import { invoke } from './invoke';
import type { Settings, QualityProfile, RecommendedRam } from '$lib/types/types';
import { appState } from '$lib/state/state.svelte';
import { settingsSaveStatus } from '$lib/state/settingsSave.svelte';
export { settingsSaveStatus } from '$lib/state/settingsSave.svelte';

let persisted: Settings | null = null;
let queue: Promise<unknown> = Promise.resolve();
function accept(settings: Settings) {
	if (persisted && (persisted.settings_revision ?? 0) > (settings.settings_revision ?? 0)) {
		return structuredClone(persisted);
	}
	persisted = structuredClone(settings);
	appState.settings = settings;
	return settings;
}
export async function getSettings() {
	return accept(await invoke<Settings>('get_settings'));
}
export function patchSettings(patch: Partial<Settings>): Promise<Settings> {
	const captured: Partial<Settings> = JSON.parse(JSON.stringify(patch));
	settingsSaveStatus.pending++;
	const task = queue
		.catch(() => {})
		.then(async () => {
			for (let attempt = 0; attempt < 2; attempt++) {
				if (!persisted) await getSettings();
				try {
					const saved = await invoke<Settings>('patch_settings', {
						patch: captured,
						expectedRevision: persisted?.settings_revision ?? 0
					});
					settingsSaveStatus.error = '';
					return accept(saved);
				} catch (error) {
					if (String(error).includes('SETTINGS_CONFLICT') && attempt === 0) {
						await getSettings();
						continue;
					}
					settingsSaveStatus.error = String(error);
					// Reload the last valid committed value; never leave an optimistic failed value.
					try {
						await getSettings();
					} catch {
						/* Keep the prior valid snapshot. */
					}
					throw error;
				}
			}
			throw new Error('SETTINGS_CONFLICT');
		})
		.finally(() => settingsSaveStatus.pending--);
	queue = task;
	return task;
}
export const setQualityProfile = (profile: QualityProfile) =>
	patchSettings({
		quality_profile: profile,
		reduce_animations: profile === 'Lite',
		disable_blur_effects: profile === 'Lite',
		disable_infinite_animations: profile !== 'Experience'
	});
export const getRecommendedRam = () => invoke<RecommendedRam>('get_recommended_ram');
export const getRecommendedRamForInstance = (name: string) =>
	invoke<RecommendedRam>('get_recommended_ram_for_instance', { name });
export const setCustomWallpaper = (sourcePath: string) =>
	invoke<string>('set_custom_wallpaper', { sourcePath });
export const getCustomWallpaperPath = () => invoke<string | null>('get_custom_wallpaper_path');
