// Backups de mundos, exportación/importación de instancias
import { invoke } from './invoke';
import type { InstanceData, WorldBackup, ImportCandidate } from '$lib/types/types';

export const getWorldBackups = (instanceName: string) =>
	invoke<WorldBackup[]>('get_world_backups', { instanceName });
export const createWorldBackup = (instanceName: string) =>
	invoke<WorldBackup | null>('create_world_backup', { instanceName });
export const deleteWorldBackup = (instanceName: string, id: string) =>
	invoke<void>('delete_world_backup', { instanceName, id });
export const restoreWorldBackup = (instanceName: string, id: string) =>
	invoke<void>('restore_world_backup', { instanceName, id });
export const listWorldBackups = (instanceName: string) =>
	invoke<string[]>('list_world_backups', { instanceName });
export const exportInstanceAsMrpack = (instanceName: string) =>
	invoke<string>('export_instance_as_mrpack', { instanceName });
export const detectImportableInstances = () =>
	invoke<ImportCandidate[]>('detect_importable_instances');
export const importExternalInstance = (path: string, newName: string) =>
	invoke<InstanceData>('import_external_instance', { path, newName });
