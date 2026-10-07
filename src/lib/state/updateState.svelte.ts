import { relaunch } from '@tauri-apps/plugin-process';
import { updateCheck, updateDownload, updateInstall } from '$lib/api/tflApi';
import type { UpdateInfo } from '$lib/types/types';

export const updateState = $state<{
	update: UpdateInfo | null;
	downloaded: boolean;
	installing: boolean;
	error: string | null;
}>({
	update: null,
	downloaded: false,
	installing: false,
	error: null
});

let checked = false;

/**
 * Chequea y descarga una actualización en segundo plano, sin bloquear ni
 * avisar nada hasta que esté lista para instalar — pensado para llamarse
 * una vez al abrir el launcher (gateado por Settings.auto_updates). Errores
 * de red/endpoint se tragan en silencio (mismo criterio que el chequeo
 * manual de Ajustes: no molestar si no hay conexión, no es un error del
 * usuario). `checked` evita correrlo dos veces si el componente que lo
 * dispara se remonta. `beta` elige el canal (Settings.beta_updates).
 */
export async function checkAndDownloadUpdate(beta: boolean): Promise<void> {
	if (checked) return;
	checked = true;

	try {
		const update = await updateCheck(beta);
		if (!update) return;
		await updateDownload();
		updateState.update = update;
		updateState.downloaded = true;
	} catch (e) {
		console.error('Chequeo automático de actualización falló:', e);
	}
}

export async function installDownloadedUpdate(): Promise<void> {
	if (!updateState.update || updateState.installing) return;
	updateState.installing = true;
	updateState.error = null;
	try {
		await updateInstall();
		await relaunch();
	} catch (e) {
		updateState.error = String(e);
		updateState.installing = false;
	}
}

export function dismissUpdateBadge(): void {
	updateState.downloaded = false;
}

/** Chequeo manual (Ajustes) en el canal pedido. */
export function checkForUpdateNow(beta: boolean): Promise<UpdateInfo | null> {
	return updateCheck(beta);
}

/** Descarga e instala la actualización encontrada por `checkForUpdateNow` y reinicia. */
export async function downloadAndInstallNow(): Promise<void> {
	await updateDownload();
	await updateInstall();
	await relaunch();
}
