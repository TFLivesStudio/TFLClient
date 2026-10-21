export type VisualStyle = 'modern' | 'pixel';
export type Material = 'matte' | 'soft-glass' | 'glass';
export type Quality = 'lite' | 'balanced' | 'experience';
export interface CapabilityInput {
	quality: Quality;
	reducedMotion: boolean;
	systemReducedMotion: boolean;
	noBlur: boolean;
	noInfiniteFx: boolean;
	active: boolean;
}
/** Every visual subsystem consumes this policy; business operations never wait for motion. */
export function resolveCapabilities(input: CapabilityInput) {
	const lite = input.quality === 'lite';
	const motion = !lite && !input.reducedMotion && !input.systemReducedMotion && input.active;
	const cinematic = motion && input.quality === 'experience';
	return {
		motion,
		cinematic,
		blur: !lite && !input.noBlur,
		ambient: cinematic && !input.noInfiniteFx,
		parallax: cinematic,
		viewer: !lite,
		animateViewer: motion && !input.noInfiniteFx,
		dpr: lite ? 1 : input.quality === 'balanced' ? 1.25 : 1.5,
		duration: motion ? (cinematic ? 420 : 160) : 0,
		stagger: cinematic ? 45 : 0
	};
}
export interface InstanceVisual {
	favorite: boolean;
	group: string;
	artwork: string;
}
export interface VisualPreferences {
	version: 1;
	style: VisualStyle;
	pixelType: 'readable' | 'classic';
	material: Material;
	adaptive: boolean;
	customAccent: string;
	libraryView: 'list' | 'grid';
	home: { recent: boolean; servers: boolean; activity: boolean; system: boolean };
	groups: string[];
	instances: Record<string, InstanceVisual>;
}
export const DEFAULT_VISUAL: VisualPreferences = {
	version: 1,
	style: 'modern',
	pixelType: 'readable',
	material: 'soft-glass',
	adaptive: true,
	customAccent: '',
	libraryView: 'list',
	home: { recent: true, servers: true, activity: true, system: true },
	groups: [],
	instances: {}
};
export const ARTWORKS = [
	'auto',
	'none',
	'mc-1',
	'mc-2',
	'mc-3',
	'mc-4',
	'mc-5',
	'mc-6',
	'mc-7',
	'mc-8',
	'mc-9',
	'mc-10',
	'mc-11',
	'mc-12',
	'mc-13'
] as const;
const record = (v: unknown): Record<string, unknown> =>
	v && typeof v === 'object' && !Array.isArray(v) ? (v as Record<string, unknown>) : {};
/** Whitelist persisted data. Never turn arbitrary stored strings into paths, URLs or CSS. */
export function parsePreferences(raw: unknown): VisualPreferences {
	const v = record(raw),
		home = record(v.home);
	const groups = Array.isArray(v.groups)
		? [
				...new Set(
					v.groups
						.filter((x): x is string => typeof x === 'string' && x.trim().length > 0)
						.map((x) => x.trim().slice(0, 40))
				)
			].slice(0, 30)
		: [];
	const instances: Record<string, InstanceVisual> = {};
	for (const [id, value] of Object.entries(record(v.instances)).slice(0, 2000)) {
		if (['__proto__', 'constructor', 'prototype'].includes(id)) continue;
		const meta = record(value);
		instances[id] = {
			favorite: meta.favorite === true,
			group: typeof meta.group === 'string' && groups.includes(meta.group) ? meta.group : '',
			artwork: ARTWORKS.includes(meta.artwork as (typeof ARTWORKS)[number])
				? String(meta.artwork)
				: 'auto'
		};
	}
	return {
		version: 1,
		style: v.style === 'pixel' ? 'pixel' : 'modern',
		pixelType: v.pixelType === 'classic' ? 'classic' : 'readable',
		material: v.material === 'matte' || v.material === 'glass' ? v.material : 'soft-glass',
		adaptive: v.adaptive !== false,
		customAccent:
			typeof v.customAccent === 'string' && /^#[0-9a-f]{6}$/i.test(v.customAccent)
				? v.customAccent
				: '',
		libraryView: v.libraryView === 'grid' ? 'grid' : 'list',
		home: {
			recent: home.recent !== false,
			servers: home.servers !== false,
			activity: home.activity !== false,
			system: home.system !== false
		},
		groups,
		instances
	};
}
export function artworkUrl(id: string): string | null {
	return /^mc-(?:[1-9]|1[0-3])$/.test(id) ? `/wallpapers/mc-wallpaper-${id.slice(3)}.jpg` : null;
}

/** Choose the stronger WCAG contrast for text on a user-selected accent. */
export function accentText(hex: string): string {
	const rgb = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
	const linear = rgb.map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
	const luminance = linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
	return (luminance + 0.05) / 0.05 >= 1.05 / (luminance + 0.05) ? '#000000' : '#ffffff';
}
