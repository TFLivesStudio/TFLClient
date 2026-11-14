/** Approved editorial art only. Entries are populated from the owner's inspected archive. */
import { APPROVED_BANNERS } from './banner-registry.ts';
export interface BannerArtwork {
	id: string;
	exact?: readonly string[];
	family?: string;
	url: string;
	thumbnail?: string;
	alternative?: string;
	position: string;
	available: boolean;
}
export const BANNERS: readonly BannerArtwork[] = APPROVED_BANNERS;
export function minecraftFamily(version: string): string | null {
	if (/^1\.\d+(?:\.\d+)?$/.test(version)) return version.split('.').slice(0, 2).join('.');
	if (/^(?:2[6-9]|[3-9]\d)\.\d+(?:\.\d+)?$/.test(version))
		return version.split('.').slice(0, 2).join('.');
	return null;
}
export function approvedBanner(id: string) {
	return BANNERS.find((b) => b.id === id && b.available);
}
export function resolveBanner(version: string, artwork = 'auto') {
	if (artwork === 'none')
		return {
			id: 'none',
			url: null,
			thumbnail: null,
			position: 'center',
			source: 'custom' as const
		};
	if (/^mc-(?:[1-9]|1[0-3])$/.test(artwork))
		return {
			id: artwork,
			url: `/wallpapers/mc-wallpaper-${artwork.slice(3)}.jpg`,
			thumbnail: `/wallpapers/mc-wallpaper-${artwork.slice(3)}.jpg`,
			position: 'center 42%',
			source: 'custom' as const
		};
	const custom = approvedBanner(artwork);
	const exact = BANNERS.find((b) => b.available && b.exact?.includes(version));
	const family = minecraftFamily(version);
	const related = family ? BANNERS.find((b) => b.available && b.family === family) : undefined;
	const selected = custom ?? exact ?? related;
	if (selected)
		return {
			...selected,
			thumbnail: selected.thumbnail ?? selected.url,
			source: custom ? ('custom' as const) : exact ? ('exact' as const) : ('family' as const)
		};
	// Existing bundled art is a fallback, never advertised as a specific Minecraft release.
	return {
		id: 'bundled-fallback',
		url: '/wallpapers/mc-wallpaper-6.jpg',
		thumbnail: '/wallpapers/mc-wallpaper-6.jpg',
		position: 'center 42%',
		source: 'fallback' as const
	};
}
