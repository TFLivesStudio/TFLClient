import { listen } from '@tauri-apps/api/event';
import { getCurrentUser, getSettings } from '$lib/api';
import {
	applyQualityVisuals,
	observeAppearance,
	restoreAppearance
} from '$lib/state/appearanceState.svelte';
import { initDownloadListener } from '$lib/state/downloadState.svelte';
import { initGameSessionListener } from '$lib/state/gameSession.svelte';
import { refreshAfterInstanceStopped, refreshInstances } from '$lib/state/instanceState.svelte';
import { initNetworkListener } from '$lib/state/network.svelte';
import { initServerSessionListener } from '$lib/state/serverSessions.svelte';
import { appState } from '$lib/state/state.svelte';
import { ui } from '$lib/state/uiState.svelte';
import { checkAndDownloadUpdate } from '$lib/state/updateState.svelte';

// Esta misma index.html también se usa para la ventana emergente del log
// en vivo — Tauri la abre con un initialization_script que setea esta
// variable global ANTES de que cargue cualquier script de la página
// (no por query string: WebviewUrl::App toma el string entero como
// path de archivo literal, no lo parsea como URL+query). Si está
// presente, esta ventana es la de log — se salta toda la carga normal
// del launcher (cuentas, instancias, ajustes…).
export const isLogWindow =
	typeof window !== 'undefined' &&
	typeof (window as unknown as { __TFL_LOG_INSTANCE__?: string }).__TFL_LOG_INSTANCE__ === 'string';

/** Secuencia de arranque del launcher. Se llama una vez desde onMount de la página. */
export async function startApp() {
	if (isLogWindow) {
		ui.loading = false;
		return;
	}

	observeAppearance();

	initDownloadListener();
	initGameSessionListener();
	listen<{ type: string; data: { status?: string } }>('app-event', async (event) => {
		if (event.payload.type !== 'InstanceStatusChanged' || event.payload.data.status !== 'stopped')
			return;
		await refreshAfterInstanceStopped();
	});
	initServerSessionListener();
	initNetworkListener();
	try {
		const [user, settings] = await Promise.all([getCurrentUser(), getSettings()]);
		appState.currentUser = user;
		appState.settings = settings;
		if (settings.theme === 'light' || settings.theme === 'dark') {
			document.documentElement.setAttribute('data-theme', settings.theme);
		}
		await restoreAppearance();
		applyQualityVisuals(settings);
		await refreshInstances();
		// No await a propósito — chequeo/descarga en segundo plano, no
		// debe bloquear el arranque del launcher. El badge (si aparece)
		// lo dispara el store cuando termine, no esto.
		if (settings.auto_updates) void checkAndDownloadUpdate();
	} finally {
		ui.loading = false;
	}
}
