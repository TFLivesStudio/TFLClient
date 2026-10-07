// Utilidades del sistema (caché, URLs externas, selectores de archivos)
import { invoke } from './invoke';

export const clearTempCache = () => invoke<number>('clear_temp_cache');
export const openExternalUrl = (url: string) => invoke<void>('open_external_url', { url });
export const pickImageFile = () => invoke<string | null>('pick_image_file');
export const pickContentFiles = (extension: string, filterLabel: string) =>
	invoke<string[]>('pick_content_files', { extension, filterLabel });
