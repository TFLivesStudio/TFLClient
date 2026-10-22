import type { SceneId } from './model';
export const CINEMATIC_SCENES = ['forest', 'embers', 'void', 'aurora', 'pixel'] as const;
export function cinematicPoster(id: SceneId, thumbnail = false) {
	return (CINEMATIC_SCENES as readonly string[]).includes(id)
		? `/scenes/cinematic/${id}${thumbnail ? '-thumb' : ''}.webp`
		: null;
}
