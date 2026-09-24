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
	],
	'0.9.11': [
		'Arreglado: la cabecera de la instancia (nombre, botón Jugar) se iba achicando cada vez más a medida que instalabas mods — ahora se queda fija.'
	],
	'0.9.12': [
		'El instalador de Windows ahora es en español y con la marca de TFL Client, en vez del asistente genérico en inglés. El instalador de macOS también tiene su propia identidad visual ahora.'
	],
	'0.9.13': [
		'Linux: instalación con un solo comando (sin lidiar con apt/dnf a mano) — el link está en el README del repo.',
		'Nuevo: podés elegir entre modo Automático (recomendado, sin diálogos de archivo) o Manual (habilita subir tu ícono/fondo propio y agregar mods por archivo) — se pregunta la primera vez y queda siempre editable en Ajustes.',
		'Mods, ResourcePacks y Shaders: la pestaña ahora se divide en Gestionar (lo instalado, con selección múltiple) y Descargar (mods populares para instalar directo, filtros por categoría, marca de lo ya instalado y poder elegir versión).'
	],
	'0.9.14': [
		'Arreglado: algunos mods (ej. Create y sus addons) se instalaban sin las dependencias que necesitan, y el juego tiraba "Incompatible mods found!" al abrir la instancia — pasaba con Fabric, Forge, NeoForge y Quilt por igual. A los mods que ya tenías instalados así, sacalos y volvé a instalarlos para que bajen completos.'
	],
	'0.9.15': [
		'Arreglado: instancias con muchos mods podían no iniciar sin ningún mensaje de error — el launcher ahora sube la RAM sola según la cantidad de mods, y si el juego igual se cierra de golpe, la ventana de log te dice por qué en vez de quedarse muda.',
		'Arreglado: las cuentas de Microsoft podían perder la sesión al cerrar y volver a abrir el launcher.',
		'Arreglado: la configuración guardada (cuentas, RAM, ajustes) ya no se puede perder si el launcher se cierra de golpe justo mientras guarda.',
		'Arreglado: actualizar un modpack de TFL Selection podía dejar la instancia a medio instalar si se cortaba la conexión — ahora baja el nuevo primero y recién después saca el viejo.',
		'Varios arreglos chicos en la pestaña de Mods/ResourcePacks/Shaders (selección múltiple más confiable, el dropdown de versiones ya no mezcla mods distintos).'
	],
	'0.9.16': [
		'Arreglado: instalar un mod podía instalar de más, sin que lo pidieras — pasó de verdad con Sodium trayendo también Canvas Renderer (dos motores de renderizado incompatibles), dejando la instancia con "Incompatible mods found!" al abrir. Instancias que ya quedaron así hay que arreglarlas a mano (sacar uno de los dos desde Gestionar → Quitar); los mods que instales de acá en adelante ya no tienen este problema.',
		'Nuevo, como respaldo: el launcher avisa si intentás instalar a mano un mod que choca con otro ya instalado, antes de que llegues a abrir el juego.'
	],
	'0.9.17': [
		'Arreglado: en Gestionar, algunos mods aparecían con un nombre técnico raro en vez de su nombre real (el caso confuso reportado: Canvas Renderer se mostraba como "fabric-20.0.2625", parecía ser el propio Fabric Loader) — ahora siempre se muestra el nombre real del mod.'
	]
};
