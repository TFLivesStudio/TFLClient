// Notas de "qué hay de nuevo" para el panel in-app (WhatsNewTips.svelte) —
// texto para JUGADORES, no el changelog técnico de código que se genera
// para cada tanda de cambios. La mayoría de los usuarios nunca abre el
// Release de GitHub, así que el contenido real tiene que estar acá adentro,
// no solo un link de salida. Agregar una entrada nueva cada vez que se
// corta una versión — 2 a 5 líneas cortas, en criollo, sin jerga de código.
export const RELEASE_NOTES: Record<string, string[]> = {
	'0.15.1-beta.1': [
		'Esta es una versión BETA de prueba, todavía en revisión — avisanos si encontrás algo raro. Podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Arrastrá un archivo .mrpack y soltalo sobre la ventana para importarlo directo, sin pasar por el selector de archivos.'
	],
	'0.15.0': [
		'Importá un archivo .mrpack suelto (de Modrinth, Prism o CurseForge) desde Biblioteca → Importar instancia: configura sola la versión de Minecraft, el loader y los mods.',
		'La detección automática de instancias para importar ahora también encuentra las de la app de Modrinth, además de Prism, MultiMC, PolyMC y CurseForge.'
	],
	'0.15.0-beta.2': [
		'Arreglado: la detección automática de instancias de la app de Modrinth no encontraba ninguna — ahora sí aparecen todas al importar.'
	],
	'0.15.0-beta.1': [
		'Esta es una versión BETA de prueba, todavía en revisión — avisanos si encontrás algo raro. Podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Importá un archivo .mrpack suelto (de Modrinth, Prism o CurseForge) desde Biblioteca → Importar instancia: configura sola la versión de Minecraft, el loader y los mods.',
		'La detección automática de instancias para importar ahora también encuentra las de la app de Modrinth, además de Prism, MultiMC, PolyMC y CurseForge.'
	],
	'0.14.0-beta.2': [
		'Conecta asistentes compatibles mediante MCP local: aprueba cada sesión, limita sus permisos y confirma las acciones críticas desde Integraciones.',
		'Cinco paisajes cinematográficos nuevos, miniaturas fieles y vídeos a 720p, manteniendo Lite estático y los fondos personales.',
		'Los cambios de escenas, apariencia y consola se sincronizan con el launcher; las instalaciones de contenido comparten bloqueos y validación de descargas.'
	],
	'0.14.0-beta.1': [
		'Ajustes se organiza por tareas y su buscador te lleva directamente a cada opción, con ayuda y validación de RAM, Java y variables de entorno.',
		'Fondos y escenas: siete paisajes originales, vídeos locales con vista previa y una imagen estática en Lite. Puedes asignar una escena a cada instancia.',
		'Diseña tu consola arrastrando sus tarjetas reales: cinco moldes, gráficas por métrica, colores y Aplicar/Cancelar/Deshacer, sin tocar el muestreo de CPU y RAM.',
		'36 iconos vectoriales nuevos, campos de búsqueda más limpios en Pixel y doble clic para instalar mods, shaders, resource packs y plugins.'
	],
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
	],
	'0.12.0-beta.1': [
		'Esta es una versión BETA con el rediseño visual más grande hasta ahora — justo por eso es la primera en salir como beta: si algo se ve raro en tu pantalla, contanos qué y dónde. Podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Home rehecha de nuevo: ahora tu personaje y el botón JUGAR comparten un mismo banner grande en vez de estar separados en cards sueltas — se ve de un vistazo qué instancia vas a abrir y con quién.',
		'La sidebar se adapta sola al tamaño de la ventana: en una ventana chica se compacta a solo íconos para no comerte espacio, sin pisar si vos ya la habías dejado compacta a mano.',
		'Los perfiles de calidad (Ajustes → Rendimiento) ahora se notan de verdad: Lite saca todo movimiento decorativo y deja el personaje quieto; Experience suma animaciones de personaje, transiciones y más efectos — antes Balanced y Experience se veían casi idénticos.',
		'Nuevo: podés pedirle al launcher que reduzca animaciones sin tener que bajar todo tu perfil a Lite (Ajustes → Rendimiento → "Reducir animaciones").',
		'Por dentro: toda la interfaz pasa a usar un mismo sistema de diseño (tamaños, colores, espaciados) en vez de que cada pantalla tuviera el suyo — más fácil de mantener parejo de acá en adelante.'
	],
	'0.12.0': [
		'Rediseño visual más grande hasta ahora: la beta se validó sin problemas y ya es la versión normal.',
		'Home rehecha de nuevo: ahora tu personaje y el botón JUGAR comparten un mismo banner grande en vez de estar separados en cards sueltas — se ve de un vistazo qué instancia vas a abrir y con quién.',
		'La sidebar se adapta sola al tamaño de la ventana: en una ventana chica se compacta a solo íconos para no comerte espacio, sin pisar si vos ya la habías dejado compacta a mano.',
		'Los perfiles de calidad (Ajustes → Rendimiento) ahora se notan de verdad: Lite saca todo movimiento decorativo y deja el personaje quieto; Experience suma animaciones de personaje, transiciones y más efectos.',
		'Nuevo: podés pedirle al launcher que reduzca animaciones sin tener que bajar todo tu perfil a Lite (Ajustes → Rendimiento → "Reducir animaciones").'
	],
	'0.12.1-beta.1': [
		'Esta es una versión BETA. Si algo anda mal, avisanos; podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'La ventana de log de una instancia corriendo ahora muestra CPU, RAM y disco como gráficos en tiempo real (en vez de un número suelto) en una barra nueva abajo. El tercer gráfico es disco de lectura/escritura — no hay forma confiable de medir GPU o internet por proceso en las 3 plataformas, así que fue la opción técnica, no la de gusto.',
		'Los menús (clic derecho sobre una instancia) ahora abren y cierran con una animación suave y se pueden recorrer con las flechas del teclado — antes aparecían de golpe y solo se cerraban con Escape.'
	],
	'0.12.1-beta.2': [
		'Arreglado: el gráfico de CPU en la ventana de log podía mostrar más de 100% (250%, 143%, etc.) en procesadores de varios núcleos. Ahora siempre queda entre 0 y 100%.'
	],
	'0.12.1': [
		'La ventana de log de una instancia corriendo ahora muestra CPU, RAM y disco como gráficos en tiempo real (en vez de un número suelto) en una barra nueva abajo. El tercer gráfico es disco de lectura/escritura — no hay forma confiable de medir GPU o internet por proceso en las 3 plataformas, así que fue la opción técnica, no la de gusto.',
		'Los menús (clic derecho sobre una instancia) ahora abren y cierran con una animación suave y se pueden recorrer con las flechas del teclado — antes aparecían de golpe y solo se cerraban con Escape.',
		'Arreglado: el gráfico de CPU podía mostrar más de 100% (250%, 143%, etc.) en procesadores de varios núcleos. Ahora siempre queda entre 0 y 100%.',
		'Arreglo de seguridad por dentro: se actualizó la librería de compresión ZIP que usa el launcher por vulnerabilidades reportadas (una crítica) — no afecta el uso normal, solo hacía falta la actualización.'
	],
	'0.12.2-beta.1': [
		'Esta es una versión BETA. Si algo anda mal, avisanos; podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Arreglado: en Mac con chip Apple Silicon (M1/M2/M3/M4/M5), instancias vanilla viejas (anteriores a la 1.19) cerraban solas apenas abrían el juego. El launcher pedía Java para ese chip, pero esas versiones de Minecraft no tienen esa variante lista — ahora el launcher detecta el caso y usa la versión de Java correcta.'
	],
	'0.12.2': [
		'Arreglado: en Mac con chip Apple Silicon (M1/M2/M3/M4/M5), instancias vanilla viejas (anteriores a la 1.19) cerraban solas apenas abrían el juego. El launcher pedía Java para ese chip, pero esas versiones de Minecraft no tienen esa variante lista — ahora el launcher detecta el caso y usa la versión de Java correcta.'
	],
	'0.12.3': [
		'Arreglado: el código para iniciar sesión con Microsoft no se podía copiar ni pegar con el click derecho. Ahora funciona normal (seguís teniendo también el botón de copiar al lado del código).'
	],
	'0.13.0-beta.1': [
		'Esta es una versión BETA de prueba, todavía en revisión — avisanos si encontrás algo raro. Podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Rediseño grande de Mods/Shaders/Resource Packs/Plugins/Modpacks → Descargar: ahora es un explorador real del catálogo de Modrinth, no un buscador de 10 resultados. Paginación de verdad (podés avanzar página por página), selector de orden (más descargados/seguidos/recientes/actualizados), y ahora también podés explorar sin escribir nada en la búsqueda.',
		'Nuevo botón "Explorar en vista amplia" que abre una pantalla grande dentro del launcher (sin ventana nueva) para explorar con más espacio, con filtros y categorías más cómodos.',
		'Nuevo: ficha completa de cada mod/shader/pack — descripción completa, categorías, versiones, galería de imágenes y links oficiales, sin salir del launcher para instalar.',
		'Modpacks ahora también deja explorar sin escribir texto, y avisa antes de instalar uno si la instancia ya tiene otro modpack instalado (puede pisar configuración compartida).'
	],
	'0.12.4-beta.1': [
		'BETA — Visual Refresh 2. Esta versión de prueba puede contener errores; contanos qué falla y en qué pantalla. Se instala desde los descargables de este release.',
		'Nuevo: estilos Modern y Pixel, materiales y escenas visuales desde Ajustes, con vista previa y opciones para personalizar tu Home.',
		'Nueva Biblioteca con lista o cuadrícula, búsqueda, favoritos y colecciones que no cambian las carpetas de tus instancias.',
		'Home renovada: la última instancia jugada, su artwork y el botón Jugar como protagonistas; podés ocultar los paneles secundarios.',
		'Detalle de instancia con navegación técnica lateral, dashboard de servidor y un espacio Explorar conectado a TFL Selection y al contenido de tus instancias.',
		'Estudio de personaje con cuentas, skins y capas. En Lite la skin es estática por defecto y podés activar 3D manualmente; el visor se detiene cuando queda oculto.',
		'Nuevo historial de notificaciones de la sesión y mejoras de navegación por teclado y foco en los diálogos.',
		'Actualizadas las dependencias TLS y de certificados para corregir los avisos de seguridad recientes.',
		'Esta beta visual es independiente de Content Discovery 0.13.0-beta.1. Como el canal beta elige la versión más alta, usá la descarga directa para probar esta edición.'
	],
	'0.13.1-beta.1': [
		'BETA 2 — Visual Refresh y Content Discovery ahora conviven: el catálogo paginado, las fichas completas y la vista amplia de la beta 0.13.0 con los estilos Modern y Pixel.',
		'Ajustes reorganizados por tarea, con búsqueda: General, Apariencia, Minecraft y Java, Cuentas, Consola y Actualizaciones.',
		'Consola de juego y servidor con gráficas históricas reales de CPU, RAM y disco. 100 % de CPU equivale a un núcleo; los procesos multihilo pueden superar ese valor.',
		'Personalizá la consola desde Ajustes o su propio botón: colores, texto, gráficas visibles, orden arrastrable, ubicación y ventana de tiempo.',
		'Pixel suma botones con relieve e inventarios más minecrafti. Sidebar compacta uniforme, controles al pie y más espacio para el nombre y la cara de tu skin.'
	],
	'0.13.1-beta.4': [
		'Esta es una versión BETA de prueba, todavía en revisión — avisanos si encontrás algo raro. Podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Arreglado: en macOS no se podía elegir un ícono/wallpaper personalizado ni agregar mods/shaders/packs desde archivo — el diálogo nativo se quedaba colgado y nunca abría.',
		'Arreglado: la pestaña activa en el menú lateral del detalle de instancia (Mods, Shaders, Mundos y backups, etc.) se veía con el borde recortado contra el panel.',
		'Nuevo: elegí entre gráfica de líneas o circular para CPU/RAM/disco en la consola (configuración de la consola).'
	],
	'0.13.1': [
		'Nuevo explorador de contenido en Mods/Shaders/Resource Packs/Plugins/Modpacks → Descargar: catálogo real de Modrinth con paginación de verdad, orden (descargados/seguidos/recientes/actualizados), exploración sin escribir texto, ficha completa de cada proyecto (descripción, categorías, versiones, galería, links oficiales) y una vista amplia a pantalla completa sin abrir ventanas nuevas.',
		'Rediseño visual completo: estilos Modern y Pixel con materiales y fondos personalizables, nueva Biblioteca con lista/cuadrícula y favoritos, Home renovada con tu última instancia como protagonista, y detalle de instancia con navegación técnica lateral.',
		'Ajustes reorganizados por tarea y con búsqueda: General, Apariencia, Minecraft y Java, Cuentas, Consola y Actualizaciones.',
		'Consola de juego y servidor con gráficas históricas reales de CPU, RAM y disco (elegí entre líneas o circular), niveles de log diferenciados, colores y orden personalizables, sincronizadas entre ventanas.',
		'Estudio de personaje con cuentas, skins y capas; en Lite la skin es estática por defecto con opción de activar 3D a mano.',
		'Arreglado: en macOS no se podía elegir un ícono/wallpaper personalizado ni agregar mods/shaders/packs desde archivo — el diálogo nativo se quedaba colgado.',
		'Arreglado: la paginación del explorador de contenido podía quedarse trabada en la página 1.'
	],
	'0.13.2': [
		'Arreglado: el botón Detener de la ventana de log no hacía nada mientras el juego estaba realmente corriendo — quedaba trabado esperando a sí mismo. Ahora corta el proceso correctamente.',
		'Arreglado: buscar en Ajustes podía dejar visible el contenido de una pestaña que ya no aparecía en los resultados.',
		'Arreglado: la marca decorativa de la tarjeta "TFL Selection" en Explorar mostraba un ícono placeholder en vez del logo real de TFL.'
	],
	'0.14.0-beta.3': [
		'Esta es una versión BETA de prueba, todavía en revisión — avisanos si encontrás algo raro. Podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Arte nuevo para las 7 escenas cinemáticas de Ajustes → Apariencia → Fondos y escenas, con 2 escenas nuevas (Cherry Night y Emerald Grotto).',
		'36 fondos de pantalla nuevos en Ajustes → Apariencia → Fondo de pantalla.'
	],
	'0.14.0-beta.6': [
		'Esta es una versión BETA de prueba, todavía en revisión — avisanos si encontrás algo raro. Podés desactivar las betas en Ajustes → Sistema → Actualizaciones.',
		'Rediseño: el fondo de pantalla que elegís en Ajustes ahora es siempre el fondo de todo el launcher. Las escenas (vídeo/atmósfera) pasan al cuadro grande de Home y a las tarjetas de la Biblioteca en vista cuadrícula.',
		'Quitamos 17 de los fondos de pantalla agregados en la beta anterior — estaban en baja resolución y se veían borrosos a pantalla completa. Los demás quedan.'
	],
	'0.14.1': [
		'Ajustes rediseñado: se organiza por tareas (General, Apariencia, Rendimiento, Sistema...) con buscador que te lleva directo a cada opción, más ayuda y validación de RAM, Java y variables de entorno.',
		'Fondos y escenas: 19 fondos de pantalla nuevos (en alta resolución) y 7 paisajes cinemáticos animados. El fondo que elegís es siempre el fondo de todo el launcher; las escenas en vídeo aparecen en el cuadro grande de Home y en las tarjetas de la Biblioteca en vista cuadrícula. Podés asignar una escena distinta a cada instancia.',
		'Diseña tu consola de rendimiento arrastrando sus tarjetas: varios moldes, gráficas por métrica y colores a elección.',
		'Conectá asistentes compatibles por MCP local desde Integraciones: aprobás cada sesión y limitás sus permisos a mano.',
		'36 íconos de instancia nuevos, y la lista de instancias en la barra lateral ahora muestra el ícono real en vez de solo las iniciales.',
		'Arreglado: el botón Detener de la ventana de log no respondía mientras el juego corría; búsquedas en Ajustes podían dejar visible una pestaña fuera de los resultados; la marca "TFL Selection" en Explorar mostraba un ícono genérico en vez del logo real.'
	]
};
