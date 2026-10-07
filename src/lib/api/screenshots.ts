// Capturas de pantalla de instancia
import { invoke } from './invoke';
import type { ScreenshotInfo } from '$lib/types/types';

export const getInstanceScreenshots = (name: string) =>
	invoke<ScreenshotInfo[]>('get_instance_screenshots', { name });
export const deleteScreenshot = (name: string, filename: string) =>
	invoke<void>('delete_screenshot', { name, filename });
export const openScreenshotsFolder = (name: string) =>
	invoke<void>('open_screenshots_folder', { name });
