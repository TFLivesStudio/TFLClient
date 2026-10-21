import { i18nState } from './index.svelte';
const words = {
	general: ['General', 'General'],
	generalHint: [
		'Idioma, archivos locales y almacenamiento del launcher.',
		'Language, local files and launcher storage.'
	],
	appearance: ['Apariencia', 'Appearance'],
	appearanceHint: [
		'Estilo, colores, fondos y efectos del launcher. No cambia el rendimiento de Minecraft.',
		'Launcher style, colors, backgrounds and effects. Does not change Minecraft performance.'
	],
	game: ['Minecraft y Java', 'Minecraft & Java'],
	gameHint: [
		'Memoria para el juego y versiones de Java. Cada instancia puede tener sus propios valores en Detalles.',
		'Game memory and Java versions. Individual instances can override these values in Details.'
	],
	accounts: ['Cuentas', 'Accounts'],
	accountsHint: [
		'Sesiones de Microsoft, cuentas sin conexión y mascotas.',
		'Microsoft sessions, offline accounts and mascots.'
	],
	console: ['Consola', 'Console'],
	consoleHint: [
		'El registro y sus gráficas, tanto para Minecraft como para servidores. Los cambios se aplican al instante.',
		'Logs and charts for Minecraft and servers. Changes apply immediately.'
	],
	updates: ['Actualizaciones', 'Updates'],
	updatesHint: [
		'Versión instalada, búsqueda de actualizaciones y canal beta.',
		'Installed version, update checks and beta channel.'
	],
	searchSettings: ['Buscar una opción…', 'Find a setting…'],
	noSettings: [
		'No hay coincidencias. Prueba con RAM, idioma, skin o gráficas.',
		'No matches. Try RAM, language, skin or charts.'
	],
	charts: ['Gráficas históricas', 'History charts'],
	chartHint: [
		'Muestras reales cada segundo mientras el proceso está activo y la consola visible. Sin datos se muestra un hueco. El historial se conserva al detenerse, durante esta sesión de consola.',
		'Real samples every second while the process is running and the console is visible. Missing data leaves a gap. History remains after stopping, during this console session.'
	],
	cpu: ['CPU', 'CPU'],
	ram: ['RAM', 'RAM'],
	disk: ['Disco', 'Disk'],
	top: ['Encima del registro', 'Above the log'],
	bottom: ['Debajo del registro', 'Below the log'],
	side: ['Al lado del registro', 'Beside the log'],
	position: ['Ubicación de las gráficas', 'Chart position'],
	columns: ['Columnas de gráficas', 'Chart columns'],
	window: ['Ventana de tiempo', 'Time window'],
	cpuMode: ['Unidad de CPU', 'CPU unit'],
	cores: ['Por núcleo · 100 % = un núcleo', 'Per core · 100% = one core'],
	total: ['Porcentaje del equipo completo', 'Percentage of the whole computer'],
	chartStyle: ['Estilo de gráfica', 'Chart style'],
	styleLines: ['Líneas (historial)', 'Lines (history)'],
	styleGauge: ['Circular (valor actual)', 'Circular (current value)'],
	dragHint: [
		'Arrastra las tarjetas para cambiar su orden, o usa las flechas. El registro mantiene su espacio.',
		'Drag cards to reorder, or use the arrows. The log keeps its own space.'
	],
	up: ['Mover antes', 'Move earlier'],
	down: ['Mover después', 'Move later'],
	show: ['Mostrar', 'Show'],
	hide: ['Ocultar', 'Hide'],
	log: ['Registro', 'Log'],
	colors: ['Colores del registro', 'Log colors'],
	info: ['Información', 'Information'],
	warn: ['Advertencias', 'Warnings'],
	error: ['Errores', 'Errors'],
	debug: ['Depuración', 'Debug'],
	fontSize: ['Tamaño del texto', 'Text size'],
	wrap: ['Ajustar líneas al ancho', 'Wrap long lines'],
	follow: ['Seguir las nuevas líneas', 'Follow new lines'],
	reset: ['Restaurar consola', 'Reset console'],
	saveError: [
		'No se pudieron guardar las preferencias. Siguen activas en esta ventana.',
		'Preferences could not be saved. They remain active in this window.'
	],
	unavailable: ['Sin muestras del proceso', 'No process samples'],
	waiting: ['Esperando la segunda muestra…', 'Waiting for the second sample…'],
	settings: ['Personalizar consola', 'Customize console'],
	close: ['Cerrar', 'Close'],
	filter: ['Buscar en el registro…', 'Search the log…'],
	all: ['Todos los niveles', 'All levels'],
	copy: ['Copiar registro visible', 'Copy visible log'],
	copied: ['Copiado', 'Copied'],
	copyFailed: ['No se pudo copiar', 'Copy failed'],
	preview: ['Vista previa', 'Preview'],
	seconds: ['segundos', 'seconds'],
	scale: ['Escala', 'Scale'],
	logHint: [
		'Los niveles se detectan en el texto de Java; stderr sin nivel se marca como error. El texto original se conserva.',
		'Levels are detected from Java output; stderr without a level is marked as error. Original text is preserved.'
	]
} as const;
export type WorkbenchKey = keyof typeof words;
export const w = (key: WorkbenchKey) => words[key][i18nState.locale === 'en' ? 1 : 0];
