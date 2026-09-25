import { listen } from '@tauri-apps/api/event';
import { appState } from './state.svelte';

// Análogo a gameSession.svelte.ts, pero un Set en vez de un único nombre —
// a diferencia del cliente (un lock global, una sola instancia corriendo a
// la vez), los servidores corren independientes entre sí y del cliente,
// así que puede haber varios activos al mismo tiempo.
export const serverSessions = $state<{ running: Set<string> }>({ running: new Set() });

let initialized = false;

export function initServerSessionListener() {
	if (initialized) return;
	initialized = true;

	listen<{ type: string; data: Record<string, unknown> }>('app-event', (event) => {
		const payload = event.payload;
		if (payload.type !== 'InstanceStatusChanged') return;
		const d = payload.data as { name: string; status: string };
		// El mismo evento lo emiten tanto client como server — solo nos
		// importa acá si la instancia en cuestión es de tipo servidor.
		const inst = appState.instances.find((i) => i.name === d.name);
		if (!inst?.server_type) return;
		const next = new Set(serverSessions.running);
		if (d.status === 'running') next.add(d.name);
		else next.delete(d.name);
		serverSessions.running = next;
	});
}
