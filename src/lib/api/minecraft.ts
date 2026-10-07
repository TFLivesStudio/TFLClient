// Versiones de Minecraft, loaders y Java
import { invoke } from './invoke';
import type { MinecraftVersion, JavaStatus } from '$lib/types/types';

export const getJavaStatus = () => invoke<JavaStatus[]>('get_java_status');
export const getAvailableVersions = () => invoke<MinecraftVersion[]>('get_available_versions');
export const getFabricLoader = (mcVersion: string) =>
	invoke<string>('get_fabric_loader', { mcVersion });
export const getQuiltLoader = (mcVersion: string) =>
	invoke<string>('get_quilt_loader', { mcVersion });
export const getForgeVersion = (mcVersion: string) =>
	invoke<string>('get_forge_version', { mcVersion });
export const getNeoforgeVersion = (mcVersion: string) =>
	invoke<string>('get_neoforge_version', { mcVersion });
