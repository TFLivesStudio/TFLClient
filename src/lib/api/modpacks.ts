// Modpacks y TFL Selection
import { invoke } from './invoke';
import type {
	Loader,
	InstalledModpack,
	ModpackUpdateInfo,
	TflSelectionEntry,
	ContentSearchPage,
	ContentProjectDetail,
	SortIndex
} from '$lib/types/types';
import { DEFAULT_PAGE_SIZE } from './content';

export const searchModpacks = (
	query: string,
	mcVersion: string,
	loader: Loader,
	categories: string[] = [],
	offset = 0,
	limit = DEFAULT_PAGE_SIZE,
	index: SortIndex = 'relevance'
) =>
	invoke<ContentSearchPage>('search_modpacks', {
		query,
		mcVersion,
		loader,
		categories,
		offset,
		limit,
		index
	});
export const getModpackProjectDetail = (projectId: string) =>
	invoke<ContentProjectDetail>('get_modpack_project_detail', { projectId });
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
