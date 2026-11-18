import { listServerTasks, type ServerTask } from '$lib/api/serverTasks';
import { listen } from '@tauri-apps/api/event';
import { refreshAfterInstanceStopped } from './instanceState.svelte';
export const serverTasks = $state<{ items: ServerTask[]; error: string }>({ items: [], error: '' });
let loading = false,
	again = false;
export async function refreshServerTasks() {
	if (loading) {
		again = true;
		return;
	}
	loading = true;
	try {
		serverTasks.items = await listServerTasks();
		serverTasks.error = '';
	} catch (error) {
		serverTasks.error = String(error);
	} finally {
		loading = false;
		if (again) {
			again = false;
			void refreshServerTasks();
		}
	}
}
export function startServerTasks() {
	void refreshServerTasks();
	const stop = listen<{ type: string }>('app-event', (event) => {
		if (event.payload.type !== 'ServerTasksChanged') return;
		void refreshServerTasks();
		void refreshAfterInstanceStopped();
	});
	return () => void stop.then((f) => f());
}
