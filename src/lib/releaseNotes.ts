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
	],
	'0.9.18': [
		'Arreglado: en Linux, instalar el .deb o .rpm no traía todas las dependencias que el launcher necesita para arrancar — el paquete solo pedía WebKitGTK y GTK, pero el programa también necesita otras librerías del sistema que no estaban declaradas, y sin ellas fallaba al abrir aunque el paquete se hubiera instalado bien. Si ya lo instalaste y no abre, borrá el paquete viejo y volvé a instalar esta versión (o corré install-linux.sh de nuevo).'
	],
	'0.9.19': [
		'Prolijo: el .deb dejó de listar dos dependencias repetidas (quedó igual de funcional, solo se veía feo con "apt show").'
	],
	'0.10.0': [
		'El logo del launcher se mudó a la barra de título, junto a minimizar/maximizar/cerrar.',
		'Los filtros de categoría en Descargar ahora son un desplegable "Categorías" en vez de ~19 chips siempre visibles.',
		'Gestionar ahora muestra el ícono de cada mod/shader/resourcepack, y sumó su propio filtro por categoría.',
		'Las pestañas de la instancia quedan fijas arriba al scrollear una lista larga en Descargar.',
		'Capturas de pantalla: botón para copiar la imagen al portapapeles, previsualización más chica por defecto con zoom a la rueda del mouse.',
		'Al crear una instancia, ahora elegís primero entre Personalizada, Modpack o Servidor.',
		'Nueva casilla para mostrar versiones experimentales (snapshots, pre-releases) al crear una instancia Vanilla.',
		'Nueva sección "Fuentes" en Ajustes > Estilos (Sistema, Nunito, Inter, Poppins, JetBrains Mono), se aplica al instante.',
		'Nuevo: selector de idioma (Español/English) en Ajustes > General — todo el launcher está traducido.',
		'MEGA: nueva categoría "Servidor" al crear una instancia — Vanilla, Paper o Purpur. Pestañas propias (Detalles, Plugins, Mundos, Archivos, Consola con comandos) y botón de Iniciar/Detener. Correr un servidor ya no bloquea jugar en el cliente al mismo tiempo, ni viceversa.'
	],
	'0.10.1': [
		'Arreglado de verdad: las pestañas de la instancia (Detalles/Mods/Shaders/...) seguían desapareciendo al scrollear en Descargar — el intento anterior no funcionaba en la práctica. Ahora sí quedan fijas.'
	],
	'0.10.2': [
		'Nuevo: en Detalles de una instancia de servidor, ahora se ve la dirección (IP:puerto) para conectarse, con botón de copiar — antes el servidor arrancaba pero no había forma de saber qué pasarle a otro jugador.'
	],
	'0.10.3': [
		'Arreglado: la dirección de conexión del servidor mostraba solo la IP local (solo servía en la misma red Wi-Fi). Ahora se muestra la IP pública y el launcher intenta abrir el puerto en el router solo — si el router lo permite, cualquiera se puede conectar con esa dirección sin configurar nada más.'
	],
	'0.10.4': [
		'Nuevo: cuando el router no deja abrir el puerto solo, ahora se puede vincular una cuenta gratis de playit.gg (un click, sin tarjeta) desde Detalles del servidor — con eso vinculado, la dirección de conexión anda siempre, para cualquiera, sin depender del router.'
	],
	'0.10.5': [
		'Nuevo: botón "Unirse a servidor" (barra lateral y Ctrl/Cmd+K) para conectarte pegando la dirección — elegís entre una instancia tuya ya creada o "Quick Join" (crea o reusa una Vanilla de la versión que quieras), y el juego arranca ya conectado, sin pasar por el menú de Multijugador.'
	],
	'0.10.6': [
		'Arreglado: unirse a un servidor real tiraba "invalid session" — el launcher no refrescaba la sesión de Microsoft antes de lanzar. Ahora se refresca siempre.',
		'Nuevo: panel de Skin y capa (ícono junto a Ajustes, cuentas Microsoft) — subir tu skin propia, elegir clásico/slim, restaurar la default, y elegir qué capa usar entre las que ya tenés.',
		'Nuevo: botón "Crear acceso directo" en cada instancia de juego — crea un ícono en el Escritorio que la abre directo, sin mostrar el launcher (queda en la bandeja del sistema mientras jugás).'
	],
	'0.10.7': [
		'Arreglado: en macOS, el panel de Skin y capa no mostraba ninguna imagen (ícono roto) — las texturas de Mojang no cargaban por una regla de seguridad interna mal configurada. Ya se ven bien en las 3 plataformas.'
	],
	'0.10.8': [
		'Nuevo: el panel de Skin y capa ahora muestra el personaje completo en 3D (arrastrá para girar, rueda para zoom) en vez del recorte de la cara — se ve la skin y la capa juntas.',
		'Nuevo: click derecho en una instancia de la barra lateral muestra accesos rápidos (Abrir, Mods, Crear acceso directo, Exportar, Carpeta) en vez del menú del navegador.'
	],
	'0.10.9': [
		'Arreglado: "Unirse a servidor" abría Minecraft normal en vez de conectar directo al servidor. Ya conecta bien en versiones 1.20.2 en adelante — versiones más viejas no soportan esta función de Mojang, así que ahora avisa antes en vez de fallar en silencio.'
	],
	'0.10.10': [
		'Nuevo: ya se puede abrir una instancia mientras otra está corriendo — antes lo bloqueaba directo. Avisa primero si eso puede traer problemas (misma cuenta en las dos, o rendimiento), con opción de "no volver a preguntarme" en Ajustes.',
		'Ajustes rediseñado: panel más ancho, "General" se dividió en Rendimiento y Sistema para que no quede todo apiñado, y los textos largos de explicación ahora son un ícono "?" con globo flotante al pasar el mouse.',
		'Arreglado: la lista de instancias en la barra lateral mostraba solo la inicial en vez del ícono real de cada una.',
		'Arreglado: la barrita de color de la instancia activa se salía un poco de las esquinas redondeadas de la card.'
	],
	'0.10.11': [
		'Arreglado de verdad: al pasar el mouse por una instancia en la barra lateral, parte del ícono se veía negro — el intento anterior (v0.10.10) no alcanzaba. Era el efecto de deslizamiento del hover, se sacó.'
	],
	'0.10.12': [
		'Arreglado: el click derecho en una instancia de la barra lateral no abría el menú de accesos rápidos (Abrir, Mods, Crear acceso directo, Exportar, Carpeta) — no pasaba nada.'
	],
	'0.10.13': [
		'Arreglado: click derecho en cualquier otra parte del launcher (fuera de una instancia) mostraba el menú del navegador (Atrás, Refrescar, Guardar como, Imprimir) — se veía como página web. Ya no aparece.'
	],
	'0.10.14': [
		'Arreglado: al instalar un mod, algunos aparecían duplicados en la lista (se bajaba también el archivo de código fuente "-sources" junto al mod real). Ya se instala solo el mod. Si ya tenés duplicados de antes, quitalos desde Gestionar.',
		'Arreglado: si borrabas un mod a mano de la carpeta, después no se podía sacar de la lista del launcher (error "no se encuentra el archivo"). Ahora se saca sin problema.'
	],
	'0.10.15': [
		'Nuevo: Importar instancias de Prism, MultiMC o CurseForge con sus mods, mundos y configuración (al crear una instancia → "Importar de otro launcher").',
		'Nuevo: servidores favoritos en "Unirse a servidor", con estado en vivo (si está arriba, jugadores, MOTD y latencia) y botón para entrar directo.',
		'Nuevo: copias de seguridad de mundos en Detalles — creá una a mano, restaurala o borrala (se guardan las últimas 10).',
		'Nuevo: podés desactivar un mod o plugin sin borrarlo, y "Actualizar todos" ahora se puede deshacer con un click.',
		'Nuevo: si el juego se cierra por un error, el launcher señala qué mods lo pueden haber causado y deja desactivarlos al toque.',
		'Nuevo: el tiempo total jugado de cada instancia aparece en Detalles.',
		'Mejorado: instalar otra versión de un mod ahora reemplaza la anterior en vez de dejar las dos, y hay un botón "Limpiar duplicados" para los que ya tenías.'
	],
	'0.10.16': [
		'Arreglado: la versión 0.10.15 no llegó a publicarse completa para macOS y Linux por un error de empaquetado. Esta trae todo lo de 0.10.15 (importar instancias, favoritos, backups de mundos, desactivar mods y más) para las 3 plataformas.'
	],
	'0.11.0-beta.1': [
		'Esta es una versión BETA: trae las novedades de la 0.11.0 antes que el resto y puede tener errores. Si algo anda mal, avisanos; podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Actualizaciones más claras: un aviso discreto cuando hay una versión nueva, se descarga sola en segundo plano con barra de progreso, y al terminar te deja "Reiniciar para actualizar" cuando quieras. Si algo falla, hay un botón para reintentar.',
		'Más seguro: el launcher ahora corre con permisos mínimos (la ventana de logs solo puede hacer lo suyo y una protección más estricta contra contenido externo). No cambia nada de cómo se usa.',
		'Arreglado: el botón "Deshacer última actualización" de los mods no aparecía después de actualizar.',
		'Arreglado: copiar una captura de pantalla al portapapeles fallaba.',
		'Por dentro: mucho orden en el código del launcher (más fácil y rápido de mejorar de ahora en más) y medición interna de rendimiento para detectar si alguna versión lo empeora.'
	],
	'0.11.0-beta.2': [
		'Arreglado: la skin y la capa no aparecían en el gestor de skins.',
		'Arreglado: el ícono de una instancia no se actualizaba en la lista lateral hasta reiniciar el launcher. Ahora cambia al instante.'
	],
	'0.11.0': [
		'Versiones beta: en Ajustes → Sistema → Actualizaciones podés activar las betas para recibir las novedades antes. Te avisa que pueden tener errores y lo podés desactivar cuando quieras.',
		'Actualizaciones más claras: un aviso discreto cuando hay una versión nueva, descarga en segundo plano con barra de progreso y "Reiniciar para actualizar" cuando quieras. Si algo falla, hay un botón para reintentar.',
		'Más seguro: el launcher corre con permisos mínimos y una protección más estricta contra contenido externo. No cambia nada de cómo se usa.',
		'Arreglado: la skin y la capa no aparecían en el gestor de skins.',
		'Arreglado: el ícono de una instancia no se actualizaba en la lista lateral hasta reiniciar.',
		'Arreglado: el botón "Deshacer última actualización" de los mods no aparecía, y copiar una captura al portapapeles fallaba.',
		'Por dentro: mucho orden en el código del launcher y medición interna de rendimiento para detectar si alguna versión lo empeora.'
	],
	'0.11.1-beta.1': [
		'Esta es una versión BETA. Si algo anda mal, avisanos; podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Nueva home: tu personaje en grande (skin real si tenés cuenta Microsoft) y la última instancia que jugaste, lista para abrir con un click.',
		'Sidebar colapsable: un botón lo deja en modo compacto (solo íconos) para ganar espacio — se acuerda entre reinicios.',
		'Nuevo botón "Home" en el sidebar para volver a la pantalla principal en cualquier momento (antes no se podía).'
	],
	'0.11.1': [
		'Nueva home: tu personaje en grande (skin real si tenés cuenta Microsoft) y la última instancia que jugaste, lista para abrir con un click.',
		'Sidebar colapsable: un botón lo deja en modo compacto (solo íconos) para ganar espacio — se acuerda entre reinicios.',
		'Nuevo botón "Home" en el sidebar para volver a la pantalla principal en cualquier momento (antes no se podía).'
	],
	'0.11.2-beta.1': [
		'Esta es una versión BETA. Si algo anda mal, avisanos; podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Home mucho más completa: banner con la última captura de tu instancia, mini panel de skins, tus servidores favoritos, actividad reciente (las últimas sesiones jugadas) y estado del sistema (Java, RAM, disco).',
		'"Otras instancias" ahora es un carrusel con flechas en vez de una fila fija.'
	]
};
