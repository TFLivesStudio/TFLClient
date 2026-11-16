export const THEME_TOKENS = [
	'--bg-main',
	'--bg-sidebar',
	'--bg-card',
	'--bg-input',
	'--bg-item-active',
	'--text-primary',
	'--text-secondary',
	'--text-muted',
	'--border',
	'--accent',
	'--accent-text'
] as const;
export interface AddonTheme {
	id: string;
	name: string;
	tokens: Record<string, string>;
}
export interface AddonLayer {
	path: string;
	speed: number;
	opacity: number;
}
export interface AddonSceneDefinition {
	id: string;
	name: string;
	background: string;
	layers: AddonLayer[];
}
export interface AddonManifest {
	schemaVersion: 1;
	id: string;
	name: string;
	author: string;
	version: string;
	description: string;
	apiVersion: string;
	launcherVersion: string;
	permissions: string[];
	themes: AddonTheme[];
	scenes: AddonSceneDefinition[];
}
export interface InstalledAddon {
	manifest: AddonManifest;
	digest: string;
	enabled: boolean;
	rollbackAvailable: boolean;
}
export interface AddonCatalog {
	safeMode: boolean;
	addons: InstalledAddon[];
	errors?: string[];
	brokenAddons?: { id: string; error: string; rollbackAvailable: boolean }[];
}
export interface AddonInspection {
	manifest: AddonManifest;
	digest: string;
	installedVersion?: string | null;
}
export function contributionId(packageId: string, id: string) {
	return `addon:${packageId}:${id}` as const;
}
export function advanceAddonLayer(
	offset: number,
	delta: number,
	speed: number,
	depth: number,
	period: number
) {
	if (![offset, delta, speed, depth, period].every(Number.isFinite) || period <= 0) return 0;
	const phase =
		offset +
		Math.min(0.1, Math.max(0, delta)) *
			Math.max(0, speed) *
			(0.4 + Math.max(0, Math.min(1, depth)));
	return ((phase % period) + period) % period;
}
export function validThemeTokens(tokens: Record<string, string>) {
	return Object.entries(tokens).every(
		([key, value]) =>
			(THEME_TOKENS as readonly string[]).includes(key) && /^#[0-9a-f]{6}$/i.test(value)
	);
}
function colorChannels(value: string): number[] | null {
	const hex = /^#([0-9a-f]{6})$/i.exec(value);
	const rgb = /^rgba?\(\s*(\d+)[, ]+\s*(\d+)[, ]+\s*(\d+)(?:\s*[,/]\s*([\d.]+))?\s*\)$/.exec(value);
	const channels = hex
		? [...[0, 2, 4].map((i) => parseInt(hex[1].slice(i, i + 2), 16)), 1]
		: rgb
			? [...rgb.slice(1, 4).map(Number), rgb[4] === undefined ? 1 : Number(rgb[4])]
			: null;
	return !channels ||
		channels.slice(0, 3).some((c) => c < 0 || c > 255) ||
		channels[3] < 0 ||
		channels[3] > 1
		? null
		: channels;
}
function luminance(channels: number[]) {
	return channels
		.map((c) => {
			const n = c / 255;
			return n <= 0.04045 ? n / 12.92 : ((n + 0.055) / 1.055) ** 2.4;
		})
		.reduce((sum, c, i) => sum + c * [0.2126, 0.7152, 0.0722][i], 0);
}
export function contrastRatio(a: string, b: string) {
	const text = colorChannels(a),
		surface = colorChannels(b);
	if (!text || !surface) return 0;
	// Translucent host defaults are checked over both extremes of a changing scene.
	return Math.min(
		...[0, 255].map((backdrop) => {
			const background = surface
				.slice(0, 3)
				.map((c) => c * surface[3] + backdrop * (1 - surface[3]));
			const foreground = text
				.slice(0, 3)
				.map((c, i) => c * text[3] + background[i] * (1 - text[3]));
			const first = luminance(foreground),
				second = luminance(background);
			return (Math.max(first, second) + 0.05) / (Math.min(first, second) + 0.05);
		})
	);
}
export function themeContrastIssues(
	tokens: Record<string, string>,
	defaults: Record<string, string>
) {
	const merged = { ...defaults, ...tokens };
	const issues: string[] = [];
	for (const text of ['--text-primary', '--text-secondary'])
		for (const surface of [
			'--bg-main',
			'--bg-sidebar',
			'--bg-card',
			'--bg-input',
			'--bg-item-active'
		])
			if (contrastRatio(merged[text] ?? '', merged[surface] ?? '') < 4.5)
				issues.push(`${text} / ${surface}`);
	if (contrastRatio(merged['--accent-text'] ?? '', merged['--accent'] ?? '') < 4.5)
		issues.push('--accent-text / --accent');
	return issues;
}
/** Internal semantic token; addons cannot provide arbitrary CSS custom properties. */
export function readableThemeAccent(
	tokens: Record<string, string>,
	defaults: Record<string, string>
) {
	const merged = { ...defaults, ...tokens };
	const accent = merged['--accent'];
	const readable = [
		'--bg-main',
		'--bg-sidebar',
		'--bg-card',
		'--bg-input',
		'--bg-item-active'
	].every((key) => contrastRatio(accent ?? '', merged[key] ?? '') >= 4.5);
	return readable ? accent : merged['--text-primary'];
}
export function enabledAddons(catalog: AddonCatalog) {
	return catalog.safeMode ? [] : catalog.addons.filter((a) => a.enabled);
}
export function findScene(catalog: AddonCatalog, id: string) {
	for (const addon of enabledAddons(catalog)) {
		const scene = addon.manifest.scenes.find((s) => contributionId(addon.manifest.id, s.id) === id);
		if (scene) return { addon, scene };
	}
	return null;
}
export function findTheme(catalog: AddonCatalog, id: string) {
	for (const addon of enabledAddons(catalog)) {
		const theme = addon.manifest.themes.find((t) => contributionId(addon.manifest.id, t.id) === id);
		if (theme && validThemeTokens(theme.tokens)) return { addon, theme };
	}
	return null;
}
