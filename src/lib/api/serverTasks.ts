import { invoke } from './invoke';
export interface ServerTask {
	id: string;
	instance_name: string;
	mc_version: string;
	server_type: string;
	status: 'preparing' | 'downloading' | 'ready' | 'failed' | 'cancelled' | 'interrupted';
	bytes_current: number;
	bytes_total: number | null;
	error: string | null;
}
export const listServerTasks = () => invoke<ServerTask[]>('list_server_tasks');
export const cancelServerTask = (id: string) => invoke<void>('cancel_server_task', { id });
export const retryServerTask = (id: string) => invoke<void>('retry_server_task', { id });
