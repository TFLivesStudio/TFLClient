// Máquina de estados pura del actualizador (sin Tauri, sin runes): todas las
// transiciones viven acá para poder razonarlas/probarlas aparte. El store
// reactivo (updateState.svelte.ts) solo despacha eventos y aplica el resultado.
//
//   none ──check──> checking ──found──> available ──download──> downloading
//     ^                 │                   │                        │
//     └───up-to-date────┘                   │                        v
//                                           │                   downloaded ──install──> installing
//   error <──(cualquier paso falla)─────────┴────────────────────────┘                       │
//     └── retry: repite el paso que falló (check / download / install) <──────────────────────┘

export type UpdatePhase =
	'none' | 'checking' | 'available' | 'downloading' | 'downloaded' | 'installing' | 'error';

export type UpdateStage = 'check' | 'download' | 'install';

export interface UpdateSnapshot {
	phase: UpdatePhase;
	/** Versión de la actualización encontrada (null mientras no hay ninguna). */
	version: string | null;
	downloadedBytes: number;
	/** Tamaño total si el servidor lo informa; null = progreso indeterminado. */
	totalBytes: number | null;
	/** Paso que falló + mensaje; solo con phase === 'error'. */
	error: { stage: UpdateStage; message: string } | null;
	/** El último chequeo terminó sin encontrar nada (para "ya tenés la última"). */
	upToDate: boolean;
	/** El chequeo lo pidió el usuario (Ajustes), no el arranque automático. */
	userInitiated: boolean;
}

export type UpdateEvent =
	| { type: 'CHECK_START'; userInitiated: boolean }
	| { type: 'CHECK_NONE' }
	| { type: 'CHECK_FOUND'; version: string }
	| { type: 'CHECK_FAILED'; message: string }
	| { type: 'DOWNLOAD_START' }
	| { type: 'DOWNLOAD_TOTAL'; totalBytes: number | null }
	| { type: 'DOWNLOAD_PROGRESS'; chunkBytes: number }
	| { type: 'DOWNLOAD_DONE' }
	| { type: 'DOWNLOAD_FAILED'; message: string }
	| { type: 'INSTALL_START' }
	| { type: 'INSTALL_FAILED'; message: string };

export const INITIAL_UPDATE_SNAPSHOT: UpdateSnapshot = {
	phase: 'none',
	version: null,
	downloadedBytes: 0,
	totalBytes: null,
	error: null,
	upToDate: false,
	userInitiated: false
};

/** Transición pura. Un evento inválido para la fase actual se ignora (devuelve `s`). */
export function reduceUpdate(s: UpdateSnapshot, e: UpdateEvent): UpdateSnapshot {
	switch (e.type) {
		case 'CHECK_START':
			if (!canStartCheck(s)) return s;
			return {
				...INITIAL_UPDATE_SNAPSHOT,
				phase: 'checking',
				userInitiated: e.userInitiated
			};
		case 'CHECK_NONE':
			if (s.phase !== 'checking') return s;
			return { ...s, phase: 'none', upToDate: true };
		case 'CHECK_FOUND':
			if (s.phase !== 'checking') return s;
			return { ...s, phase: 'available', version: e.version, upToDate: false };
		case 'CHECK_FAILED':
			if (s.phase !== 'checking') return s;
			return { ...s, phase: 'error', error: { stage: 'check', message: e.message } };
		case 'DOWNLOAD_START':
			if (s.phase !== 'available' && !(s.phase === 'error' && s.error?.stage === 'download'))
				return s;
			return { ...s, phase: 'downloading', downloadedBytes: 0, totalBytes: null, error: null };
		case 'DOWNLOAD_TOTAL':
			if (s.phase !== 'downloading') return s;
			return { ...s, totalBytes: e.totalBytes };
		case 'DOWNLOAD_PROGRESS':
			if (s.phase !== 'downloading') return s;
			return { ...s, downloadedBytes: s.downloadedBytes + e.chunkBytes };
		case 'DOWNLOAD_DONE':
			if (s.phase !== 'downloading') return s;
			return { ...s, phase: 'downloaded' };
		case 'DOWNLOAD_FAILED':
			if (s.phase !== 'downloading') return s;
			return { ...s, phase: 'error', error: { stage: 'download', message: e.message } };
		case 'INSTALL_START':
			if (s.phase !== 'downloaded' && !(s.phase === 'error' && s.error?.stage === 'install'))
				return s;
			return { ...s, phase: 'installing', error: null };
		case 'INSTALL_FAILED':
			if (s.phase !== 'installing') return s;
			return { ...s, phase: 'error', error: { stage: 'install', message: e.message } };
	}
}

/** Un chequeo nuevo solo arranca "en frío" o reintentando un chequeo fallido. */
export function canStartCheck(s: UpdateSnapshot): boolean {
	return s.phase === 'none' || (s.phase === 'error' && s.error?.stage === 'check');
}

/** Qué paso repite el botón "Reintentar" (null si no hay error). */
export function retryStage(s: UpdateSnapshot): UpdateStage | null {
	return s.phase === 'error' && s.error ? s.error.stage : null;
}

/** 0–100, o null si no se conoce el tamaño total (progreso indeterminado). */
export function progressPercent(s: UpdateSnapshot): number | null {
	if (s.totalBytes === null || s.totalBytes <= 0) return null;
	return Math.min(100, Math.floor((s.downloadedBytes / s.totalBytes) * 100));
}

/**
 * ¿Debe mostrarse el badge flotante? El estado 'none'/'checking' nunca lo
 * muestra (chequeo en segundo plano silencioso). Un fallo de chequeo
 * automático tampoco: sin conexión no es un error del usuario — solo se ve
 * en Ajustes. `dismissed` oculta el badge hasta que cambie la fase.
 */
export function isBadgeVisible(s: UpdateSnapshot, dismissed: boolean): boolean {
	if (dismissed) return false;
	switch (s.phase) {
		case 'available':
		case 'downloading':
		case 'downloaded':
		case 'installing':
			return true;
		case 'error':
			return s.error?.stage !== 'check' || s.userInitiated;
		default:
			return false;
	}
}
