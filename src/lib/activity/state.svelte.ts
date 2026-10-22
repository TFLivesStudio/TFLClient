import { listen } from '@tauri-apps/api/event';
import { invoke } from '$lib/api/invoke';
import { connectActivity, type ActivitySnapshot } from './contract';
export const activityState = $state<{
	snapshot: ActivitySnapshot;
	loading: boolean;
	error: boolean;
}>({
	snapshot: { revision: 0, tasks: [], storage_warning: false },
	loading: true,
	error: false
});
let connection: ReturnType<typeof connectActivity> | undefined;
export function startActivities() {
	connection?.stop();
	connection = connectActivity(
		(changed) => listen('activity-changed', changed),
		() => invoke<ActivitySnapshot>('get_activity_tasks'),
		(snapshot) => {
			activityState.snapshot = snapshot;
			activityState.loading = false;
			activityState.error = false;
		},
		() => {
			activityState.error = true;
			activityState.loading = false;
		}
	);
	const current = connection;
	const focus = () => {
		void current.sync();
	};
	window.addEventListener('focus', focus);
	return () => {
		current.stop();
		window.removeEventListener('focus', focus);
		if (connection === current) connection = undefined;
	};
}
export const resyncActivities = () => connection?.sync();
export const cancelActivity = (taskId: string) => invoke<void>('cancel_activity_task', { taskId });
