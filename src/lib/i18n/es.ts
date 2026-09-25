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
		language: 'Idioma'
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
