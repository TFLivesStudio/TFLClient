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
		logout: 'Cerrar sesión'
	},
	createInstance: {
		title: 'Nueva instancia',
		chooserSubtitle: '¿Cómo la querés armar?',
		customTitle: 'Personalizada',
		customDesc: 'Creá tu propia instancia con los mods que vos elijas.',
		modpackTitle: 'Modpack',
		modpackDesc: 'Descargá una instancia ya armada de TFL Selection.',
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
	}
};

export type Dictionary = typeof es;
