import { invoke } from '@tauri-apps/api/core';
import type {
	Settings,
	MinecraftUser,
	InstanceData,
	MinecraftVersion,
	DeviceCode,
	QualityProfile,
	Loader,
	ModSearchHit,
	RecommendedRam,
	JavaStatus,
	InstalledModpack,
	ModpackUpdateInfo,
	InstalledModInfo,
	ModUpdateAvailable,
	ModVersionSummary,
	TflSelectionEntry,
	ScreenshotInfo,
	ServerType,
	WorldInfo,
	FileEntry,
	ServerConnectionInfo
} from '$lib/types/types';

// ── Auth ─────────────────────────────────────────────────────────────
export const getDeviceCode = () => invoke<DeviceCode>('get_device_code');
export const authenticateWithDeviceCode = (
	deviceCode: string,
	interval: number,
	expiresIn: number
) =>
	invoke<MinecraftUser>('authenticate_with_device_code', {
		deviceCode,
		interval,
		expiresIn
	});
export const addOfflineAccount = (username: string) =>
	invoke<MinecraftUser>('add_offline_account', { username });
export const getCurrentUser = () => invoke<MinecraftUser>('get_current_user');
export const accountAllowsMultiplayer = () => invoke<boolean>('account_allows_multiplayer');
export const logout = () => invoke<void>('logout');
export const getUserList = () => invoke<MinecraftUser[]>('get_user_list');
export const switchUser = (uuid: string) => invoke<MinecraftUser>('switch_user', { uuid });
export const removeUser = (uuid: string) => invoke<void>('remove_user', { uuid });

// ── Settings ─────────────────────────────────────────────────────────
export const getSettings = () => invoke<Settings>('get_settings');
export const updateSettings = (settings: Settings) =>
	invoke<void>('update_settings', { newSettings: settings });
export const setQualityProfile = (profile: QualityProfile) =>
	invoke<Settings>('set_quality_profile', { profile });
export const getRecommendedRam = () => invoke<RecommendedRam>('get_recommended_ram');
export const getRecommendedRamForInstance = (name: string) =>
	invoke<RecommendedRam>('get_recommended_ram_for_instance', { name });
export const clearTempCache = () => invoke<number>('clear_temp_cache');
export const setCustomWallpaper = (sourcePath: string) =>
	invoke<string>('set_custom_wallpaper', { sourcePath });
export const getCustomWallpaperPath = () => invoke<string | null>('get_custom_wallpaper_path');
export const getJavaStatus = () => invoke<JavaStatus[]>('get_java_status');

// ── Instances ────────────────────────────────────────────────────────
export const createInstance = (name: string, mcVersion: string, loader: Loader) =>
	invoke<InstanceData>('create_instance', { name, mcVersion, loader });
export const getInstances = () => invoke<InstanceData[]>('get_instances');
export const deleteInstance = (name: string) => invoke<void>('delete_instance', { name });
export const renameInstance = (oldName: string, newName: string) =>
	invoke<InstanceData>('rename_instance', { oldName, newName });
export const duplicateInstance = (name: string) => invoke<InstanceData>('duplicate_instance', { name });
export const updateInstanceMemory = (
	name: string,
	minMemory: number | null,
	maxMemory: number | null
) => invoke<InstanceData>('update_instance_memory', { name, minMemory, maxMemory });
export const launchInstance = (instanceName: string) => invoke<void>('launch', { instanceName });
export const stopRunningInstance = () => invoke<void>('stop_running_instance');
export const getRunningInstance = () => invoke<string | null>('get_running_instance');
export const openInstanceFolder = (name: string) => invoke<void>('open_instance_folder', { name });
export const openExternalUrl = (url: string) => invoke<void>('open_external_url', { url });
export const pickImageFile = () => invoke<string | null>('pick_image_file');
export const setInstanceIcon = (name: string, sourcePath: string) =>
	invoke<string>('set_instance_icon', { name, sourcePath });
export const getInstanceIconPath = (name: string) =>
	invoke<string | null>('get_instance_icon_path', { name });
