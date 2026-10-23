export type TaskStatus =
	'queued' | 'running' | 'succeeded' | 'failed' | 'cancel_requested' | 'cancelled' | 'interrupted';
export interface ActivityTask {
	instance_id: string | null;
	project_id: string | null;
	id: string;
	revision: number;
	kind: 'content.install';
	source: 'ui' | 'mcp' | 'system';
	target: 'mod' | 'shader' | 'resource_pack' | 'plugin';
	status: TaskStatus;
	stage: 'waiting' | 'resolving' | 'downloading' | 'installing' | 'finished';
	created_at: number;
	updated_at: number;
	finished_at: number | null;
	progress: { current: number; total: number } | null;
	cancellable: boolean;
	error: 'install_failed' | 'interrupted' | null;
}
export interface ActivitySnapshot {
	revision: number;
	tasks: ActivityTask[];
	storage_warning: boolean;
}
export const terminal = (status: TaskStatus) =>
	['succeeded', 'failed', 'cancelled', 'interrupted'].includes(status);

/** One projection, subscribe before snapshot, coalesced reloads, stale responses ignored. */
export function connectActivity(
	listen: (changed: () => void) => Promise<() => void>,
	load: () => Promise<ActivitySnapshot>,
	apply: (snapshot: ActivitySnapshot) => void,
	onError: () => void
) {
	let disposed = false,
		busy = false,
		dirty = false,
		subscribed = false;
	let stop: (() => void) | undefined;
	let revision = -1;
	let retryTimer: ReturnType<typeof setTimeout> | undefined;
	let backoff = 1000;
	async function sync() {
		if (disposed) return;
		if (busy) {
			dirty = true;
			return;
		}
		busy = true;
		try {
			if (!subscribed) {
				const unlisten = await listen(() => {
					void sync();
				});
				if (disposed) {
					unlisten();
					return;
				}
				stop = unlisten;
				subscribed = true;
			}
			do {
				dirty = false;
				const snapshot = await load();
				if (!disposed && snapshot.revision >= revision) {
					revision = snapshot.revision;
					apply(snapshot);
				}
			} while (dirty && !disposed);
			backoff = 1000;
			clearTimeout(retryTimer);
		} catch {
			if (!disposed) {
				onError();
				clearTimeout(retryTimer);
				retryTimer = setTimeout(() => {
					void sync();
				}, backoff);
				backoff = Math.min(backoff * 2, 30000);
			}
		} finally {
			busy = false;
		}
	}
	void sync();
	return {
		sync,
		stop() {
			disposed = true;
			clearTimeout(retryTimer);
			stop?.();
		}
	};
}
