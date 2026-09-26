// Diccionario base — español, el idioma por defecto del launcher. Cada
// namespace corresponde más o menos a un componente, para que agregar o
// tocar textos de una pantalla no obligue a bucear todo el archivo.
export const es = {
	common: {
		cancel: 'Cancelar',
		back: 'Volver',
		continue: 'Continuar',
		copy: 'Copiar',
		copied: 'Copiado'
	},
	onboarding: {
		tagline: 'Iniciá sesión para empezar a jugar',
		microsoftTitle: 'Cuenta Microsoft',
		microsoftSub: 'Acceso completo, multijugador incluido',
		offlineTitle: 'Cuenta offline',
		offlineSub: 'Solo singleplayer',
		usernameLabel: 'Nombre de usuario',
		offlineHint: 'Las cuentas offline solo pueden jugar en singleplayer.',
		fetchingCode: 'Obteniendo código...',
		goTo: 'Andá a',
		enterCode: 'e ingresá el código:',
		waitingConfirmation: 'Esperando confirmación...',
		copyCodeFailed: 'No se pudo copiar. Seleccioná el código y copialo manualmente.'
	},
	settings: {
		language: 'Idioma',
		title: 'Ajustes',
		close: 'Cerrar',
		tabs: {
			appearance: 'Estilo',
			general: 'General',
			accounts: 'Cuentas',
			java: 'Java'
		},
		theme: {
			label: 'Tema',
			dark: 'Oscuro',
			light: 'Claro',
			lightDisabledTitle: 'Modo claro no compatible con superficie OLED'
		},
		quality: {
			label: 'Perfil de calidad'
		},
		ram: {
			label: 'Memoria (RAM) global',
			detected: '— {{gb}} GB detectados',
			min: 'Mínima (MB)',
			max: 'Máxima (MB)',
			save: 'Guardar'
		},
		storage: {
			label: 'Almacenamiento',
			clearCache: 'Limpiar caché temporal',
			cacheCleared: 'Liberados {{mb}} MB.',
			clearCacheFailed: 'No se pudo limpiar: {{error}}'
		},
		dialogs: {
			label: 'Diálogos de archivo nativos',
			auto: 'Automático',
			manual: 'Manual',
			hint: 'Automático (recomendado): subir tu propio ícono/wallpaper o agregar mods por archivo queda desactivado — no hace falta, todo se instala solo. Manual: lo habilita, pero en algunos casos puede cerrar el launcher de golpe (diálogo nativo de archivos).'
		},
		updates: {
			label: 'Actualizaciones',
			autoEnabled: 'Auto-actualizar: Activado',
			autoDisabled: 'Auto-actualizar: Desactivado',
			hint: 'Con esto activado, el launcher chequea y baja actualizaciones solo al abrir, en segundo plano — vos elegís cuándo instalar con el aviso que aparece.',
			check: 'Buscar actualizaciones',
			upToDate: 'Ya tenés la última versión.',
			available: 'Actualización disponible: v{{version}}',
			download: 'Descargar e instalar'
		},
		accent: {
			label: 'Color de acento',
			orange: 'Naranja',
			violet: 'Violeta',
			teal: 'Verde azulado',
			blue: 'Azul eléctrico',
			rose: 'Rosa plasma',
			lime: 'Lima',
			iris: 'Iris',
			jade: 'Jade',
			crimson: 'Carmesí',
			cyan: 'Cian',
			grass: 'Césped',
			plum: 'Ciruela'
		},
		surface: {
			label: 'Superficie',
			incompatibleTitle: 'Superficie OLED no compatible con modo claro',
			obsidian: 'Obsidiana',
			midnight: 'Medianoche',
			slate: 'Pizarra',
			oled: 'OLED'
		},
		ambience: {
			label: 'Efecto ambiental',
			aurora: 'Aurora',
			cosmic: 'Cósmico',
			minimal: 'Minimal',
			particles: 'Partículas'
		},
		wallpaper: {
			label: 'Fondo de pantalla',
			yourImage: 'Tu imagen',
			change: 'Cambiar tu imagen',
			upload: 'Subir tu imagen',
			uploadDisabledTitle: 'Activá el modo Manual (más abajo) para usar esto',
			autoModeDisabled: 'Desactivado en modo Automático — activá "Manual" en la sección de abajo.',
			items: {
				none: 'Ninguno',
				voidNight: 'Noche vacía',
				nether: 'Nether',
				end: 'El End',
				deepOcean: 'Océano',
				auroraGradient: 'Aurora',
				animatedAurora: 'Aurora animada',
				obsidianSolid: 'Obsidiana',
				charcoal: 'Carbón',
				savanna: 'Sabana',
				goldenSunset: 'Atardecer dorado',
				lakeNight: 'Noche en el lago',
				neonArcade: 'Arcade nocturno',
				redCanyon: 'Cañón rojo',
				enchantedValley: 'Valle encantado',
				snowyPeak: 'Cumbre nevada',
				stoneBridge: 'Puente de piedra',
				mistyFortress: 'Fortaleza en la niebla',
				villageTower: 'Torre del pueblo',
				deepCave: 'Cueva profunda',
				sunsetCoast: 'Costa al atardecer',
				abstractBlocks: 'Cubos abstractos'
			}
		},
		density: {
			label: 'Densidad de interfaz',
			comfortable: 'Cómoda',
			compact: 'Compacta'
		},
		cardStyle: {
			label: 'Estilo de tarjeta',
			rich: 'Rica',
			minimal: 'Minimal'
		},
		font: {
			label: 'Fuentes',
			system: 'Sistema',
			hint: 'Se aplica al instante, sin reiniciar el launcher.'
		},
		accounts: {
			saved: 'Cuentas guardadas',
			use: 'Usar',
			active: 'Activa',
			remove: 'Quitar cuenta',
			addMicrosoft: 'Agregar cuenta Microsoft',
			microsoftEnter: 'e ingresá:',
			waitingConfirmation: 'Esperando confirmación…',
			copyCode: 'Copiar código',
			startMicrosoft: 'Iniciar con Microsoft',
			addOffline: 'Agregar cuenta offline'
		},
		java: {
			label: 'Runtimes de Java',
			hint: 'Se instalan solos la primera vez que una instancia los necesita — no hace falta instalar Java a mano.',
			major: 'Java {{major}}',
			installed: 'Instalado',
			notInstalled: 'No instalado'
		}
	},
	sidebar: {
		offline: 'Sin conexión — jugá lo que ya tenés instalado',
		yourInstances: 'Tus instancias',
		createInstance: 'Crear instancia',
		search: 'Buscar…',
		noInstancesYet: 'Todavía no creaste ninguna instancia',
		createFirst: 'Crear la primera',
		noResultsFor: 'Sin resultados para "{{query}}"',
		offlineAccountType: 'Offline — solo singleplayer',
		openSettings: 'Ajustes',
		logout: 'Cerrar sesión',
		joinServer: 'Unirse a servidor'
	},
	joinServer: {
		title: 'Unirse a servidor',
		addressLabel: 'Dirección del servidor',
		addressPlaceholder: 'ejemplo.com:25565',
		modeExisting: 'Una instancia mía',
		modeQuick: 'Quick Join',
		noInstances: 'No tenés ninguna instancia de cliente todavía — usá Quick Join.',
		quickHint: 'Crea (o reusa) una instancia Vanilla con esa versión y te conecta directo, sin pasar por el menú de Multijugador.',
		crackedBlocked: 'Las cuentas offline solo pueden jugar en singleplayer — iniciá sesión con una cuenta Microsoft para unirte a un servidor.',
		join: 'Unirse',
		joining: 'Conectando…'
	},
	createInstance: {
		title: 'Nueva instancia',
		chooserSubtitle: '¿Cómo la querés armar?',
		customTitle: 'Personalizada',
		customDesc: 'Creá tu propia instancia con los mods que vos elijas.',
		modpackTitle: 'Modpack',
		modpackDesc: 'Descargá una instancia ya armada de TFL Selection.',
		serverTitle: 'Servidor',
		serverDesc: 'Creá tu propio servidor de Minecraft para jugar con otros.',
		nameLabel: 'Nombre',
		namePlaceholder: 'Mi mundo',
		versionLabel: 'Versión de Minecraft',
		loadingVersions: 'Cargando versiones…',
		loaderLabel: 'Loader',
		loaderHint:
			'La versión del loader (Fabric/Forge/NeoForge/Quilt) se resuelve sola — la más reciente/recomendada para esta versión de Minecraft.',
		experimentalToggle: 'Mostrar versiones experimentales (snapshots, pre-releases, release candidates)',
		create: 'Crear'
	},
	createServer: {
		title: 'Nuevo servidor',
		serverTypeLabel: 'Tipo de servidor',
		vanillaDesc: 'El servidor oficial de Mojang, sin plugins.',
		paperDesc: 'El más usado — rápido y con soporte de plugins.',
		purpurDesc: 'Basado en Paper, con más opciones de personalización.',
		versionLabel: 'Versión de Minecraft',
		noVersionsForType: 'No hay versiones disponibles para este tipo de servidor todavía.',
		loadingVersions: 'Cargando versiones…',
		create: 'Crear servidor',
		creating: 'Descargando el servidor…'
	},
	contentManager: {
		manageTab: 'Gestionar',
		downloadTab: 'Descargar',
		categories: 'Categorías',
		selectNone: 'Ninguno',
		selectAll: 'Seleccionar todos',
		manualModeRequired: 'Activá el modo Manual en Ajustes para usar esto',
		addByFile: 'Añadir por archivo',
		checkUpdates: 'Buscar actualizaciones',
		remove: 'Quitar',
		autoModeDisablesAddByFile: 'Añadir por archivo está desactivado en modo Automático — activá "Manual" en Ajustes.',
		allUpToDate: 'Todo al día — no hay actualizaciones.',
		noChangelogNotes: 'Sin notas de cambios.',
		changelogLoadFailed: 'No se pudo cargar: {{error}}',
		partialLocalAdd: 'Se agregaron {{added}} de {{total}} archivos — el resto no tenía la extensión esperada o no se pudo copiar.',
		removeFailedPartial: 'No se pudieron quitar {{failed}} de {{total}}: {{names}}',
		duplicateOne: 'Hay un mod',
		duplicateMany: 'Hay {{count}} mods',
		duplicateWarning: 'instalado dos veces (versiones distintas del mismo mod a la vez) — puede causar crashes. Revisá: {{list}}',
		updateAll: 'Actualizar todos',
		notInstalledYet: 'Todavía no instalaste ningún {{kind}} acá.',
		noneMatchCategories: 'Ningún {{kind}} instalado coincide con esas categorías.',
		select: 'Seleccionar',
		changelogButton: 'Novedades',
		searchPlaceholder: 'Buscar {{noun}} en Modrinth…',
		favorites: 'Favoritos',
		results: 'Resultados',
		favorite: 'Favorito',
		viewVersions: 'Ver versiones',
		alreadyInstalled: 'Ya instalado',
		noCompatibleVersions: 'No hay versiones compatibles con esta instancia.'
	},
	instanceDetail: {
		changeIcon: 'Cambiar ícono de la instancia',
		rename: 'Renombrar',
		duplicate: 'Duplicar',
		duplicateTitle: 'Duplicar instancia',
		exportMrpack: 'Exportar como .mrpack',
		verifyIntegrity: 'Verificar integridad',
		delete: 'Eliminar',
		deleteTitle: 'Eliminar instancia',
		deleteMessage:
			'"{{name}}" y todos sus archivos (mundos, mods, configs) se van a borrar. No se puede deshacer.',
		exported: 'Exportado: {{path}}',
		integrityOk: 'Todo en orden — no falta ningún archivo instalado.',
		integrityMissing: 'Faltan {{count}} archivo{{plural}} que un modpack instaló: {{list}}',
		neverPlayed: 'Nunca jugada',
		playedToday: 'Hoy',
		playedYesterday: 'Ayer',
		playedDaysAgo: 'Hace {{days}} días',
		preparing: 'Preparando…',
		instanceRunning: '"{{name}}" está corriendo',
		play: 'Jugar',
		modsNotSupportedVanilla: 'No soportado en Vanilla',
		loading: 'Cargando…',
		modsInstalledCount: '{{count}} instalados',
		folder: 'Carpeta',
		instanceFiles: 'Archivos de la instancia',
		tabs: {
			details: 'Detalles',
			mods: 'Mods',
			shaders: 'Shaders',
			resourcePacks: 'Resource Packs',
			modpacks: 'Modpacks',
			screenshots: 'Capturas'
		},
		ram: {
			sectionLabel: 'Memoria (esta instancia)',
			hint: 'Vacío usa el valor global de Ajustes{{recommended}}.',
			recommendedSuffix: ' (recomendado: {{mb}} MB)',
			globalPlaceholder: 'Global'
		}
	},
	instanceIconPicker: {
		title: 'Ícono de la instancia',
		uploadOwn: 'Subir mi propia imagen',
		autoModeNotice: 'Desactivado en modo Automático — activá "Manual" en Ajustes para usarlo.'
	},
	instanceLogWindow: {
		stopFailed: 'No se pudo detener: {{error}}',
		running: 'Corriendo…',
		exited: 'Proceso finalizado{{codeSuffix}}',
		exitedCodeSuffix: ' (código {{code}})',
		stats: '· CPU {{cpu}}% · RAM {{ram}} MB',
		stop: 'Detener',
		closeNotice: 'Cerrar aviso',
		waitingOutput: 'Esperando salida del juego…',
		errors: {
			outOfMemory: 'Se quedó sin memoria RAM asignada — probá subir la RAM máxima en Ajustes.',
			unsupportedJavaVersion:
				'Esta versión de Minecraft necesita una versión de Java más nueva que la instalada.',
			insufficientSystemRam:
				'Le pediste más RAM de la que tiene disponible tu compu — bajá el máximo en Ajustes.',
			jniError: 'Conflicto entre mods, o una versión de Java incompatible con esta instancia.',
			mixinConflict: 'Un mod (mixin) chocó con otro — probá sacar el último mod que instalaste.',
			missingDependency:
				'Falta una dependencia que un mod necesita — revisá si instalaste todo lo que pedía.',
			duplicateMod: 'Tenés el mismo mod instalado dos veces — revisá la lista de mods de la instancia.',
			nativeCrash:
				'La JVM crasheó de forma nativa (no es un error de un mod puntual) — puede ser drivers de video desactualizados, o falta de RAM real de la compu. Java dejó un hs_err_pid*.log con el detalle en la carpeta de la instancia.'
		},
		crashSummary: 'La JVM crasheó de forma nativa — resumen del reporte:\n{{summary}}',
		exitOom:
			'El sistema operativo cortó el juego por falta de memoria RAM (no solo la asignada al juego — memoria real de la compu). Cerrá otros programas o bajá la memoria máxima en Ajustes.',
		exitNullCode:
			'El juego se cerró de golpe sin avisar — normalmente es el sistema operativo cortando el proceso por falta de RAM. Probá subir la memoria máxima en Ajustes.',
		exitOtherCode:
			'El juego se cerró con código {{code}} sin un error reconocible en el log — con varios mods instalados, suele ser falta de RAM. Probá subir la memoria máxima en Ajustes.'
	},
	nativeDialogModePrompt: {
		title: '¿Cómo querés instalar contenido propio?',
		lead: 'Esto define si vas a poder subir tu propio ícono/wallpaper o agregar mods por archivo a mano. Lo podés cambiar cuando quieras desde Ajustes.',
		autoRecommended: 'Automático (recomendado)',
		autoSub: 'Todo se instala solo — sin diálogos de archivo, sin riesgo de crash.',
		manualSub:
			'Habilita subir tu propio ícono/wallpaper y mods por archivo — en algunos casos puede cerrar el launcher de golpe.',
		bannerSub: 'Automático: sin diálogos de archivo. Manual: los habilita, con riesgo de crash.',
		dismissAriaLabel: 'Cerrar sin elegir todavía',
		dismissTitle: 'Cerrar sin elegir — se vuelve a preguntar la próxima vez'
	},
	screenshotsPanel: {
		hint: 'Capturas de esta instancia (F2 en el juego las guarda acá). El launcher no saca capturas por sí mismo, solo las muestra y las administra.',
		openFolder: 'Abrir carpeta',
		empty: 'Sin capturas todavía. Jugá y presioná F2 para guardar una.',
		copyImage: 'Copiar imagen',
		delete: 'Borrar'
	},
	tflSelection: {
		eyebrow: 'Contenido destacado',
		subtitle: 'Descubrí packs curados y añadilos a una instancia compatible, o creá una nueva al toque.',
		allTab: 'Todos',
		empty: 'Todavía no hay modpacks en la selección.',
		createNewOption: '＋ Crear instancia nueva — {{mcVersion}} ({{loader}})',
		needsLoaderInstance: 'Necesitás una instancia con Fabric, Forge, NeoForge o Quilt para instalar este pack.',
		noMatchingInstance: 'Ninguna instancia tuya coincide con las versiones de este pack.',
		added: 'Añadido',
		add: 'Añadir',
		noVersionBuild: 'No hay un build de este pack para esa versión',
		noInstanceBuild: 'No hay un build de este pack para esa instancia'
	},
	modpacksPanel: {
		vanillaNotSupported: 'Vanilla no soporta modpacks — elegí Fabric, Forge, NeoForge o Quilt al crear la instancia.',
		hint: 'Instalar un modpack agrega sus mods y configuración a esta instancia — no reemplaza lo que ya tenías. Quitarlo borra exactamente lo que trajo, nada más.',
		installedCount: 'Instalados ({{count}})',
		update: 'Actualizar'
	},
	shadersPanel: {
		qualityLabel: 'Calidad gráfica (rápido)',
		presetLow: 'Bajo',
		presetMedium: 'Medio',
		presetHigh: 'Alto'
	},
	home: {
		exploreTflSelection: 'Explorar TFL Selection',
		title: 'Tu biblioteca Minecraft,\nbien organizada.',
		subtitle: 'Creá una instancia para jugar, instalar contenido y ajustar cada perfil.',
		modsCardTitle: 'Mods y modpacks',
		modsCardHint: 'Por instancia',
		javaCardTitle: 'Java automático',
		javaCardHint: 'Sin configuración manual'
	},
	whatsNewTips: {
		welcomeTitle: 'Bienvenido a TFL Client',
		updatesTitle: 'Novedades en v{{version}}',
		tip1: 'Ctrl+K (⌘K en Mac) abre una paleta para saltar entre instancias o acciones sin tocar el mouse.',
		tip2: '"TFL Selection" trae modpacks curados y los que publique el equipo — un click y quedan listos.',
		tip3: 'Cada instancia tiene su propia pestaña de Mods, Shaders y Modpacks — nada se mezcla entre instancias.',
		tip4: 'En Ajustes → Estilo podés cambiar acento, superficie, densidad y hasta el fondo animado.',
		updatedToVersionWithNotes: 'Se actualizó solo a la versión {{version}}:',
		viewTechnicalDetail: 'Ver el detalle técnico en GitHub',
		updatedToVersionNoNotes:
			'Se actualizó solo a la versión {{version}}. El detalle completo de qué cambió está en el Release de GitHub.',
		viewFullRelease: 'Ver el Release completo',
		gotIt: 'Entendido'
	},
	titleBar: {
		minimize: 'Minimizar',
		maximize: 'Maximizar',
		restore: 'Restaurar'
	},
	updateBadge: {
		available: 'Actualización disponible',
		installing: 'Instalando…',
		clickToInstall: 'v{{version}} — click para instalar'
	},
	commandPalette: {
		action: 'Acción',
		searchPlaceholder: 'Buscar instancias o acciones…',
		noResults: 'Sin resultados'
	},
	confirmDialog: {
		confirm: 'Confirmar'
	},
	downloadProgressBar: {
		resolving: 'Resolviendo…',
		library: 'Descargando librerías',
		asset: 'Descargando assets',
		native: 'Descargando nativos',
		client: 'Descargando cliente',
		verifying: 'Verificando',
		extracting: 'Extrayendo',
		processing: 'Procesando',
		jre: 'Descargando Java',
		downloading: 'Descargando',
		generic: 'Trabajando…'
	},
	modsPanel: {
		vanillaHint: 'Vanilla no soporta mods — elegí Fabric, Forge, NeoForge o Quilt al crear la instancia.'
	},
	resourcePacksPanel: {
		hint: 'Los resource packs funcionan en cualquier instancia, con o sin mods — no necesitás nada extra instalado. Se activan desde el menú de Minecraft (Opciones → Paquetes de recursos).'
	},
	pluginsPanel: {
		vanillaHint: 'Vanilla no soporta plugins — elegí Paper o Purpur al crear el servidor.'
	},
	serverConsole: {
		notRunning: 'El servidor no está corriendo.',
		waitingOutput: 'Esperando salida del servidor…',
		commandPlaceholder: 'Escribí un comando y presioná Enter…',
		send: 'Enviar'
	},
	serverWorlds: {
		hint: 'Mundos detectados en la carpeta del servidor.',
		empty: 'Todavía no hay ningún mundo generado — se crea solo al iniciar el servidor por primera vez.',
		delete: 'Borrar mundo',
		confirmDelete: '¿Seguro? Borrar'
	},
	serverFileManager: {
		newFolder: 'Nueva carpeta',
		newFolderPrompt: 'Nombre de la carpeta',
		up: 'Subir un nivel',
		empty: 'Esta carpeta está vacía.',
		save: 'Guardar'
	},
	serverInstance: {
		start: 'Iniciar servidor',
		starting: 'Iniciando…',
		stop: 'Detener servidor',
		stopping: 'Deteniendo…',
		build: 'Build',
		tabPlugins: 'Plugins',
		tabWorlds: 'Mundos',
		tabFiles: 'Archivos',
		tabConsole: 'Consola',
		connectionLabel: 'Cómo conectarse',
		connectionLanHint: 'Para jugar con otra PC en tu misma red Wi-Fi, pasales esa dirección tal cual.',
		connectionPublicReady: 'Esta dirección ya sirve para cualquiera, esté donde esté — TFL Client abrió el puerto solo en tu router.',
		connectionPublicManual:
			'Esta dirección sirve para cualquiera, pero tu router no dejó abrirla sola: hay que abrir (port forward) el puerto {{port}} a mano — buscá "port forwarding" + el modelo de tu router.',
		connectionLanFallback: 'Si están en la misma red Wi-Fi, también pueden usar {{address}} directamente.',
		connectionUnavailable: 'No se pudo detectar la dirección de red — revisá que tengas conexión a internet o red local.',
		connectionTunnelReady: 'Esta dirección ya sirve para cualquiera, esté donde esté — no depende de tu router.',
		playitPitch: '¿Se sigue complicando la conexión? Vinculá una cuenta gratis de playit.gg (un click, sin tarjeta) y la dirección va a andar siempre, sin depender del router.',
		playitLinkButton: 'Vincular playit.gg',
		playitLinking: 'Esperando confirmación en el navegador…'
	},
	skinManager: {
		title: 'Skin y capa',
		tabSkin: 'Skin',
		tabCape: 'Capa',
		variantClassic: 'Clásico',
		variantSlim: 'Slim (brazos finos)',
		uploadSkin: 'Subir skin',
		resetSkin: 'Restaurar por defecto',
		noCapes: 'Esta cuenta no tiene ninguna capa — se consiguen con eventos/promociones de Mojang.',
		noCape: 'Sin capa'
	}
};

export type Dictionary = typeof es;
