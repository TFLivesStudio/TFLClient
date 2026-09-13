import { listen } from '@tauri-apps/api/event';
import type { AppEvent } from '$lib/types/types';

export interface DownloadTask {
	task: string;
	stage: string;
	itemCurrent: number;
	itemTotal: number;
	bytesCurrent: number;
	bytesTotal: number;
	currentItem: string | null;
	failed?: string;
}

export const downloadTasks = $state<Record<string, DownloadTask>>({});

let initialized = false;

export function initDownloadListener() {
	if (initialized) return;
	initialized = true;

	listen<AppEvent>('app-event', (event) => {
		const payload = event.payload as unknown as {
			type: string;
			data: Record<string, unknown>;
		};

		if (payload.type === 'DownloadProgress') {
			const d = payload.data as {
				task: string;
				stage: string;
				item_current: number;
				item_total: number;
				bytes_current: number;
				bytes_total: number;
				current_item: string | null;
			};
			downloadTasks[d.task] = {
				task: d.task,
				stage: d.stage,
				itemCurrent: d.item_current,
				itemTotal: d.item_total,
				bytesCurrent: d.bytes_current,
				bytesTotal: d.bytes_total,
				currentItem: d.current_item
			};
		} else if (payload.type === 'DownloadFinished') {
			const d = payload.data as { task: string };
			delete downloadTasks[d.task];
		} else if (payload.type === 'DownloadFailed') {
			const d = payload.data as { task: string; error: string };
			if (downloadTasks[d.task]) {
				downloadTasks[d.task].failed = d.error;
			}
			setTimeout(() => {
				delete downloadTasks[d.task];
			}, 5000);
		}
	});
}
