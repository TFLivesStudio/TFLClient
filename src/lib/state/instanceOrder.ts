import type { InstanceData } from '$lib/types/types';
export function orderByActivity(instances: InstanceData[], running: (i: InstanceData) => boolean) {
	return [...instances].sort(
		(a, b) =>
			Number(running(b)) - Number(running(a)) ||
			Math.max(b.last_activity ?? 0, b.last_played) -
				Math.max(a.last_activity ?? 0, a.last_played) ||
			a.uuid.localeCompare(b.uuid, 'en')
	);
}
