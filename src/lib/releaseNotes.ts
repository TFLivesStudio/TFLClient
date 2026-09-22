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
	],
	'0.9.7': [
		'En macOS: subir tu propia imagen (ícono de instancia o fondo de pantalla) queda deshabilitado por ahora — seguía crasheando el launcher pese al arreglo anterior. El resto del launcher no se ve afectado, y en Windows/Linux sigue andando normal.'
	],
	'0.9.8': [
		'TFL Selection: los packs curados ahora pueden crear su propia instancia (con la versión y el loader correctos) con un solo click, sin tener que armarla vos antes.'
	],
	'0.9.9': [
		'TFL Selection: los packs curados ahora se organizan en pestañas por categoría (PvP, Chill, Técnico, etc.) en vez de una sola lista larga.'
	],
	'0.9.10': [
		'El launcher ahora busca y baja actualizaciones solo al abrir, en segundo plano — cuando está lista aparece un aviso abajo a la izquierda, un click instala y reinicia. Podés apagar esto en Ajustes → Actualizaciones.',
		'Arreglado: en Ajustes → Estilo, el color de acento activo y algunas filas de opciones se veían cortadas/amontonadas.'
	]
};
