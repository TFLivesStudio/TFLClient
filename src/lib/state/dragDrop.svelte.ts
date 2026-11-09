import { getCurrentWebview } from '@tauri-apps/api/webview';
import { openMrpackDrop } from '$lib/state/uiState.svelte';

// Arrastrar y soltar un .mrpack sobre la ventana: Tauri intercepta el drag a
// nivel OS (dragDropEnabled queda en su default `true`, no hace falta tocar
// tauri.conf.json) y lo entrega acá como ruta de archivo absoluta — no pasa
// por el picker nativo (pick_content_files), así que no lo bloquea
// `native_dialog_mode` (ese gate existe solo por el crash del diálogo nativo
// en macOS, no por manejar archivos en sí).
export const dragDrop = $state<{ over: boolean }>({ over: false });

let initialized = false;

export function initDragDropListener() {
	if (initialized) return;
	initialized = true;
	void getCurrentWebview().onDragDropEvent((event) => {
		const { type } = event.payload;
		if (type === 'enter' || type === 'over') {
			dragDrop.over = true;
			return;
		}
		dragDrop.over = false;
		if (type !== 'drop') return;
		const mrpack = event.payload.paths.find((p) => p.toLowerCase().endsWith('.mrpack'));
		if (mrpack) openMrpackDrop(mrpack);
	});
}
