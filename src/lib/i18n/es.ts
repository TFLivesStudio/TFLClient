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
	}
};

export type Dictionary = typeof es;
