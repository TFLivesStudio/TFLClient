import { getCurrentUser, getSettings, logout as apiLogout } from '$lib/api';
import { applyQualityVisuals } from '$lib/state/appearanceState.svelte';
import { appState } from '$lib/state/state.svelte';
import { ui } from '$lib/state/uiState.svelte';
import type { MinecraftUser } from '$lib/types/types';

export async function handleOnboardingDone(user: MinecraftUser) {
	appState.currentUser = user;
	appState.settings = await getSettings();
	if (appState.settings) applyQualityVisuals(appState.settings);
	ui.justOnboarded = true;
}

export async function handleLogout() {
	await apiLogout();
	appState.currentUser = await getCurrentUser();
	appState.settings = await getSettings();
}
