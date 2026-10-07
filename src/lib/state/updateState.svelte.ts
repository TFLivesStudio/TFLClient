import { listen } from '@tauri-apps/api/event';
import { relaunch } from '@tauri-apps/plugin-process';
import { updateCheck, updateDownload, updateInstall } from '$lib/api';
import { appState } from '$lib/state/state.svelte';
import type { UpdateInfo } from '$lib/types/types';
import {
	INITIAL_UPDATE_SNAPSHOT,
	canStartCheck,
	reduceUpdate,
	retryStage,
	type UpdateEvent,
	type UpdateSnapshot
} from '$lib/state/updateMachine';

// Store reactivo del actualizador. La lógica de transiciones está en
// updateMachine.ts (pura); acá solo se habla con el plugin de Tauri y se
// despachan eventos. Flujo: chequeo en segundo plano (nunca bloquea la UI) →
// descarga en segundo plano con progreso → "Reiniciar para actualizar".
// El plugin no puede aplicar una actualización "al próximo reinicio" por sí
// solo (`install()` hay que llamarlo explícitamente), así que la actualización
// se instala cuando el usuario toca el botón de reiniciar.

export const updateState = $state<UpdateSnapshot & { dismissed: boolean }>({
	...INITIAL_UPDATE_SNAPSHOT,
	dismissed: false
});

// El paquete de la actualización vive del lado Rust (commands/updater.rs);
// acá solo se sabe si hay una pendiente.
let hasPendingUpdate = false;
let autoCheckStarted = false;
let lastAutoDownload = false;

const CHECK_TIMEOUT_MS = 30_000;

function dispatch(event: UpdateEvent): void {
	const previousPhase = updateState.phase;
	const next = reduceUpdate(updateState, event);
	if (next === updateState) return;
	Object.assign(updateState, next);
	// Un badge cerrado reaparece cuando cambia la fase (p. ej. terminó la descarga).
	if (next.phase !== previousPhase) updateState.dismissed = false;
}

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
	return new Promise<T>((resolve, reject) => {
		const timer = setTimeout(() => reject(new Error('Tiempo de espera agotado')), ms);
		promise.then(
			(value) => {
				clearTimeout(timer);
				resolve(value);
			},
			(error) => {
				clearTimeout(timer);
				reject(error);
			}
		);
	});
}

function errorMessage(e: unknown): string {
	return e instanceof Error ? e.message : String(e);
}

/**
 * Chequea si hay una actualización y, con `autoDownload`, la descarga en
 * segundo plano. Nunca lanza: los fallos quedan en `updateState.error`.
 * `autoDownload` lo decide quien llama según `Settings.auto_updates`.
 */
export async function checkForUpdates(options: {
	userInitiated: boolean;
	autoDownload: boolean;
}): Promise<void> {
	if (!canStartCheck(updateState)) return;
	lastAutoDownload = options.autoDownload;
	dispatch({ type: 'CHECK_START', userInitiated: options.userInitiated });

	let update: UpdateInfo | null;
	try {
		// El canal (estable/beta) sale del ajuste actual: así sirve igual para el
		// chequeo automático, el manual y los reintentos.
		update = await withTimeout(
			updateCheck(appState.settings?.beta_updates === true),
			CHECK_TIMEOUT_MS
		);
	} catch (e) {
		console.error('Chequeo de actualización falló:', e);
		dispatch({ type: 'CHECK_FAILED', message: errorMessage(e) });
		return;
	}
	if (!update) {
		dispatch({ type: 'CHECK_NONE' });
		return;
	}
	hasPendingUpdate = true;
	dispatch({ type: 'CHECK_FOUND', version: update.version });
	if (options.autoDownload) await downloadUpdate();
}

/**
 * Chequeo automático al abrir el launcher (gateado por Settings.auto_updates
 * en quien llama). `autoCheckStarted` evita correrlo dos veces si el
 * componente que lo dispara se remonta. Un fallo de red acá no molesta al
 * usuario: no hay badge, solo se ve en Ajustes.
 */
export async function checkAndDownloadUpdate(): Promise<void> {
	if (autoCheckStarted) return;
	autoCheckStarted = true;
	// En `tauri dev` el binario es de desarrollo: bajar e instalar el release
	// oficial encima lo pisaría. El chequeo manual de Ajustes sigue disponible.
	if (import.meta.env.DEV) {
		console.info('[updater] chequeo automático omitido en desarrollo');
		return;
	}
	await checkForUpdates({ userInitiated: false, autoDownload: true });
}

export async function downloadUpdate(): Promise<void> {
	if (!hasPendingUpdate) return;
	dispatch({ type: 'DOWNLOAD_START' });
	if (updateState.phase !== 'downloading') return;

	// El backend avisa el progreso acumulado; la máquina de estados espera
	// "bytes de este tramo", así que se pasa la diferencia.
	let lastDownloaded = 0;
	let totalAnnounced = false;
	const unlisten = await listen<{ downloaded: number; total: number | null }>(
		'update-download-progress',
		(event) => {
			const { downloaded, total } = event.payload;
			if (!totalAnnounced && total !== null) {
				totalAnnounced = true;
				dispatch({ type: 'DOWNLOAD_TOTAL', totalBytes: total });
			}
			if (downloaded > lastDownloaded) {
				dispatch({ type: 'DOWNLOAD_PROGRESS', chunkBytes: downloaded - lastDownloaded });
				lastDownloaded = downloaded;
			}
		}
	);
	try {
		await updateDownload();
		dispatch({ type: 'DOWNLOAD_DONE' });
	} catch (e) {
		console.error('Descarga de actualización falló:', e);
		dispatch({ type: 'DOWNLOAD_FAILED', message: errorMessage(e) });
	} finally {
		unlisten();
	}
}

/** Instala la actualización ya descargada y reinicia el launcher. */
export async function installDownloadedUpdate(): Promise<void> {
	if (!hasPendingUpdate) return;
	dispatch({ type: 'INSTALL_START' });
	if (updateState.phase !== 'installing') return;
	try {
		await updateInstall();
		await relaunch();
	} catch (e) {
		console.error('Instalación de actualización falló:', e);
		dispatch({ type: 'INSTALL_FAILED', message: errorMessage(e) });
	}
}

/** Repite el paso que falló (chequeo, descarga o instalación). */
export async function retryUpdate(): Promise<void> {
	switch (retryStage(updateState)) {
		case 'check':
			await checkForUpdates({ userInitiated: true, autoDownload: lastAutoDownload });
			break;
		case 'download':
			await downloadUpdate();
			break;
		case 'install':
			await installDownloadedUpdate();
			break;
	}
}

export function dismissUpdateBadge(): void {
	updateState.dismissed = true;
}
