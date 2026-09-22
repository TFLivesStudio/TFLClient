import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export const updateState = $state<{
	update: Update | null;
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
 * dispara se remonta.
 */
export async function checkAndDownloadUpdate(): Promise<void> {
	if (checked) return;
	checked = true;

	try {
		const update = await check();
		if (!update) return;
		await update.download();
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
		await updateState.update.install();
		await relaunch();
	} catch (e) {
		updateState.error = String(e);
		updateState.installing = false;
	}
}

export function dismissUpdateBadge(): void {
	updateState.downloaded = false;
}
