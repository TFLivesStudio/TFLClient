export type AccountType = 'Cracked' | 'Microsoft' | 'Yggdrasil';

export interface MinecraftUser {
	username: string;
	uuid: string;
	user_type: AccountType;
	yggdrasil_server_url?: string | null;
}

export type QualityProfile = 'Lite' | 'Balanced' | 'Experience';

export interface Settings {
	user: MinecraftUser[];
	active_user_idx: number;
	min_memory: number;
	max_memory: number;
	jre8_path: string;
	jre17_path: string;
	jre21_path: string;
	jre25_path: string;
	language: string;
	theme: string;
	quality_profile: QualityProfile;
	discord_presence: boolean;
	onboarded: boolean;
	jvm_args: string;
	env_vars: Record<string, string>;
	reduce_animations: boolean;
	disable_blur_effects: boolean;
	disable_infinite_animations: boolean;
}

export type Loader = 'vanilla' | 'fabric' | 'forge' | 'neoforge' | 'quilt';

export interface InstanceData {
	uuid: string;
	name: string;
	mc_version: string;
	loader: Loader;
	loader_version: string | null;
	launch_version_id: string;
	last_played: number;
	min_memory: number | null;
	max_memory: number | null;
}

export interface MinecraftVersion {
	id: string;
	type: string;
	release_time: string;
}

export interface DeviceCode {
	user_code: string;
	device_code: string;
	verification_uri: string;
	expires_in: number;
	interval: number;
}

export interface RecommendedRam {
	total_mb: number;
	recommended_min_mb: number;
	recommended_max_mb: number;
}

export interface JavaStatus {
	major: number;
	installed: boolean;
	path: string | null;
}

export interface ModSearchHit {
	project_id: string;
	title: string;
	description: string;
	icon_url: string | null;
	downloads: number;
	author: string;
}

export interface InstalledModInfo {
	filename: string;
	project_id: string | null;
	title: string | null;
	version_id: string | null;
}

export interface ModUpdateAvailable {
	filename: string;
	project_id: string;
	title: string;
	new_version_id: string;
}

export interface InstalledModpack {
	project_id: string;
	version_id: string;
	title: string;
	file_count: number;
}

export interface ModpackUpdateInfo {
	version_id: string;
	new_mrpack_url: string;
}

export interface ModpackVersion {
	mc_version: string;
	loader: string;
	mrpack_url: string;
}

export interface TflSelectionEntry {
	id: string;
	title: string;
	description: string;
	icon_url: string | null;
	source: 'modrinth' | 'community';
	project_id: string | null;
	versions: ModpackVersion[];
	category: string | null;
}

export interface DownloadProgressEvent {
	type: 'DownloadProgress';
	data: {
		task: string;
		stage: string;
		item_current: number;
		item_total: number;
		bytes_current: number;
		bytes_total: number;
		current_item: string | null;
	};
}

export interface DownloadFinishedEvent {
	type: 'DownloadFinished';
	data: { task: string };
}

export interface DownloadFailedEvent {
	type: 'DownloadFailed';
	data: { task: string; error: string };
}

export type AppEvent =
	| DownloadProgressEvent
	| DownloadFinishedEvent
	| DownloadFailedEvent
	| { type: string; data: unknown };

export interface ScreenshotInfo {
	filename: string;
	path: string;
	modified_ms: number;
}
