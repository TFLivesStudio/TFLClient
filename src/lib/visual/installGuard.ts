/** Coalesce the same project/instance/kind while its existing installer is working. */
const pending = new Map<string, Promise<void>>();
export async function coalesceInstall(key: string, install: () => Promise<unknown>): Promise<void> {
	const existing = pending.get(key);
	if (existing) return existing;
	const task = Promise.resolve()
		.then(install)
		.then(() => {});
	pending.set(key, task);
	try {
		await task;
	} finally {
		if (pending.get(key) === task) pending.delete(key);
	}
}
