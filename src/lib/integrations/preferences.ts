import { invoke } from '$lib/api/invoke';
import { listen } from '@tauri-apps/api/event';
interface Snapshot {
	revision: number;
	values: Record<string, unknown>;
}
const storageKeys: Record<string, string> = {
	visual: 'tfl-visual-refresh-v1',
	scenes: 'tfl-scenes-v1',
	console: 'tfl-console-v1'
};
let snapshot: Snapshot = { revision: 0, values: {} };
let ready = false,
	applying = false;
let queue: Promise<void> = Promise.resolve();
export const authorityStatus = { error: '' };
function apply(next: Snapshot) {
	if (
		!next ||
		!Number.isInteger(next.revision) ||
		!next.values ||
		next.revision < snapshot.revision
	)
		return;
	snapshot = next;
	applying = true;
	try {
		for (const [key, storageKey] of Object.entries(storageKeys)) {
			if (next.values[key] !== undefined) {
				localStorage.setItem(storageKey, JSON.stringify(next.values[key]));
				window.dispatchEvent(new StorageEvent('storage', { key: storageKey }));
			}
		}
		window.dispatchEvent(new CustomEvent('tfl-authority-updated'));
	} finally {
		applying = false;
	}
}
export function persistAuthority(key: string, value: unknown) {
	if (!ready || applying || JSON.stringify(snapshot.values[key]) === JSON.stringify(value)) return;
	const captured = structuredClone(value);
	queue = queue.then(async () => {
		try {
			const next = await invoke<Snapshot>('visual_preferences_write', {
				key,
				value: captured,
				expectedRevision: snapshot.revision
			});
			apply(next);
			authorityStatus.error = '';
		} catch {
			authorityStatus.error = 'Preference conflict or save error. Reloaded persisted settings.';
			try {
				apply(await invoke<Snapshot>('visual_preferences_get'));
			} catch {
				/* cache remains available offline */
			}
		}
	});
}
export async function startPreferenceAuthority() {
	try {
		const next = await invoke<Snapshot>('visual_preferences_get');
		if (!next || !Number.isInteger(next.revision)) return;
		apply(next);
		ready = true;
		// Migrate only missing records. Never overwrite an existing authority with stale localStorage.
		for (const [key, storageKey] of Object.entries(storageKeys)) {
			if (next.values[key] === undefined) {
				const raw = localStorage.getItem(storageKey);
				if (raw) {
					try {
						persistAuthority(key, JSON.parse(raw));
					} catch {
						/* malformed legacy cache */
					}
				}
			}
		}
		await listen<Snapshot>('mcp-preferences-changed', (e) => apply(e.payload));
	} catch {
		/* An old launcher or browser preview can still use its existing preferences. */
	}
}
