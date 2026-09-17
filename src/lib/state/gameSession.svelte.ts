import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export const gameSession = $state<{ running: string | null }>({ running: null });

let initialized = false;

export function initGameSessionListener() {
	if (initialized) return;
	initialized = true;

	invoke<string | null>('get_running_instance')
		.then((name) => (gameSession.running = name))
		.catch(() => {});

	listen<{ type: string; data: Record<string, unknown> }>('app-event', (event) => {
		const payload = event.payload;
		if (payload.type === 'InstanceStatusChanged') {
			const d = payload.data as { name: string; status: string };
			if (d.status === 'running') {
				gameSession.running = d.name;
			} else if (d.status === 'stopped' && gameSession.running === d.name) {
				gameSession.running = null;
			}
		}
	});
}
