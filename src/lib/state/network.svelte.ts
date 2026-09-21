// El loop de lanzar/jugar una instancia ya instalada funciona sin red hoy
// (ensure_downloaded/ensure_java ya chequean archivos locales primero y
// solo pegan a internet si falta algo — se confirmó leyendo el código, no
// se rearmó nada ahí). Lo que faltaba era avisar cuando NO hay conexión,
// en vez de dejar que cada panel tire un error de red crudo y confuso.
export const network = $state<{ online: boolean }>({
	online: typeof navigator !== 'undefined' ? navigator.onLine : true
});

let initialized = false;

export function initNetworkListener() {
	if (initialized || typeof window === 'undefined') return;
	initialized = true;
	window.addEventListener('online', () => (network.online = true));
	window.addEventListener('offline', () => (network.online = false));
}
