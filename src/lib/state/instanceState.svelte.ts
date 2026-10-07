import { getInstances } from '$lib/api';
import { appState } from '$lib/state/state.svelte';
import { closeModal, openModal } from '$lib/state/uiState.svelte';
import type { InstanceData } from '$lib/types/types';

// Navegación pedida desde fuera de InstanceDetail (menú contextual de la
// barra lateral: "Ir a Mods"). `navNonce` fuerza un remount de InstanceDetail
// incluso si la instancia ya estaba seleccionada, donde el cambio de uuid
// solo no alcanzaría.
export const instanceNav = $state<{ pendingInitialTab: string | undefined; navNonce: number }>({
	pendingInitialTab: undefined,
	navNonce: 0
});

export async function refreshInstances() {
	appState.instances = await getInstances();
}

export function selectInstance(instance: InstanceData) {
	appState.selectedInstance = instance;
}

/** Selecciona la instancia y abre InstanceDetail directamente en `tab`. */
export function openInstanceTab(instance: InstanceData, tab: string) {
	instanceNav.pendingInitialTab = tab;
	instanceNav.navNonce++;
	appState.selectedInstance = instance;
}

/** Al cerrarse el juego se guardó el tiempo jugado y la última vez que se
 * abrió — se vuelve a leer la lista (y la instancia seleccionada) para que
 * Detalles lo muestre ya, sin esperar a reabrir el launcher. */
export async function refreshAfterInstanceStopped() {
	await refreshInstances();
	const current = appState.selectedInstance;
	if (current) {
		const fresh = appState.instances.find((i) => i.uuid === current.uuid);
		if (fresh) appState.selectedInstance = fresh;
	}
}

export async function handleCreated(instance: InstanceData) {
	closeModal('create');
	await refreshInstances();
	appState.selectedInstance = instance;
}

export async function handleServerCreated(instance: InstanceData) {
	closeModal('createServer');
	await refreshInstances();
	appState.selectedInstance = instance;
	if (appState.settings?.server_connection_info_shown !== true) {
		openModal('serverConnectionInfo');
	}
}

export async function handleInstanceChanged(instance: InstanceData | null) {
	await refreshInstances();
	appState.selectedInstance = instance;
}