export const getInstanceScreenshots = (name: string) =>
	invoke<ScreenshotInfo[]>('get_instance_screenshots', { name });
export const deleteScreenshot = (name: string, filename: string) =>
	invoke<void>('delete_screenshot', { name, filename });
export const openScreenshotsFolder = (name: string) =>
	invoke<void>('open_screenshots_folder', { name });

// ── Versions / loaders ─────────────────────────────────────────────────
export const getAvailableVersions = () => invoke<MinecraftVersion[]>('get_available_versions');
export const getFabricLoader = (mcVersion: string) =>
	invoke<string>('get_fabric_loader', { mcVersion });
export const getQuiltLoader = (mcVersion: string) =>
	invoke<string>('get_quilt_loader', { mcVersion });
export const getForgeVersion = (mcVersion: string) =>
	invoke<string>('get_forge_version', { mcVersion });
export const getNeoforgeVersion = (mcVersion: string) =>
	invoke<string>('get_neoforge_version', { mcVersion });

// ── Mods ─────────────────────────────────────────────────────────────
export const searchMods = (query: string, mcVersion: string, loader: Loader, categories: string[] = []) =>
	invoke<ModSearchHit[]>('search_mods', { query, mcVersion, loader, categories });
export const installMod = (
	instanceName: string,
	projectId: string,
	mcVersion: string,
	loader: Loader,
	versionId?: string
) => invoke<void>('install_mod', { instanceName, projectId, mcVersion, loader, versionId: versionId ?? null });
export const getInstanceMods = (instanceName: string) =>
	invoke<string[]>('get_instance_mods', { instanceName });
export const removeMod = (instanceName: string, filename: string) =>
	invoke<void>('remove_mod', { instanceName, filename });
export const getInstalledModsInfo = (instanceName: string) =>
	invoke<InstalledModInfo[]>('get_installed_mods_info', { instanceName });
export const checkModUpdates = (instanceName: string) =>
	invoke<ModUpdateAvailable[]>('check_mod_updates', { instanceName });
export const updateAllMods = (instanceName: string) => invoke<number>('update_all_mods', { instanceName });
export const getModVersionChangelog = (versionId: string) =>
	invoke<string | null>('get_mod_version_changelog', { versionId });
export const findDuplicateMods = (instanceName: string) =>
	invoke<string[][]>('find_duplicate_mods', { instanceName });
export const getModVersions = (projectId: string, mcVersion: string, loader: Loader) =>
	invoke<ModVersionSummary[]>('get_mod_versions', { projectId, mcVersion, loader });

// ── Shaders ──────────────────────────────────────────────────────────
export const searchShaders = (query: string, mcVersion: string, categories: string[] = []) =>
	invoke<ModSearchHit[]>('search_shaders', { query, mcVersion, categories });
export const installShader = (
	instanceName: string,
	projectId: string,
	mcVersion: string,
	versionId?: string
) => invoke<void>('install_shader', { instanceName, projectId, mcVersion, versionId: versionId ?? null });
export const getInstanceShaders = (instanceName: string) =>
	invoke<string[]>('get_instance_shaders', { instanceName });
export const removeShader = (instanceName: string, filename: string) =>
	invoke<void>('remove_shader', { instanceName, filename });
export const getInstalledShadersInfo = (instanceName: string) =>
	invoke<InstalledModInfo[]>('get_installed_shaders_info', { instanceName });
export const getShaderVersions = (projectId: string, mcVersion: string) =>
	invoke<ModVersionSummary[]>('get_shader_versions', { projectId, mcVersion });

// ── Resource packs ───────────────────────────────────────────────────
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

// ── Plugins (instancias de servidor) ────────────────────────────────
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

// ── Agregar por archivo local (solo modo Manual, ver Settings) ────────
export const pickContentFiles = (extension: string, filterLabel: string) =>
	invoke<string[]>('pick_content_files', { extension, filterLabel });
export const addLocalModFiles = (instanceName: string, paths: string[]) =>
	invoke<number>('add_local_mod_files', { instanceName, paths });
export const addLocalShaderFiles = (instanceName: string, paths: string[]) =>
	invoke<number>('add_local_shader_files', { instanceName, paths });
