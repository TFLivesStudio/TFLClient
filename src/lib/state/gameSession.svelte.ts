import { listen } from '@tauri-apps/api/event';
import { getRunningInstances } from '$lib/api';
import type { RunningInstanceInfo } from '$lib/types/types';

// Puede haber más de una instancia de cliente corriendo a la vez (con
// confirmación de por medio si hace falta, ver InstanceDetail.svelte) —
// por eso una lista y no un único nombre. `account_uuid` es la cuenta que
// se usó para lanzar CADA una, no necesariamente la activa ahora mismo.
export const gameSession = $state<{ running: RunningInstanceInfo[] }>({ running: [] });

let initialized = false;

async function refresh() {
	try {
		gameSession.running = await getRunningInstances();
	} catch {
		// sin cambios — mejor quedarse con el último estado conocido que
		// vaciarlo por un hipo de IPC puntual.
	}
}

export function initGameSessionListener() {
	if (initialized) return;
	initialized = true;

	refresh();

	listen<{ type: string; data: Record<string, unknown> }>('app-event', (event) => {
		if (event.payload.type === 'InstanceStatusChanged') refresh();
	});
}
