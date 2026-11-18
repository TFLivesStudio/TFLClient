import { appState } from './state.svelte';
import { patchSettings } from '$lib/api/settings';
export type ExperienceMode = 'simple' | 'advanced';
export const experience = {
	get mode(): ExperienceMode {
		return appState.settings?.ui_mode ?? 'simple';
	}
};
export const experienceStatus = $state({ saving: false, error: '' });
export async function setExperienceMode(mode: ExperienceMode) {
	if (!appState.settings || experienceStatus.saving || mode === experience.mode) return;
	experienceStatus.saving = true;
	experienceStatus.error = '';
	try {
		await patchSettings({ ui_mode: mode });
	} catch {
		experienceStatus.error = 'No se pudo guardar el modo. Se conserva la elección anterior.';
	} finally {
		experienceStatus.saving = false;
	}
}
