// Historial acotado de sesiones jugadas, para "Actividad reciente" en la home.
import { invoke } from './invoke';
import type { ActivityEntry } from '$lib/types/types';

export const getRecentActivity = (limit?: number) =>
	invoke<ActivityEntry[]>('get_recent_activity', { limit });
