// Utilidades del sistema (caché, URLs externas, selectores de archivos)
import { invoke } from './invoke';
import type { SystemStatus } from '$lib/types/types';

export const clearTempCache = () => invoke<number>('clear_temp_cache');
export const getSystemStatus = () => invoke<SystemStatus>('get_system_status');
export const openExternalUrl = (url: string) => invoke<void>('open_external_url', { url });
export const pickImageFile = () => invoke<string | null>('pick_image_file');
export const pickContentFiles = (extension: string, filterLabel: string) =>
	invoke<string[]>('pick_content_files', { extension, filterLabel });
