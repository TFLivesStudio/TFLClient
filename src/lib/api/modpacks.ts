// Modpacks y TFL Selection
import { invoke } from './invoke';
import type {
	Loader,
	ModSearchHit,
	InstalledModpack,
	ModpackUpdateInfo,
	TflSelectionEntry
} from '$lib/types/types';

export const searchModpacks = (query: string, mcVersion: string, loader: Loader) =>
	invoke<ModSearchHit[]>('search_modpacks', { query, mcVersion, loader });
export const installModpack = (
	instanceName: string,
	projectId: string,
	mcVersion: string,
	loader: Loader
) => invoke<InstalledModpack>('install_modpack', { instanceName, projectId, mcVersion, loader });
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
export const getTflSelection = () => invoke<TflSelectionEntry[]>('get_tfl_selection');
