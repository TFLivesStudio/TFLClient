// Contenido de instancia: mods, shaders, resource packs y plugins (búsqueda, instalación, info, actualizaciones, rollback, duplicados)
import { invoke } from './invoke';
import type {
	Loader,
	ModSearchHit,
	InstalledModInfo,
	ModUpdateAvailable,
	ModVersionSummary,
	ServerType,
	ModRollbackInfo
} from '$lib/types/types';

export const searchMods = (
	query: string,
	mcVersion: string,
	loader: Loader,
	categories: string[] = []
) => invoke<ModSearchHit[]>('search_mods', { query, mcVersion, loader, categories });
export const installMod = (
	instanceName: string,
	projectId: string,
	mcVersion: string,
	loader: Loader,
	versionId?: string
) =>
	invoke<void>('install_mod', {
		instanceName,
		projectId,
		mcVersion,
		loader,
		versionId: versionId ?? null
	});
export const getInstanceMods = (instanceName: string) =>
	invoke<string[]>('get_instance_mods', { instanceName });
export const removeMod = (instanceName: string, filename: string) =>
	invoke<void>('remove_mod', { instanceName, filename });
export const getInstalledModsInfo = (instanceName: string) =>
	invoke<InstalledModInfo[]>('get_installed_mods_info', { instanceName });
export const checkModUpdates = (instanceName: string) =>
	invoke<ModUpdateAvailable[]>('check_mod_updates', { instanceName });
export const updateAllMods = (instanceName: string) =>
	invoke<number>('update_all_mods', { instanceName });
export const getModVersionChangelog = (versionId: string) =>
	invoke<string | null>('get_mod_version_changelog', { versionId });
export const findDuplicateMods = (instanceName: string) =>
	invoke<string[][]>('find_duplicate_mods', { instanceName });
export const removeDuplicateMods = (instanceName: string) =>
	invoke<number>('remove_duplicate_mods', { instanceName });
export const setModEnabled = (instanceName: string, filename: string, enabled: boolean) =>
	invoke<string>('set_mod_enabled', { instanceName, filename, enabled });
export const setPluginEnabled = (instanceName: string, filename: string, enabled: boolean) =>
	invoke<string>('set_plugin_enabled', { instanceName, filename, enabled });
export const getModRollbackInfo = (instanceName: string) =>
	invoke<ModRollbackInfo | null>('get_mod_rollback_info', { instanceName });
export const rollbackModUpdate = (instanceName: string) =>
	invoke<number>('rollback_mod_update', { instanceName });
export const discardModRollback = (instanceName: string) =>
	invoke<void>('discard_mod_rollback', { instanceName });
export const getModVersions = (projectId: string, mcVersion: string, loader: Loader) =>
	invoke<ModVersionSummary[]>('get_mod_versions', { projectId, mcVersion, loader });
export const searchShaders = (query: string, mcVersion: string, categories: string[] = []) =>
	invoke<ModSearchHit[]>('search_shaders', { query, mcVersion, categories });
export const installShader = (
	instanceName: string,
	projectId: string,
	mcVersion: string,
	versionId?: string
) =>
	invoke<void>('install_shader', {
		instanceName,
		projectId,
		mcVersion,
		versionId: versionId ?? null
	});
export const getInstanceShaders = (instanceName: string) =>
	invoke<string[]>('get_instance_shaders', { instanceName });
export const removeShader = (instanceName: string, filename: string) =>
	invoke<void>('remove_shader', { instanceName, filename });
export const getInstalledShadersInfo = (instanceName: string) =>
	invoke<InstalledModInfo[]>('get_installed_shaders_info', { instanceName });
export const getShaderVersions = (projectId: string, mcVersion: string) =>
	invoke<ModVersionSummary[]>('get_shader_versions', { projectId, mcVersion });
export const searchResourcepacks = (query: string, mcVersion: string, categories: string[] = []) =>
	invoke<ModSearchHit[]>('search_resourcepacks', { query, mcVersion, categories });
export const installResourcepack = (
	instanceName: string,
	projectId: string,
	mcVersion: string,
	versionId?: string
) =>
	invoke<void>('install_resourcepack', {
		instanceName,
		projectId,
		mcVersion,
		versionId: versionId ?? null
	});
export const getInstanceResourcepacks = (instanceName: string) =>
	invoke<string[]>('get_instance_resourcepacks', { instanceName });
export const removeResourcepack = (instanceName: string, filename: string) =>
	invoke<void>('remove_resourcepack', { instanceName, filename });
export const getInstalledResourcepacksInfo = (instanceName: string) =>
	invoke<InstalledModInfo[]>('get_installed_resourcepacks_info', { instanceName });
export const getResourcepackVersions = (projectId: string, mcVersion: string) =>
	invoke<ModVersionSummary[]>('get_resourcepack_versions', { projectId, mcVersion });
export const searchPlugins = (
	query: string,
	mcVersion: string,
	serverType: ServerType,
	categories: string[] = []
) => invoke<ModSearchHit[]>('search_plugins', { query, mcVersion, serverType, categories });
export const installPlugin = (
	instanceName: string,
	projectId: string,
	mcVersion: string,
	serverType: ServerType,
	versionId?: string
) =>
	invoke<void>('install_plugin', {
		instanceName,
		projectId,
		mcVersion,
		serverType,
		versionId: versionId ?? null
	});
export const getInstancePlugins = (instanceName: string) =>
	invoke<string[]>('get_instance_plugins', { instanceName });
export const removePlugin = (instanceName: string, filename: string) =>
	invoke<void>('remove_plugin', { instanceName, filename });
export const getInstalledPluginsInfo = (instanceName: string) =>
	invoke<InstalledModInfo[]>('get_installed_plugins_info', { instanceName });
export const getPluginVersions = (projectId: string, mcVersion: string, serverType: ServerType) =>
	invoke<ModVersionSummary[]>('get_plugin_versions', { projectId, mcVersion, serverType });
export const addLocalPluginFiles = (instanceName: string, paths: string[]) =>
	invoke<number>('add_local_plugin_files', { instanceName, paths });
export const addLocalModFiles = (instanceName: string, paths: string[]) =>
	invoke<number>('add_local_mod_files', { instanceName, paths });
export const addLocalShaderFiles = (instanceName: string, paths: string[]) =>
	invoke<number>('add_local_shader_files', { instanceName, paths });
export const addLocalResourcepackFiles = (instanceName: string, paths: string[]) =>
	invoke<number>('add_local_resourcepack_files', { instanceName, paths });
