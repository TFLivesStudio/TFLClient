import { landscapeLayers, LANDSCAPE_PERIOD, type LandscapeId } from './landscape';
import type { SceneId } from './model';
const palettes: Record<string, [LandscapeId, string, string, string]> = {
	forest: ['midnight-forest', '#183336', '#496d60', '#132a28'],
	embers: ['dithered-sunset', '#381a24', '#ad593b', '#281b25'],
	void: ['aurora-valley', '#201a37', '#78638f', '#20182f'],
	night: ['midnight-forest', '#15243c', '#596d84', '#142436'],
	cave: ['midnight-forest', '#142c32', '#4d7374', '#14292b'],
	aurora: ['aurora-valley', '#13373c', '#71a49b', '#16323a'],
	pixel: ['dithered-sunset', '#372e46', '#ad786f', '#302b3f']
};
const dataSvg = (body: string, width = LANDSCAPE_PERIOD) =>
	`data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} 216" shape-rendering="crispEdges">${body}</svg>`)}`;
function tone(far: string, near: string, depth: number) {
	return (
		'#' +
		[1, 3, 5]
			.map((i) =>
				Math.round(
					parseInt(far.slice(i, i + 2), 16) * (1 - depth) +
						parseInt(near.slice(i, i + 2), 16) * depth
				)
					.toString(16)
					.padStart(2, '0')
			)
			.join('')
	);
}
/** Original, host-owned periodic geometry replaces flat cinematic art during normal playback. */
export function sceneVectorLayers(id: SceneId) {
	const palette = palettes[id];
	if (!palette) return [];
	return landscapeLayers(palette[0]).map((layer) => ({
		depth: layer.depth,
		url: dataSvg(
			layer.shapes
				.map((s) => {
					const fill = tone(palette[2], palette[3], layer.depth);
					return s.rect
						? `<rect x="${s.rect[0]}" y="${s.rect[1]}" width="${s.rect[2]}" height="${s.rect[3]}" fill="${fill}"/>`
						: `<polygon points="${s.points!.map((p) => p.join(',')).join(' ')}" fill="${fill}"/>`;
				})
				.join('')
		)
	}));
}
export function sceneVectorPoster(id: SceneId) {
	const palette = palettes[id];
	if (!palette) return '';
	return dataSvg(
		`<rect width="384" height="216" fill="${palette[1]}"/><path fill="${palette[2]}" d="M0 118L54 83 113 107 178 66 249 111 309 88 384 118V216H0z"/><path fill="${palette[3]}" d="M0 163L41 123 75 166 137 142 206 175 278 119 336 156 384 142V216H0z"/>`,
		384
	);
}
