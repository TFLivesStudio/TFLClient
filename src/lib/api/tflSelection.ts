import { invoke } from './invoke';
import type { InstanceData } from '$lib/types/types';

export type SelectionChannelName = 'stable' | 'beta' | 'dev';

export interface SelectionChannelView {
	channel: SelectionChannelName;
	release: string;
	tag: string;
	manifestSha256: string;
}

/** Espeja `catalog::Support` del backend: siempre viene uno de los dos. */
export type SelectionSupport =
	{ state: 'supported' } | { state: 'launcher-update-required'; reason: string };

/** Espeja `catalog::TargetView` — ya filtrado/gateado en el backend: nunca
 * trae un target sin canal publicado, en canal dev, ni con loader que este
 * launcher no sabe instalar (eso lo indica `support`, no una ausencia). */
export interface SelectionTarget {
	targetId: string;
	displayName: string;
	summary: string;
	minecraftVersion: string;
	loader: string;
	sortOrder: number;
	channels: SelectionChannelView[];
	support: SelectionSupport;
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
	invoke<SelectionTarget[]>('get_selection_catalog', { forceRefresh });

export const installSelectionTarget = (instanceName: string, targetId: string, channel: string) =>
	invoke<InstanceData>('install_selection_target', { instanceName, targetId, channel });

export const verifySelectionIntegrity = (instanceName: string) =>
	invoke<SelectionIntegrityReport>('verify_selection_integrity', { instanceName });

export const repairSelectionInstance = (instanceName: string) =>
	invoke<void>('repair_selection_instance', { instanceName });

export const checkSelectionUpdate = (instanceName: string) =>
	invoke<SelectionUpdateInfo | null>('check_selection_update', { instanceName });
