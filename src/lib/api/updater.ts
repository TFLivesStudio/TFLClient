import { invoke } from './invoke';
import type { UpdateInfo } from '$lib/types/types';

// Actualizaciones del launcher. El canal (estable/beta) lo elige el backend
// según el parámetro `beta` (Settings.beta_updates); ver commands/updater.rs.
export const updateCheck = (beta: boolean) => invoke<UpdateInfo | null>('update_check', { beta });
export const updateDownload = () => invoke<void>('update_download');
export const updateInstall = () => invoke<void>('update_install');
