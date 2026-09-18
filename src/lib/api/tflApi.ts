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
	TflSelectionEntry
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
export const getJavaStatus = () => invoke<JavaStatus[]>('get_java_status');

// ── Instances ────────────────────────────────────────────────────────
export const createInstance = (name: string, mcVersion: string, loader: Loader) =>
	invoke<InstanceData>('create_instance', { name, mcVersion, loader });
export const getInstances = () => invoke<InstanceData[]>('get_instances');
export const deleteInstance = (name: string) => invoke<void>('delete_instance', { name });
export const renameInstance = (oldName: string, newName: string) =>
	invoke<InstanceData>('rename_instance', { oldName, newName });
export const updateInstanceMemory = (
	name: string,
	minMemory: number | null,
	maxMemory: number | null
) => invoke<InstanceData>('update_instance_memory', { name, minMemory, maxMemory });
export const launchInstance = (instanceName: string) => invoke<void>('launch', { instanceName });
export const stopRunningInstance = () => invoke<void>('stop_running_instance');
export const getRunningInstance = () => invoke<string | null>('get_running_instance');
export const openInstanceFolder = (name: string) => invoke<void>('open_instance_folder', { name });
export const pickImageFile = () => invoke<string | null>('pick_image_file');
export const setInstanceIcon = (name: string, sourcePath: string) =>
	invoke<string>('set_instance_icon', { name, sourcePath });
export const getInstanceIconPath = (name: string) =>
	invoke<string | null>('get_instance_icon_path', { name });

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
export const searchMods = (query: string, mcVersion: string, loader: Loader) =>
	invoke<ModSearchHit[]>('search_mods', { query, mcVersion, loader });
export const installMod = (
	instanceName: string,
	projectId: string,
	mcVersion: string,
	loader: Loader
) => invoke<void>('install_mod', { instanceName, projectId, mcVersion, loader });
export const getInstanceMods = (instanceName: string) =>
	invoke<string[]>('get_instance_mods', { instanceName });
export const removeMod = (instanceName: string, filename: string) =>
	invoke<void>('remove_mod', { instanceName, filename });

// ── Shaders ──────────────────────────────────────────────────────────
export const searchShaders = (query: string, mcVersion: string) =>
	invoke<ModSearchHit[]>('search_shaders', { query, mcVersion });
export const installShader = (instanceName: string, projectId: string, mcVersion: string) =>
	invoke<void>('install_shader', { instanceName, projectId, mcVersion });
export const getInstanceShaders = (instanceName: string) =>
	invoke<string[]>('get_instance_shaders', { instanceName });
export const removeShader = (instanceName: string, filename: string) =>
	invoke<void>('remove_shader', { instanceName, filename });

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

// ── TFL Selection ────────────────────────────────────────────────────
export const getTflSelection = () => invoke<TflSelectionEntry[]>('get_tfl_selection');
