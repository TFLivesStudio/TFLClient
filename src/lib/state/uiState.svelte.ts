import { appState } from '$lib/state/state.svelte';
import type { InstanceData } from '$lib/types/types';

// Estado de UI global del shell de la app: flags de arranque/onboarding,
// menú contextual y modales. Vive en un módulo (objetos `$state` mutables,
// igual que `appState`) para que cualquier componente pueda abrir un modal
// sin pasar callbacks por varios niveles.

export interface ContextMenuState {
	x: number;
	y: number;
	instance: InstanceData;
}

export const ui = $state<{
	/** true hasta que termina la carga inicial (cuenta, ajustes, instancias). */
	loading: boolean;
	/** Se acaba de completar el onboarding de cuenta en esta sesión. */
	justOnboarded: boolean;
	/** El banner de modo de diálogos se cerró con la X (solo por esta sesión). */
	dialogPromptDismissed: boolean;
	/** Menú contextual abierto sobre una instancia de la barra lateral. */
	contextMenu: ContextMenuState | null;
}>({
	loading: true,
	justOnboarded: false,
	dialogPromptDismissed: false,
	contextMenu: null
});

export const modals = $state({
	createChooser: false,
	create: false,
	createServer: false,
	importInstance: false,
	serverConnectionInfo: false,
	joinServer: false,
	skinManager: false,
	settings: false,
	tflSelection: false
});

export type ModalName = keyof typeof modals;

export function openModal(name: ModalName) {
	modals[name] = true;
}

export function closeModal(name: ModalName) {
	modals[name] = false;
}

export function openInstanceContextMenu(instance: InstanceData, x: number, y: number) {
	ui.contextMenu = { x, y, instance };
}

export function closeInstanceContextMenu() {
	ui.contextMenu = null;
}

/** La paleta de comandos no se abre encima de estos modales. Ojo: el modal
 * de importar NO está en la lista (así era antes de extraer esto). */
export function isCommandPaletteBlocked(): boolean {
	return (
		modals.createChooser ||
		modals.create ||
		modals.createServer ||
		modals.joinServer ||
		modals.skinManager ||
		modals.serverConnectionInfo ||
		modals.settings ||
		modals.tflSelection
	);
}

// Derivados del flujo de arranque/onboarding. Se exponen como getters (no se
// puede exportar un `$derived` desde un módulo); al leerse dentro de un
// template o un efecto siguen siendo reactivos.
export const flow = {
	get needsOnboarding(): boolean {
		return appState.settings?.onboarded !== true;
	},
	// Instalación nueva: se pregunta el modo automático/manual como modal
	// bloqueante justo después del onboarding de cuenta (ver `justOnboarded`).
	// Usuario existente que actualiza a una versión con este campo (nunca
	// pasó por `justOnboarded` en esta sesión): mismo aviso pero como banner
	// no bloqueante — ver NativeDialogModePrompt.svelte. Ambos casos dejan
	// de mostrarse apenas se elige un modo (marca `native_dialog_mode_prompted`).
	// Cerrar el banner con la X sin elegir nada (`dialogPromptDismissed`) NO
	// persiste ningún modo — solo lo oculta por esta sesión; la próxima vez
	// que se abra el launcher se vuelve a preguntar, en vez de fijar
	// "Automático" en silencio como pasaba antes.
	get needsDialogModePrompt(): boolean {
		return (
			appState.settings?.onboarded === true &&
			appState.settings?.native_dialog_mode_prompted !== true &&
			!ui.dialogPromptDismissed
		);
	}
};
