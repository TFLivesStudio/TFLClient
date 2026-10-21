// Canonical setting descriptors. Rendered groups and deep search use these same IDs/labels.
export type SettingsPage =
	'general' | 'appearance' | 'scenes' | 'game' | 'accounts' | 'console' | 'updates';
export const SETTINGS = [
	{
		id: 'language',
		page: 'general',
		label: ['Idioma', 'Language'],
		keywords: 'language idioma español english',
		current: 'settings.language',
		default: 'es',
		control: 'segment',
		storage: 'settings.json.language',
		profiles: 'All',
		behavior: 'instant',
		reason: 'Primary: language belongs to application behavior.'
	},
	{
		id: 'files',
		page: 'general',
		label: ['Archivos y diálogos', 'Files & dialogs'],
		keywords: 'file carpeta dialogos seleccionar archivo importar manual automatico',
		current: 'settings.native_dialog_mode',
		default: 'auto',
		control: 'segment',
		storage: 'settings.json.native_dialog_mode',
		profiles: 'All',
		behavior: 'instant',
		reason: 'Native integration; Manual enables local imports.'
	},
	{
		id: 'storage',
		page: 'general',
		label: ['Almacenamiento temporal', 'Temporary storage'],
		keywords: 'cache caché espacio disco limpiar almacenamiento storage cache',
		current: 'clear_temp_cache',
		default: 'action',
		control: 'action',
		storage: 'shared/temp',
		profiles: 'All',
		behavior: 'explicit',
		reason: 'Maintenance belongs with application files; does not delete games.'
	},
	{
		id: 'style',
		page: 'appearance',
		label: ['Estilo visual', 'Visual style'],
		keywords: 'minecrafti pixel modern diseño estilo visual',
		current: 'visual.style',
		default: 'modern',
		control: 'cards',
		storage: 'tfl-visual-refresh-v1.style',
		profiles: 'All',
		behavior: 'instant',
		reason: 'Primary visual choice before lower level details.'
	},
	{
		id: 'theme',
		page: 'appearance',
		label: ['Tema y superficie', 'Theme & surface'],
		keywords: 'dark light oled negro oscuro claro tema theme surface',
		current: 'settings.theme / tfl-surface',
		default: 'dark / obsidian',
		control: 'cards',
		storage: 'settings.json.theme + tfl-surface',
		profiles: 'All',
		behavior: 'instant',
		reason: 'OLED requires dark; handled as one coherent choice.'
	},
	{
		id: 'accent',
		page: 'appearance',
		label: ['Color de acento', 'Accent color'],
		keywords: 'cambiar color colores accent custom personalizado hex paleta',
		current: 'visual.customAccent / tfl-accent',
		default: 'orange',
		control: 'palette',
		storage: 'tfl-visual-refresh-v1.customAccent + tfl-accent',
		profiles: 'All',
		behavior: 'instant',
		reason: 'One palette includes presets and custom color; readable contrast is derived.'
	},
	{
		id: 'material',
		page: 'appearance',
		label: ['Material y transparencias', 'Material & transparency'],
		keywords: 'glass matte material cristal vidrio transparencias',
		current: 'visual.material',
		default: 'soft-glass',
		control: 'segment',
		storage: 'tfl-visual-refresh-v1.material',
		profiles: 'Non-Lite blur',
		behavior: 'instant',
		reason: 'Visual surface; blur capability is subject to quality and accessibility.'
	},
	{
		id: 'font',
		page: 'appearance',
		label: ['Tipografía', 'Typography'],
		keywords: 'fuente font tipografia tamaño letra clasica pixel',
		current: 'tfl-font / visual.pixelType',
		default: 'system / readable',
		control: 'select',
		storage: 'tfl-font + tfl-visual-refresh-v1.pixelType',
		profiles: 'All',
		behavior: 'instant',
		reason: 'Text readability next to style; Pixel classic stays optional.'
	},
	{
		id: 'density',
		page: 'appearance',
		label: ['Densidad y tarjetas', 'Density & cards'],
		keywords: 'density compacto tamaño escala densidad tarjetas cards',
		current: 'tfl-density / tfl-card-style',
		default: 'comfortable / rich',
		control: 'segment',
		storage: 'tfl-density + tfl-card-style',
		profiles: 'All',
		behavior: 'instant',
		reason: 'Amount of UI content and card presentation share a group.'
	},
	{
		id: 'quality',
		page: 'appearance',
		label: ['Calidad visual del launcher', 'Launcher visual quality'],
		keywords: 'lite balanced experience rendimiento visual calidad efectos fps',
		current: 'settings.quality_profile',
		default: 'Balanced',
		control: 'cards',
		storage: 'settings.json.quality_profile',
		profiles: 'Defines all',
		behavior: 'instant',
		reason: 'Distinct from game performance and RAM; profiles reset granular effect defaults.'
	},
	{
		id: 'motion',
		page: 'appearance',
		label: ['Movimiento y efectos', 'Motion & effects'],
		keywords: 'animacion animation blur desenfoque transparencias particles reducir movimiento',
		current: 'settings.reduce_animations / disable_blur_effects / disable_infinite_animations',
		default: 'false',
		control: 'toggles',
		storage: 'settings.json',
		profiles: 'Non-Lite',
		behavior: 'instant',
		reason: 'Accessibility and ongoing effects belong together; system reduced motion wins.'
	},
	{
		id: 'home',
		page: 'appearance',
		label: ['Organización de Inicio', 'Home organization'],
		keywords: 'home inicio paneles recientes servidores actividad sistema',
		current: 'visual.home',
		default: 'all visible',
		control: 'toggles',
		storage: 'tfl-visual-refresh-v1.home',
		profiles: 'All',
		behavior: 'instant',
		reason: 'Home content is a presentation choice with a contextual shortcut.'
	},
	{
		id: 'scene-current',
		page: 'scenes',
		label: ['Escena actual y vista previa', 'Current scene & preview'],
		keywords: 'escena ambiente wallpaper fondo background vista previa preview',
		current: 'scenes.global',
		default: 'legacy',
		control: 'preview',
		storage: 'tfl-scenes-v1',
		profiles: 'All',
		behavior: 'apply',
		reason: 'Preview does not overwrite the active scene before Apply.'
	},
	{
		id: 'scene-gallery',
		page: 'scenes',
		label: ['Galería de escenas', 'Scene gallery'],
		keywords: 'aurora bosque forest nether end cave pixel cielo noche escenas 2.5d presets',
		current: 'scenes.global.preset',
		default: 'legacy',
		control: 'cards',
		storage: 'tfl-scenes-v1',
		profiles: 'All; motion restricted',
		behavior: 'apply',
		reason: 'Visual choices belong in a gallery, not an engine form.'
	},
	{
		id: 'scene-static',
		page: 'scenes',
		label: ['Fondos estáticos', 'Static wallpapers'],
		keywords: 'fondo imagen estática wallpaper jpg png respaldo poster',
		current: 'tfl-wallpaper / scenes.poster',
		default: 'none',
		control: 'gallery',
		storage: 'tfl-wallpaper + shared/appearance',
		profiles: 'All',
		behavior: 'instant',
		reason: 'Existing wallpaper preferences retained; also serve as video fallback.'
	},
	{
		id: 'scene-video',
		page: 'scenes',
		label: ['Fondos de vídeo', 'Video wallpapers'],
		keywords: 'fondo animado animated wallpaper video mp4 webm importar loop codec',
		current: 'scenes.mediaId',
		default: 'none',
		control: 'import / preview',
		storage: 'shared/appearance/scenes + tfl-scenes-v1',
		profiles: 'Lite static; Balanced opt-in; Experience',
		behavior: 'apply',
		reason: 'Local media managed by backend; decode errors keep the poster.'
	},
	{
		id: 'scene-atmosphere',
		page: 'scenes',
		label: ['Luz y atmósfera', 'Light & atmosphere'],
		keywords: 'dia noche hora lluvia nieve niebla clima luz weather day night lighting',
		current: 'scenes.adaptTime / weather / lighting',
		default: 'time on / clear',
		control: 'segments',
		storage: 'tfl-scenes-v1',
		profiles: 'Lite static; others adaptive',
		behavior: 'apply',
		reason: 'Decorative weather is explicitly not a weather service.'
	},
	{
		id: 'scene-playback',
		page: 'scenes',
		label: ['Reproducción y rendimiento', 'Playback & performance'],
		keywords: 'fps movimiento parallax particulas particles shader gpu minecraft abierto pausa',
		current: 'scenes.motion / fps / particles / gamePolicy',
		default: 'subtle / 24 / true / reduce',
		control: 'segments',
		storage: 'tfl-scenes-v1',
		profiles: 'Central capability policy',
		behavior: 'apply',
		reason: 'Motion and work budget explained together; video retains encoded FPS.'
	},
	{
		id: 'scene-instance',
		page: 'scenes',
		label: ['Escenas por instancia', 'Scenes per instance'],
		keywords: 'instancia individual override escena fondo instance',
		current: 'scenes.instances',
		default: 'inherit global',
		control: 'select',
		storage: 'tfl-scenes-v1.instances',
		profiles: 'All',
		behavior: 'apply',
		reason: 'Separate visual override, independent of game folders and instances.'
	},
	{
		id: 'ram',
		page: 'game',
		label: ['Memoria para Minecraft', 'Memory for Minecraft'],
		keywords: 'memoria memory ram asignar minimo maximo megabytes',
		current: 'settings.min_memory / max_memory',
		default: '1024 / 2048 MB',
		control: 'slider + numeric',
		storage: 'settings.json; instance override',
		profiles: 'All',
		behavior: 'save',
		reason: 'Game launch budget; explain installed RAM and per-instance override.'
	},
	{
		id: 'java',
		page: 'game',
		label: ['Java para el juego', 'Game Java'],
		keywords: 'java jre runtime version ruta path instalación automatico',
		current: 'settings.jre*_path',
		default: 'managed',
		control: 'paths / status',
		storage: 'settings.json.jre*_path',
		profiles: 'All',
		behavior: 'save',
		reason: 'Runtime belongs to game execution; empty paths retain automatic selection.'
	},
	{
		id: 'jvm',
		page: 'game',
		label: ['Argumentos de Java', 'Java arguments'],
		keywords: 'jvm argumentos flags java opciones arranque heap',
		current: 'settings.jvm_args',
		default: 'empty',
		control: 'textarea',
		storage: 'settings.json.jvm_args',
		profiles: 'All',
		behavior: 'save',
		reason: 'Specialized but visible in the same game page; do not inject shell commands.'
	},
	{
		id: 'environment',
		page: 'game',
		label: ['Variables de entorno', 'Environment variables'],
		keywords: 'env variables entorno environment java ruta',
		current: 'settings.env_vars',
		default: '{}',
		control: 'key=value editor',
		storage: 'settings.json.env_vars',
		profiles: 'All',
		behavior: 'save',
		reason: 'Execution context; values can contain sensitive data and are not logged.'
	},
	{
		id: 'multi-instance',
		page: 'game',
		label: ['Varias instancias a la vez', 'Multiple running instances'],
		keywords: 'multi varias instancias simultaneas warning aviso',
		current: 'settings.multi_instance_warning_dismissed',
		default: 'false',
		control: 'toggle',
		storage: 'settings.json',
		profiles: 'All',
		behavior: 'instant',
		reason: 'User-facing guard for concurrent game launches, not visual layout.'
	},
	{
		id: 'accounts',
		page: 'accounts',
		label: ['Tus cuentas', 'Your accounts'],
		keywords: 'cuenta microsoft offline login sesion autenticar identity',
		current: 'settings.user / active_user_idx',
		default: 'existing active account',
		control: 'list / actions',
		storage: 'settings.json; protected tokens',
		profiles: 'All',
		behavior: 'explicit',
		reason: 'Identity actions kept through existing auth API.'
	},
	{
		id: 'account-microsoft',
		page: 'accounts',
		label: ['Añadir cuenta Microsoft', 'Add Microsoft account'],
		keywords: 'microsoft login premium iniciar sesion cuenta codigo',
		current: 'device-code auth',
		default: 'action',
		control: 'device code',
		storage: 'existing auth API',
		profiles: 'All',
		behavior: 'explicit',
		reason: 'Use real flow, preserve cancellation and errors.'
	},
	{
		id: 'account-offline',
		page: 'accounts',
		label: ['Añadir cuenta sin conexión', 'Add offline account'],
		keywords: 'offline sin conexion cuenta usuario nombre',
		current: 'offline account',
		default: 'action',
		control: 'validated name',
		storage: 'settings.json.user',
		profiles: 'All',
		behavior: 'explicit',
		reason: 'Distinct identity type; no invented premium capabilities.'
	},
	{
		id: 'account-skin',
		page: 'accounts',
		label: ['Skins y mascotas', 'Skins & mascots'],
		keywords: 'skin capa cape mascota nori perfil avatar character',
		current: 'mascot UUID / skin studio',
		default: 'deterministic mascot',
		control: 'context action',
		storage: 'tfl-mascots + existing Mojang API',
		profiles: 'Profile policy',
		behavior: 'explicit',
		reason: 'Open existing studio; do not change authentication.'
	},
	{
		id: 'console-layout',
		page: 'console',
		label: ['Diseñar la consola', 'Design your console'],
		keywords: 'consola console layout arrastrar mover molde monitor inferior lateral superior',
		current: 'console.dock / columns / order',
		default: 'top / 2 / CPU,RAM,disk',
		control: 'WYSIWYG',
		storage: 'tfl-console-v1',
		profiles: 'All',
		behavior: 'apply / undo',
		reason: 'One shared layout renderer, demo data clearly labeled.'
	},
	{
		id: 'console-metrics',
		page: 'console',
		label: ['Gráficas y métricas', 'Charts & metrics'],
		keywords: 'cpu ram disco monitor historial grafica anillos lineas color porcentaje nucleos',
		current: 'console.visible / colors / chartStyle / seconds / cpuMode',
		default: 'CPU + RAM; lines; 60s; cores',
		control: 'context inspector',
		storage: 'tfl-console-v1',
		profiles: 'All',
		behavior: 'apply',
		reason: '100% CPU means one core; resident RAM is not heap.'
	},
	{
		id: 'console-log',
		page: 'console',
		label: ['Texto y niveles del registro', 'Log text & levels'],
		keywords: 'warn error logs consola font fuente filtros seguimiento colores',
		current: 'console.fontSize / wrap / follow / colors',
		default: '12 / true / true',
		control: 'context inspector',
		storage: 'tfl-console-v1',
		profiles: 'All',
		behavior: 'apply',
		reason: 'Readability, severity, search and follow share the log block.'
	},
	{
		id: 'updates',
		page: 'updates',
		label: ['Actualizaciones del launcher', 'Launcher updates'],
		keywords: 'version beta canal actualizaciones update release buscar descargar instalar',
		current: 'settings.auto_updates / beta_updates; updater state',
		default: 'auto true / beta false',
		control: 'status + actions',
		storage: 'settings.json + updater machine',
		profiles: 'All',
		behavior: 'explicit',
		reason: 'Beta consent and signed updater remain intact; version notes nearby.'
	}
] as const;
export type SettingId = (typeof SETTINGS)[number]['id'];
export function setting(id: SettingId) {
	return SETTINGS.find((s) => s.id === id)!;
}
export const normalized = (value: string) =>
	value
		.toLowerCase()
		.normalize('NFD')
		.replace(/[\u0300-\u036f]/g, '')
		.trim();
