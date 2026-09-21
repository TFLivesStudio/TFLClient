// Notas de "qué hay de nuevo" para el panel in-app (WhatsNewTips.svelte) —
// texto para JUGADORES, no el changelog técnico de código que se genera
// para cada tanda de cambios. La mayoría de los usuarios nunca abre el
// Release de GitHub, así que el contenido real tiene que estar acá adentro,
// no solo un link de salida. Agregar una entrada nueva cada vez que se
// corta una versión — 2 a 5 líneas cortas, en criollo, sin jerga de código.
export const RELEASE_NOTES: Record<string, string[]> = {
	'0.9.1': [
		'Pantalla de bienvenida y "qué hay de nuevo" cada vez que el launcher se actualiza solo.',
		'Fondo de pantalla "Aurora animada" nuevo en Ajustes → Estilo.',
		'Perfiles gráficos rápidos (Bajo/Medio/Alto) en la pestaña de Shaders — un click instala un shader curado.'
	],
	'0.9.2': [
		'Escape (Esc) ahora cierra cualquier ventana del launcher al toque, sin tocar el mouse.',
		'Arreglado: elegir la superficie OLED y el modo claro al mismo tiempo dejaba el launcher en un estado raro.',
		'Ahora podés subir tu propia imagen como fondo de pantalla, en Ajustes → Estilo.',
		'El perfil de calidad "Experiencia" ahora se ve realmente distinto a "Balanceado".'
	],
	'0.9.3': [
		'Nueva pestaña de Resource Packs: buscá e instalá packs de texturas de Modrinth sin salir del launcher.',
		'Nuevo gestor de capturas de pantalla: mirá, ampliá y borrá los screenshots de cada instancia.'
	],
	'0.9.6': [
		'Arreglado: en macOS, subir tu propio ícono de instancia o skin crasheaba el launcher entero al elegir el archivo.',
		'Arreglado: las instancias en las versiones de Minecraft más nuevas (26.x) fallaban al abrir por instalarse una versión de Java vieja — ahora se instala la correcta sola.'
	]
};
