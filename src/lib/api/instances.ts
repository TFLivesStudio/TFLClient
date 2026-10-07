// Instancias de cliente: CRUD, lanzamiento, iconos, integridad y accesos directos
import { invoke } from './invoke';
import type { InstanceData, Loader, RunningInstanceInfo, CrashDiagnosis } from '$lib/types/types';

export const createInstance = (name: string, mcVersion: string, loader: Loader) =>
	invoke<InstanceData>('create_instance', { name, mcVersion, loader });
export const getInstances = () => invoke<InstanceData[]>('get_instances');
export const deleteInstance = (name: string) => invoke<void>('delete_instance', { name });
export const renameInstance = (oldName: string, newName: string) =>
	invoke<InstanceData>('rename_instance', { oldName, newName });
export const duplicateInstance = (name: string) =>
	invoke<InstanceData>('duplicate_instance', { name });
export const updateInstanceMemory = (
	name: string,
	minMemory: number | null,
	maxMemory: number | null
) => invoke<InstanceData>('update_instance_memory', { name, minMemory, maxMemory });
export const launchInstance = (instanceName: string, serverAddress?: string) =>
	invoke<void>('launch', { instanceName, serverAddress: serverAddress ?? null });
export const stopRunningInstance = (instanceName: string) =>
	invoke<void>('stop_running_instance', { instanceName });
export const getRunningInstances = () => invoke<RunningInstanceInfo[]>('get_running_instances');
export const openInstanceFolder = (name: string) => invoke<void>('open_instance_folder', { name });
export const setInstanceIcon = (name: string, sourcePath: string) =>
	invoke<string>('set_instance_icon', { name, sourcePath });
export const getInstanceIconPath = (name: string) =>
	invoke<string | null>('get_instance_icon_path', { name });
export const verifyInstanceIntegrity = (instanceName: string) =>
	invoke<string[]>('verify_instance_integrity', { instanceName });
export const diagnoseCrash = (instanceName: string) =>
	invoke<CrashDiagnosis>('diagnose_crash', { instanceName });
export const createInstanceShortcut = (instanceName: string) =>
	invoke<string>('create_instance_shortcut', { instanceName });