export function searchSettings(query: string, locale: 'es' | 'en') {
	const words = normalized(query).split(/\s+/).filter(Boolean);
	if (!words.length) return [];
	return SETTINGS.map((s) => ({
		setting: s,
		score: words.reduce(
			(score, word) =>
				score +
				(normalized(s.label[locale === 'en' ? 1 : 0]).includes(word)
					? 4
					: normalized(`${s.keywords} ${s.label.join(' ')}`).includes(word)
						? 1
						: 0),
			0
		)
	}))
		.filter((r) =>
			words.every((word) =>
				normalized(`${r.setting.label.join(' ')} ${r.setting.keywords}`).includes(word)
			)
		)
		.sort((a, b) => b.score - a.score)
		.map((r) => r.setting);
}
export const PAGES: { id: SettingsPage; label: [string, string]; description: [string, string] }[] =
	[
		{
			id: 'general',
			label: ['General', 'General'],
			description: [
				'Idioma, archivos e integración con tu equipo.',
				'Language, files and integration with your computer.'
			]
		},
		{
			id: 'appearance',
			label: ['Apariencia', 'Appearance'],
			description: [
				'Estilo, color y legibilidad. Los cambios se aplican al instante y no modifican el rendimiento del juego.',
				'Style, color and readability. Changes apply immediately and do not change game performance.'
			]
		},
		{
			id: 'scenes',
			label: ['Fondos y escenas', 'Backgrounds & scenes'],
			description: [
				'El ambiente de tus mundos. Previsualiza una escena antes de aplicarla.',
				'The atmosphere of your worlds. Preview a scene before applying it.'
			]
		},
		{
			id: 'game',
			label: ['Minecraft y Java', 'Minecraft & Java'],
			description: [
				'Valores globales para iniciar el juego. Los ajustes de cada instancia pueden reemplazarlos.',
				'Global game launch defaults. Individual instances can override them.'
			]
		},
		{
			id: 'accounts',
			label: ['Cuentas', 'Accounts'],
			description: [
				'Tus identidades, sesiones, skins y mascotas.',
				'Your identities, sessions, skins and mascots.'
			]
		},
		{
			id: 'console',
			label: ['Consola y monitoreo', 'Console & monitoring'],
			description: [
				'Diseña tu consola con bloques reales. La vista previa usa datos de demostración.',
				'Design your console with real blocks. The preview uses demonstration data.'
			]
		},
		{
			id: 'updates',
			label: ['Actualizaciones', 'Updates'],
			description: [
				'Versión instalada, canal y actualizaciones del launcher.',
				'Installed version, channel and launcher updates.'
			]
		}
	];
