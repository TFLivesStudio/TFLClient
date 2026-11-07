import { invoke } from './invoke';
import type { InstanceData } from '$lib/types/types';

export interface ChannelPointer {
	release: string;
	tag: string;
	manifest_sha256: string;
	published_at?: string;
}

export interface SelectionTarget {
	target_id: string;
	displayName?: string;
	summary?: string;
	minecraft_version: string;
	loader: string;
	sort_order?: number;
	channels: {
		stable?: ChannelPointer;
		beta?: ChannelPointer;
		dev?: ChannelPointer;
	};
}

export interface SelectionCatalog {
	schema_version: number;
	pack_id: string;
	generation: number;
	updated_at: string;
	targets: SelectionTarget[];
}

export interface SelectionIntegrityReport {
	is_valid: boolean;
	missing: string[];
	corrupted: string[];
	foreign_mods: string[];
}

export interface SelectionUpdateInfo {
	current_release: string;
	latest_release: string;
	channel: string;
	target_id: string;
}

export const getSelectionCatalog = (forceRefresh = false) =>
	invoke<SelectionCatalog>('get_selection_catalog', { forceRefresh });

export const installSelectionTarget = (instanceName: string, targetId: string, channel: string) =>
	invoke<InstanceData>('install_selection_target', { instanceName, targetId, channel });

export const verifySelectionIntegrity = (instanceName: string) =>
	invoke<SelectionIntegrityReport>('verify_selection_integrity', { instanceName });

export const repairSelectionInstance = (instanceName: string) =>
	invoke<void>('repair_selection_instance', { instanceName });

export const checkSelectionUpdate = (instanceName: string) =>
	invoke<SelectionUpdateInfo | null>('check_selection_update', { instanceName });
