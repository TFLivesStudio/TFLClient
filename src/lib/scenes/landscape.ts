/** Original procedural art. No reference sprites, textures or upstream source are redistributed. */
export const LANDSCAPE_IDS = ['midnight-forest', 'dithered-sunset', 'aurora-valley'] as const;
export type LandscapeId = (typeof LANDSCAPE_IDS)[number];
export const LANDSCAPE_WIDTH = 384;
export const LANDSCAPE_HEIGHT = 216;
export const BAYER_4 = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5] as const;
export function isLandscape(id: string): id is LandscapeId {
	return LANDSCAPE_IDS.includes(id as LandscapeId);
}
export const LANDSCAPE_PALETTES: Record<
	LandscapeId,
	{
		sky: string[];
		mountains: string[];
		trees: string[];
		light: string;
		cloud: string;
		water: string;
	}
> = {
	'midnight-forest': {
		sky: ['#10162e', '#1b2246', '#303259', '#494369', '#6b577c'],
		mountains: ['#434663', '#303c54', '#243346'],
		trees: ['#233949', '#172d38', '#0d212b'],
		light: '#e7f3d3',
		cloud: '#747797',
		water: '#253d52'
	},
	'dithered-sunset': {
		sky: ['#393457', '#6b4268', '#ae5b73', '#e58b78', '#ffd09a'],
		mountains: ['#9f6877', '#79566c', '#51465d'],
		trees: ['#4a4055', '#352f46', '#242739'],
		light: '#ffe1a1',
		cloud: '#f1ab99',
		water: '#a0707e'
	},
	'aurora-valley': {
		sky: ['#0a162c', '#102c43', '#20485b', '#386775', '#749390'],
		mountains: ['#53778c', '#34596f', '#213e55'],
		trees: ['#244350', '#17323e', '#102733'],
		light: '#c3ead9',
		cloud: '#699fa0',
		water: '#142f43'
	}
};
export interface LandscapeShape {
	fill: string;
	points?: number[][];
	rect?: number[];
	opacity?: number;
}
export interface LandscapeLayer {
	depth: number;
	shapes: LandscapeShape[];
}
export function noise(seed: number) {
	const n = Math.sin(seed * 127.1 + 311.7) * 43758.5453;
	return n - Math.floor(n);
}
/** Palette selection depends only on logical coordinates: camera movement cannot shimmer the sky. */
export function skyColor(id: LandscapeId, x: number, y: number, dither = true) {
	const colors = LANDSCAPE_PALETTES[id].sky;
	const value = Math.max(0, Math.min(colors.length - 1, (y / 155) * (colors.length - 1)));
	const band = Math.floor(value);
	const threshold = (BAYER_4[((y & 3) << 2) + (x & 3)] + 0.5) / 16;
	return colors[Math.min(colors.length - 1, band + (dither && value - band > threshold ? 1 : 0))];
}
export function landscapeLayers(id: LandscapeId): LandscapeLayer[] {
	const p = LANDSCAPE_PALETTES[id];
	const layers: LandscapeLayer[] = [];
	for (let layer = 0; layer < 3; layer++) {
		const shapes: LandscapeShape[] = [];
		const baseline = 110 + layer * 18;
		const points = [
			[-24, 216],
			[-24, baseline]
		];
		for (let x = -24; x <= 408; x += 3) {
			const ridge =
				Math.sin(x * (0.022 + layer * 0.004) + layer * 2.3) * (20 - layer * 4) +
				Math.sin(x * 0.061 + layer) * 9;
			points.push([x, Math.round(baseline - 12 + ridge)]);
		}
		points.push([408, 216]);
		shapes.push({ fill: p.mountains[layer], points });
		if (id === 'aurora-valley' && layer < 2) {
			for (let x = -24; x < 408; x += 3) {
				const pt = points[(x + 24) / 3 + 2];
				if (pt[1] < baseline - 18)
					shapes.push({
						fill: layer ? '#91b2bb' : '#bfd0d0',
						rect: [x, pt[1], 3, 3 + Math.round(noise(x + layer) * 4)]
					});
			}
		}
		layers.push({ depth: 0.15 + layer * 0.17, shapes });
	}
	if (id === 'aurora-valley')
		layers.push({ depth: 0.4, shapes: [{ fill: p.water, rect: [-24, 147, 432, 69] }] });
	for (let layer = 0; layer < 3; layer++) {
		const shapes: LandscapeShape[] = [];
		const floor = 149 + layer * 19;
		for (let i = 0; i < 22; i++) {
			const x = Math.round(i * 22 - 29 + noise(i + layer * 20) * 9);
			// Open center of Aurora's reflective lake and the sunset valley.
			if (id !== 'midnight-forest' && x > 100 - layer * 20 && x < 286 + layer * 20) continue;
			const h = Math.round(16 + layer * 10 + noise(i + 53 + layer) * 30);
			const w = Math.round(h * 0.24);
			shapes.push({ fill: p.trees[layer], rect: [x - 1, floor - h, 3, h + 10] });
			for (let tier = 0; tier < 4; tier++) {
				const top = floor - h + tier * Math.round(h * 0.16);
				const spread = Math.round(w * (0.5 + tier * 0.3));
				shapes.push({
					fill: p.trees[layer],
					points: [
						[x, top],
						[x - spread, top + Math.round(h * 0.36)],
						[x + spread, top + Math.round(h * 0.36)]
					]
				});
			}
		}
		if (layer === 2)
			shapes.push({
				fill: p.trees[2],
				points: [
					[-24, 207],
					[48, 201],
					[114, 214],
					[198, 211],
					[272, 206],
					[343, 201],
					[408, 205],
					[408, 216],
					[-24, 216]
				]
			});
		layers.push({ depth: 0.5 + layer * 0.25, shapes });
	}
	return layers;
}
function shapeSvg(s: LandscapeShape) {
	const attr = `fill="${s.fill}"${s.opacity === undefined ? '' : ` opacity="${s.opacity}"`}`;
	return s.rect
		? `<rect x="${s.rect[0]}" y="${s.rect[1]}" width="${s.rect[2]}" height="${s.rect[3]}" ${attr}/>`
		: `<polygon points="${s.points!.map((p) => p.join(',')).join(' ')}" ${attr}/>`;
}
/** Same original geometry as Canvas; Lite and no-Canvas fallbacks allocate no drawing context. */
export function landscapePoster(
	id: string,
	dither = true,
	options: {
		weather?: string;
		lighting?: boolean;
		adaptTime?: boolean;
		phase?: string;
		particles?: boolean;
	} = {}
): string {
	if (!isLandscape(id)) return '';
	const p = LANDSCAPE_PALETTES[id];
	let svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 384 216" shape-rendering="crispEdges"><defs>`;
	for (let i = 0; i < 16; i++) {
		svg += `<pattern id="b${i}" width="4" height="4" patternUnits="userSpaceOnUse">`;
		for (let y = 0; y < 4; y++)
			for (let x = 0; x < 4; x++)
				if (BAYER_4[y * 4 + x] < i)
					svg += `<rect x="${x}" y="${y}" width="1" height="1" fill="white"/>`;
		svg += '</pattern>';
		svg += `<mask id="m${i}"><rect width="384" height="216" fill="url(#b${i})"/></mask>`;
	}
	svg += '</defs>';
	for (let y = 0; y < 216; y++) {
		const v = Math.min(4, (y / 155) * 4),
			band = Math.floor(v);
		svg += `<rect y="${y}" width="384" height="1" fill="${p.sky[band]}"/>`;
		if (dither && band < 4)
			svg += `<rect y="${y}" width="384" height="1" fill="${p.sky[band + 1]}" mask="url(#m${Math.floor((v - band) * 16)})"/>`;
	}
	if (id !== 'dithered-sunset')
		for (let i = 0; i < 68; i++)
			svg += `<rect x="${Math.floor(noise(i + 1) * 384)}" y="${Math.floor(noise(i + 200) * 100)}" width="1" height="1" fill="${p.light}" opacity=".65"/>`;
	svg += `<path d="M274 33h16v3h6v7h3v16h-3v7h-6v3h-16v-3h-6v-7h-3V43h3v-7h6z" transform="translate(0 21)" fill="${p.light}"/>`;
	if (id === 'aurora-valley')
		svg +=
			'<path d="M20 44q70-45 140 1t145-10l-3 21q-65 30-145-1T20 67z" transform="translate(0 32)" fill="#61d7b1" opacity=".25"/>';
	for (let i = 0; i < 7; i++) {
		const x = ((i * 79) % 504) - 60,
			y = 28 + ((i * 17) % 66);
		svg += `<path d="M${x + 8} ${y}h38v2h14v3h-60v-3h8z" fill="${p.cloud}" opacity=".2"/>`;
	}
	for (const layer of landscapeLayers(id)) svg += layer.shapes.map(shapeSvg).join('');
	svg += `<path d="M0 148h384v3H0zM0 173h384v2H0z" fill="${p.cloud}" opacity=".14"/>`;
	if (options.weather === 'mist')
		svg += `<rect y="130" width="384" height="50" fill="${p.cloud}" opacity=".13"/>`;
	if (options.weather === 'rain' || options.weather === 'snow')
		for (let i = 0; i < 55; i++)
			svg += `<rect x="${Math.floor(noise(i + 501) * 384)}" y="${Math.floor(noise(i + 601) * 216)}" width="1" height="${options.weather === 'rain' ? 4 : 1}" fill="${p.light}" opacity=".35"/>`;
	if (options.particles && id === 'midnight-forest')
		for (let i = 0; i < 18; i++)
			svg += `<rect x="${Math.floor(noise(i + 80) * 384)}" y="${Math.floor(123 + noise(i + 83) * 70)}" width="1" height="1" fill="#e2ed9c" opacity=".6"/>`;
	if (options.lighting)
		svg += `<rect width="384" height="216" fill="${options.adaptTime && options.phase === 'day' ? '#f2e9bb' : options.adaptTime && options.phase === 'sunset' ? '#e7a478' : '#11162d'}" opacity=".09"/>`;
	svg += '</svg>';
	return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}
