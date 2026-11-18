import { invoke } from './invoke';
import type { ImportCandidate, InstanceData, ServerType } from '$lib/types/types';

export interface ImportPreview {
	candidate: ImportCandidate & { loader_version?: string | null };
	format: string;
	importable: boolean;
	included: string[];
	omitted: string[];
	warnings: string[];
	file_count: number;
	size_bytes: number;
	fingerprint: string;
	server_type: ServerType | null;
}
export const inspectInstanceSource = (path: string) =>
	invoke<ImportPreview>('inspect_instance_source', { path });
export const pickImportDirectory = () => invoke<string | null>('pick_import_directory');
export const importInstanceSource = (
	path: string,
	newName: string,
	previewFingerprint: string,
	requestId: string
) =>
	invoke<InstanceData>('import_instance_source', { path, newName, previewFingerprint, requestId });
export const cancelInstanceImport = (requestId: string) =>
	invoke<boolean>('cancel_instance_import', { requestId });
