// Ajustes, RAM recomendada y fondo personalizado
import { invoke } from './invoke';
import type { Settings, QualityProfile, RecommendedRam } from '$lib/types/types';

export const getSettings = () => invoke<Settings>('get_settings');
export const updateSettings = (settings: Settings) =>
	invoke<void>('update_settings', { newSettings: settings });
export const setQualityProfile = (profile: QualityProfile) =>
	invoke<Settings>('set_quality_profile', { profile });
export const getRecommendedRam = () => invoke<RecommendedRam>('get_recommended_ram');
export const getRecommendedRamForInstance = (name: string) =>
	invoke<RecommendedRam>('get_recommended_ram_for_instance', { name });
export const setCustomWallpaper = (sourcePath: string) =>
	invoke<string>('set_custom_wallpaper', { sourcePath });
export const getCustomWallpaperPath = () => invoke<string | null>('get_custom_wallpaper_path');