export const addLocalResourcepackFiles = (instanceName: string, paths: string[]) =>
	invoke<number>('add_local_resourcepack_files', { instanceName, paths });

// ── Modpacks ─────────────────────────────────────────────────────────
export const searchModpacks = (query: string, mcVersion: string, loader: Loader) =>
	invoke<ModSearchHit[]>('search_modpacks', { query, mcVersion, loader });
export const installModpack = (
	instanceName: string,
	projectId: string,
	mcVersion: string,
	loader: Loader
) =>
	invoke<InstalledModpack>('install_modpack', { instanceName, projectId, mcVersion, loader });
export const getInstanceModpacks = (instanceName: string) =>
	invoke<InstalledModpack[]>('get_instance_modpacks', { instanceName });
export const removeModpack = (instanceName: string, versionId: string) =>
	invoke<void>('remove_modpack', { instanceName, versionId });
export const installModpackFromUrl = (instanceName: string, sourceId: string, mrpackUrl: string) =>
	invoke<InstalledModpack>('install_modpack_from_url', { instanceName, sourceId, mrpackUrl });
export const updateCommunityModpack = (
	instanceName: string,
	oldVersionId: string,
	sourceId: string,
	mrpackUrl: string
) =>
	invoke<InstalledModpack>('update_community_modpack', {
		instanceName,
		oldVersionId,
		sourceId,
		mrpackUrl
	});
export const checkModpackUpdates = (instanceName: string) =>
	invoke<ModpackUpdateInfo[]>('check_modpack_updates', { instanceName });
export const verifyInstanceIntegrity = (instanceName: string) =>
	invoke<string[]>('verify_instance_integrity', { instanceName });
export const exportInstanceAsMrpack = (instanceName: string) =>
	invoke<string>('export_instance_as_mrpack', { instanceName });
export const listWorldBackups = (instanceName: string) =>
	invoke<string[]>('list_world_backups', { instanceName });

// ── TFL Selection ────────────────────────────────────────────────────
export const getTflSelection = () => invoke<TflSelectionEntry[]>('get_tfl_selection');

// ── Servidores ───────────────────────────────────────────────────────
export const getServerVersions = (serverType: ServerType) =>
	invoke<string[]>('get_server_versions', { serverType });
export const getServerConnectionInfo = (instanceName: string) =>
	invoke<ServerConnectionInfo>('get_server_connection_info', { instanceName });
export const createServerInstance = (name: string, mcVersion: string, serverType: ServerType) =>
	invoke<InstanceData>('create_server_instance', { name, mcVersion, serverType });
export const launchServer = (instanceName: string) => invoke<void>('launch_server', { instanceName });
export const stopServer = (instanceName: string) => invoke<void>('stop_server', { instanceName });
export const sendServerCommand = (instanceName: string, command: string) =>
	invoke<void>('send_server_command', { instanceName, command });
export const isServerRunning = (instanceName: string) =>
	invoke<boolean>('is_server_running', { instanceName });
export const listServerWorlds = (instanceName: string) =>
	invoke<WorldInfo[]>('list_server_worlds', { instanceName });
export const deleteServerWorld = (instanceName: string, worldName: string) =>
	invoke<void>('delete_server_world', { instanceName, worldName });

// ── File manager de instancia (servidores) ──────────────────────────
export const listInstanceDir = (instanceName: string, subpath: string) =>
	invoke<FileEntry[]>('list_instance_dir', { instanceName, subpath });
export const readInstanceTextFile = (instanceName: string, subpath: string) =>
	invoke<string>('read_instance_text_file', { instanceName, subpath });
export const writeInstanceTextFile = (instanceName: string, subpath: string, content: string) =>
	invoke<void>('write_instance_text_file', { instanceName, subpath, content });
export const deleteInstancePath = (instanceName: string, subpath: string) =>
	invoke<void>('delete_instance_path', { instanceName, subpath });
export const createInstanceDir = (instanceName: string, subpath: string) =>
	invoke<void>('create_instance_dir', { instanceName, subpath });
