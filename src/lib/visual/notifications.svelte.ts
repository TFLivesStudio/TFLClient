import { react, stopReactions } from '$lib/scenes/reactions.svelte';
import { listen } from '@tauri-apps/api/event';
export interface Notice {
	id: number;
	kind: 'success' | 'error' | 'info';
	message: string;
	title: 'finished' | 'failed' | 'sessionStarted' | 'sessionStopped';
	time: number;
	read: boolean;
}
export const notifications = $state<{ items: Notice[]; toasts: Notice[] }>({
	items: [],
	toasts: []
});
let sequence = 0;
export function addNotice(notice: Omit<Notice, 'id' | 'time' | 'read'>) {
	if (notice.kind === 'success' || notice.kind === 'error') react(notice.kind);
	const item = { ...notice, id: ++sequence, time: Date.now(), read: false };
	notifications.items = [item, ...notifications.items].slice(0, 50);
	notifications.toasts = [item, ...notifications.toasts].slice(0, 3);
}
export function startNotifications() {
	let disposed = false;
	let unlisten: (() => void) | undefined;
	listen<{ type: string; data: Record<string, unknown> }>(
		'app-event',
		({ payload: { type, data } }) => {
			const name = String(data.task ?? data.name ?? '').slice(0, 200);
			if (type === 'DownloadFinished')
				addNotice({ kind: 'success', title: 'finished', message: name });
			else if (type === 'DownloadFailed')
				addNotice({
					kind: 'error',
					title: 'failed',
					message: `${name}: ${String(data.error ?? '').slice(0, 600)}`
				});
			else if (
				type === 'InstanceStatusChanged' &&
				(data.status === 'running' || data.status === 'stopped')
			)
				addNotice({
					kind: 'info',
					title: data.status === 'running' ? 'sessionStarted' : 'sessionStopped',
					message: name
				});
		}
	)
		.then((fn) => {
			if (disposed) fn();
			else unlisten = fn;
		})
		.catch(() => {
			/* Existing IPC errors remain in their operation's UI. */
		});
	return () => {
		disposed = true;
		unlisten?.();
		stopReactions();
	};
}
