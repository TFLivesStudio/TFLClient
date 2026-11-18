import { invoke } from './invoke';

export interface ManagedWorld {
	name: string;
	display_name: string;
	version: string | null;
	data_version: number | null;
	size_bytes: number | null;
	metadata_error: string | null;
	dimensions: string[];
	server_primary: boolean;
	missing: boolean;
}
export interface ManagedWorldBackup {
	id: string;
	created_at: number;
	size_bytes: number;
	sha256: string;
}
export interface WorldDatapack {
	id: string;
	enabled: boolean;
	pack_format: number | null;
	min_format: unknown;
	max_format: unknown;
	world_version: string | null;
	compatibility: 'unknown' | 'compatible' | 'incompatible';
	error: string | null;
}
export interface DatapackHit {
	project_id: string;
	title: string;
	description: string;
	author: string;
}

export const listInstanceWorlds = (instanceName: string) =>
	invoke<ManagedWorld[]>('list_instance_worlds', { instanceName });
export const renameInstanceWorld = (instanceName: string, worldName: string, displayName: string) =>
	invoke<void>('rename_instance_world', { instanceName, worldName, displayName });
export const deleteInstanceWorld = (instanceName: string, worldName: string) =>
	invoke<void>('delete_instance_world', { instanceName, worldName });
export const backupInstanceWorld = (instanceName: string, worldName: string) =>
	invoke<ManagedWorldBackup>('backup_instance_world', { instanceName, worldName });
export const listInstanceWorldBackups = (instanceName: string, worldName: string) =>
	invoke<ManagedWorldBackup[]>('list_instance_world_backups', { instanceName, worldName });
export const restoreInstanceWorld = (instanceName: string, worldName: string, id: string) =>
	invoke<void>('restore_instance_world', { instanceName, worldName, id });
export const listWorldDatapacks = (instanceName: string, worldName: string) =>
	invoke<WorldDatapack[]>('list_world_datapacks', { instanceName, worldName });
export const installWorldDatapack = (instanceName: string, worldName: string, path: string) =>
	invoke<void>('install_world_datapack', { instanceName, worldName, path });
export const changeWorldDatapack = (
	instanceName: string,
	worldName: string,
	id: string,
	enabled: boolean,
	remove: boolean
) => invoke<void>('change_world_datapack', { instanceName, worldName, id, enabled, remove });
export const searchWorldDatapacks = (instanceName: string, worldName: string, query: string) =>
	invoke<DatapackHit[]>('search_world_datapacks', { instanceName, worldName, query });
export const downloadWorldDatapack = (instanceName: string, worldName: string, projectId: string) =>
	invoke<void>('download_world_datapack', { instanceName, worldName, projectId });
