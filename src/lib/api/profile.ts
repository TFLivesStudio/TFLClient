// Perfil de Mojang (skin / capa)
import { invoke } from './invoke';
import type { MojangProfile } from '$lib/types/types';

export const getMojangProfile = () => invoke<MojangProfile>('get_mojang_profile');
export const setSkinFromFile = (path: string, variant: 'classic' | 'slim') =>
	invoke<void>('set_skin_from_file', { path, variant });
export const resetSkin = () => invoke<void>('reset_skin');
export const setActiveCape = (capeId: string) => invoke<void>('set_active_cape', { capeId });
export const hideCape = () => invoke<void>('hide_cape');
