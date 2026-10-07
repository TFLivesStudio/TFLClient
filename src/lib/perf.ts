import { invoke } from '@tauri-apps/api/core';

// Mediciones del lado de la interfaz. No hay telemetría: se escriben en la
// consola del WebView y se mandan al backend (`perf_record`) solo para que el
// reporte único (`window.__tflPerf()`) junte lo de los dos lados.
//
// Activas en desarrollo (`tauri dev`) o si en la consola del WebView se hace
// `localStorage.setItem('tfl-perf', '1')` (y el launcher se abrió con la
// variable de entorno TFL_PERF=1, que es la que activa el lado Rust). Apagadas,
// `perfRecord` no hace nada.

export const perfEnabled: boolean = (() => {
	if (import.meta.env.DEV) return true;
	try {
		return localStorage.getItem('tfl-perf') === '1';
	} catch {
		return false;
	}
})();

/** Registra una medición de la interfaz (se guarda como `ui.<name>`). */
export function perfRecord(name: string, ms: number): void {
	if (!perfEnabled || !Number.isFinite(ms) || ms < 0) return;
	console.info(`[perf] ui.${name}: ${ms.toFixed(0)} ms`);
	invoke('perf_record', { name: `ui.${name}`, ms }).catch(() => {});
}

/** Mide una operación asíncrona y la registra. */
export async function perfMeasure<T>(name: string, run: () => Promise<T>): Promise<T> {
	if (!perfEnabled) return run();
	const started = performance.now();
	try {
		return await run();
	} finally {
		perfRecord(name, performance.now() - started);
	}
}

/**
 * "Tiempo hasta UI usable": desde que el WebView empezó a cargar la página
 * hasta que se pintó el primer cuadro con el launcher ya listo (pantalla de
 * carga sacada, instancias cargadas). Llamar cuando el estado inicial ya está.
 */
export function perfMarkUsable(): void {
	if (!perfEnabled) return;
	requestAnimationFrame(() => requestAnimationFrame(() => perfRecord('usable', performance.now())));
}

/** `await window.__tflPerf()` en la consola del WebView imprime el reporte completo. */
export function exposePerfReport(): void {
	if (!perfEnabled || typeof window === 'undefined') return;
	(window as unknown as { __tflPerf?: () => Promise<unknown> }).__tflPerf = async () => {
		const report = await invoke<{
			summary: Record<string, unknown>;
			idle: unknown;
			enabled: boolean;
		}>('get_perf_report');
		console.table(report.summary);
		console.info('[perf] reposo del launcher:', report.idle);
		return report;
	};
}
