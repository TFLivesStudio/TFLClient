export const SCENE_IDS = [
	'legacy',
	'forest',
	'embers',
	'void',
	'night',
	'cave',
	'aurora',
	'pixel'
] as const;
export type SceneId = (typeof SCENE_IDS)[number];
export interface SceneConfig {
	preset: SceneId;
	mediaId: string | null;
	poster: string;
	motion: 'off' | 'subtle' | 'full';
	fps: 15 | 24 | 30 | 60;
	particles: boolean;
	weather: 'clear' | 'mist' | 'rain' | 'snow';
	adaptTime: boolean;
	lighting: boolean;
	reactive: boolean;
	balancedVideo: boolean;
	gamePolicy: 'pause' | 'reduce' | 'keep';
	shader: boolean;
}
export interface ScenePreferences {
	version: 1;
	global: SceneConfig;
	instances: Record<string, SceneConfig>;
}
export const DEFAULT_SCENE: SceneConfig = {
	preset: 'legacy',
	mediaId: null,
	poster: 'auto',
	motion: 'subtle',
	fps: 24,
	particles: true,
	weather: 'clear',
	adaptTime: true,
	lighting: true,
	reactive: true,
	balancedVideo: false,
	gamePolicy: 'reduce',
	shader: false
};
export const DEFAULT_SCENES: ScenePreferences = {
	version: 1,
	global: DEFAULT_SCENE,
	instances: {}
};
const object = (v: unknown): Record<string, unknown> =>
	v && typeof v === 'object' && !Array.isArray(v) ? (v as Record<string, unknown>) : {};
export function parseScene(raw: unknown): SceneConfig {
	const s = object(raw),
		d = structuredClone(DEFAULT_SCENE);
	if (SCENE_IDS.includes(s.preset as SceneId)) d.preset = s.preset as SceneId;
	if (
		typeof s.mediaId === 'string' &&
		/^(?:builtin-(?:aurora|embers)|scene-[a-f0-9]{10,40})$/.test(s.mediaId)
	)
		d.mediaId = s.mediaId;
	if (
		s.poster === 'custom' ||
		(typeof s.poster === 'string' && /^mc-(?:[1-9]|1[0-3])$/.test(s.poster))
	)
		d.poster = s.poster;
	if (['off', 'subtle', 'full'].includes(s.motion as string))
		d.motion = s.motion as SceneConfig['motion'];
	if ([15, 24, 30, 60].includes(s.fps as number)) d.fps = s.fps as SceneConfig['fps'];
	if (['clear', 'mist', 'rain', 'snow'].includes(s.weather as string))
		d.weather = s.weather as SceneConfig['weather'];
	if (['pause', 'reduce', 'keep'].includes(s.gamePolicy as string))
		d.gamePolicy = s.gamePolicy as SceneConfig['gamePolicy'];
	for (const key of [
		'particles',
		'adaptTime',
		'lighting',
		'reactive',
		'balancedVideo',
		'shader'
	] as const)
		if (typeof s[key] === 'boolean') d[key] = s[key];
	return d;
}
export function parseScenes(raw: unknown): ScenePreferences {
	const s = object(raw),
		instances: Record<string, SceneConfig> = {};
	if (s.version !== undefined && s.version !== 1) return structuredClone(DEFAULT_SCENES);
	for (const [id, value] of Object.entries(object(s.instances)).slice(0, 2000))
		if (!['__proto__', 'constructor', 'prototype'].includes(id)) instances[id] = parseScene(value);
	return { version: 1, global: parseScene(s.global), instances };
}
export interface ScenePolicyInput {
	quality: 'lite' | 'balanced' | 'experience';
	reduced: boolean;
	active: boolean;
	gameRunning: boolean;
	config: SceneConfig;
}
export function sceneCapabilities({
	quality,
	reduced,
	active,
	gameRunning,
	config: c
}: ScenePolicyInput) {
	const paused = !active || (gameRunning && c.gamePolicy === 'pause');
	const reducedGame = gameRunning && c.gamePolicy === 'reduce';
	const motion = quality !== 'lite' && !reduced && !paused && !reducedGame && c.motion !== 'off';
	return {
		motion,
		video: motion && (quality === 'experience' || c.balancedVideo),
		parallax: motion,
		particles: motion && c.particles,
		shader: motion && quality === 'experience' && c.shader,
		maxParticles: quality === 'experience' ? 24 : 8,
		fps: motion ? Math.min(c.fps, quality === 'experience' ? 60 : 24) : 0,
		dpr: quality === 'experience' ? 1.5 : 1,
		depth: motion ? (quality === 'experience' && c.motion === 'full' ? 1 : 0.4) : 0
	};
}
export function dayPhase(hour: number) {
	return hour >= 6 && hour < 11
		? 'morning'
		: hour < 17 && hour >= 11
			? 'day'
			: hour >= 17 && hour < 20
				? 'sunset'
				: 'night';
}
export const SCENES: {
	id: SceneId;
	name: [string, string];
	description: [string, string];
	colors: [string, string, string];
}[] = [
	{
		id: 'legacy',
		name: ['Tu fondo actual', 'Your current background'],
		description: ['Conserva la ambientación anterior.', 'Keeps the existing atmosphere.'],
		colors: ['#161a20', '#262c37', '#ff7a2e']
	},
	{
		id: 'forest',
		name: ['Portal Glade', 'Portal Glade'],
		description: [
			'Un portal antiguo entre árboles y flores.',
			'An ancient portal among trees and flowers.'
		],
		colors: ['#242e1e', '#557754', '#6b9569']
	},
	{
		id: 'embers',
		name: ['Crimson Dusk', 'Crimson Dusk'],
		description: ['Un atardecer rojo sobre aguas en calma.', 'A crimson sunset over still water.'],
		colors: ['#350b10', '#8c312f', '#ee8756']
	},
	{
		id: 'void',
		name: ['Ender Twilight', 'Ender Twilight'],
		description: [
			'Endermen y torres bajo un cielo violeta.',
			'Endermen and towers beneath a violet sky.'
		],
		colors: ['#1e1427', '#483157', '#83559e']
	},
	{
		id: 'night',
		name: ['Cherry Night', 'Cherry Night'],
		description: ['Cerezos iluminados bajo la luna.', 'Cherry blossoms lit beneath the moon.'],
		colors: ['#16080d', '#3d1c1f', '#7f4237']
	},
	{
		id: 'cave',
		name: ['Emerald Grotto', 'Emerald Grotto'],
		description: ['Un bosque esmeralda bajo tierra.', 'An emerald forest deep underground.'],
		colors: ['#09130f', '#24432f', '#5db28d']
	},
	{
		id: 'aurora',
		name: ['Aurora Drift', 'Aurora Drift'],
		description: [
			'Auroras sobre un lago helado entre montañas.',
			'Auroras above an icy mountain lake.'
		],
		colors: ['#17132e', '#4c3871', '#5f478e']
	},
	{
		id: 'pixel',
		name: ['Blocky Paradise', 'Blocky Paradise'],
		description: ['Un valle de bloques con río y nubes.', 'A blocky valley with river and clouds.'],
		colors: ['#143023', '#3b886f', '#4aaa8b']
	}
];
export function sceneInfo(id: SceneId) {
	return SCENES.find((s) => s.id === id)!;
}
