import {
	LANDSCAPE_WIDTH as W,
	LANDSCAPE_HEIGHT as H,
	LANDSCAPE_PALETTES,
	landscapeLayers,
	skyColor,
	noise,
	type LandscapeId,
	type LandscapeShape
} from './landscape';
import type { SceneConfig } from './model';

function drawShape(ctx: CanvasRenderingContext2D, s: LandscapeShape) {
	ctx.fillStyle = s.fill;
	if (s.rect) ctx.fillRect(s.rect[0], s.rect[1], s.rect[2], s.rect[3]);
	else {
		ctx.beginPath();
		s.points!.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
		ctx.closePath();
		ctx.fill();
	}
}
/** Fixed 384x216 caches, independent of DPR/window size. Lifetime belongs to one mounted scene. */
export function createLandscapeRenderer(
	canvas: HTMLCanvasElement,
	id: LandscapeId,
	dither: boolean
) {
	canvas.width = W;
	canvas.height = H;
	const ctx = canvas.getContext('2d', { alpha: false });
	if (!ctx) return null;
	ctx.imageSmoothingEnabled = false;
	const resources: HTMLCanvasElement[] = [];
	function cache() {
		const surface = document.createElement('canvas');
		surface.width = W + 48;
		surface.height = H;
		const context = surface.getContext('2d');
		if (!context) throw new Error('Canvas unavailable');
		context.imageSmoothingEnabled = false;
		resources.push(surface);
		return { surface, context };
	}
	try {
		const sky = cache();
		for (let y = 0; y < H; y++)
			for (let x = 0; x < W; x++) {
				sky.context.fillStyle = skyColor(id, x, y, dither);
				sky.context.fillRect(x, y, 1, 1);
			}
		const p = LANDSCAPE_PALETTES[id];
		if (id !== 'dithered-sunset')
			for (let i = 0; i < 68; i++) {
				sky.context.fillStyle = i % 3 ? '#b7c8d1' : p.light;
				sky.context.fillRect(Math.floor(noise(i + 1) * W), Math.floor(noise(i + 200) * 100), 1, 1);
			}
		// Stepped disc rather than antialiased circle.
		sky.context.fillStyle = p.light;
		for (let y = -18; y <= 18; y++) {
			const half = Math.floor(Math.sqrt(324 - y * y));
			sky.context.fillRect(282 - half, 72 + y, half * 2, 1);
		}
		if (id === 'midnight-forest') {
			sky.context.fillStyle = '#c3d2bd';
			sky.context.fillRect(278, 63, 5, 7);
			sky.context.fillRect(287, 75, 7, 4);
		}
		const layers = landscapeLayers(id).map((layer) => {
			const cached = cache();
			cached.context.translate(24, 0);
			layer.shapes.forEach((shape) => drawShape(cached.context, shape));
			return { ...cached, depth: layer.depth };
		});
		let disposed = false;
		function draw(
			t: number,
			x: number,
			y: number,
			depth: number,
			config: SceneConfig,
			phase: string
		) {
			if (disposed || !ctx) return;
			ctx.globalAlpha = 1;
			ctx.drawImage(sky.surface, 0, 0);
			if (id === 'aurora-valley') {
				for (let ribbon = 0; ribbon < 3; ribbon++) {
					ctx.fillStyle = ['#65e0b3', '#7fcbb7', '#9a93d8'][ribbon];
					for (let i = 0; i < W; i += 2) {
						const top = Math.round(
							63 +
								ribbon * 9 +
								Math.sin(i / 57 + t * 0.14 + ribbon) * 16 +
								Math.sin(i / 29 - t * 0.1) * 5
						);
						for (let j = 0; j < 5; j++) {
							ctx.globalAlpha = (1 - j / 5) * 0.16;
							ctx.fillRect(i, top + j * 3, 2, 3);
						}
					}
				}
			}
			// Slow cloud strips use whole texels; sky dithering remains fixed in screen space.
			for (let i = 0; i < 7; i++) {
				const xx = Math.floor(((i * 79 + t * (0.4 + (i % 3) * 0.3)) % (W + 120)) - 60);
				const yy = 28 + ((i * 17) % 66);
				ctx.globalAlpha = id === 'dithered-sunset' ? 0.35 : 0.15;
				ctx.fillStyle = p.cloud;
				ctx.fillRect(xx + 8, yy, 38, 2);
				ctx.fillRect(xx, yy + 2, 60, 3);
				ctx.fillRect(xx + 4, yy + 5, 47, 1);
			}
			for (const [index, layer] of layers.entries()) {
				ctx.globalAlpha = 1;
				const dx = Math.round(x * depth * layer.depth * 13),
					dy = Math.round(y * depth * layer.depth * 4);
				ctx.drawImage(layer.surface, -24 + dx, dy);
				if (index < 3) {
					ctx.fillStyle = p.cloud;
					ctx.globalAlpha = 0.045;
					ctx.fillRect(0, 127 + index * 16, W, 7);
				}
			}
			if (id === 'aurora-valley') {
				for (let i = 0; i < 34; i++) {
					const yy = 150 + i;
					const xx = Math.round(185 + Math.sin(t * 0.6 + i * 0.9) * 13);
					ctx.globalAlpha = 0.08 + (1 - i / 34) * 0.12;
					ctx.fillStyle = i % 3 ? '#6bd3bb' : p.light;
					ctx.fillRect(xx - i, yy, i * 2 + 9, 1);
				}
			}
			if (id === 'midnight-forest' || config.weather === 'mist')
				for (let i = 0; i < 4; i++) {
					ctx.fillStyle = p.cloud;
					ctx.globalAlpha = 0.08;
					const offset = Math.floor(Math.sin(t * 0.08 + i) * 24);
					ctx.fillRect(offset - 24, 143 + i * 15, W + 48, 3);
				}
			if (config.particles)
				for (let i = 0; i < (id === 'midnight-forest' ? 18 : 9); i++) {
					const xx = Math.round(noise(i + 80) * W + Math.sin(t * 0.34 + i) * 6);
					const yy = Math.round(123 + noise(i + 83) * 70 + Math.sin(t * 0.4 + i * 2) * 4);
					ctx.globalAlpha = Math.max(0, Math.sin(t * 0.8 + i * 2)) * 0.7;
					ctx.fillStyle = id === 'midnight-forest' ? '#e2ed9c' : p.light;
					ctx.fillRect(xx, yy, 1, 1);
					ctx.globalAlpha *= 0.12;
					ctx.fillRect(xx - 2, yy - 2, 5, 5);
				}
			if (config.weather === 'snow' || config.weather === 'rain')
				for (let i = 0; i < 55; i++) {
					ctx.globalAlpha = 0.35;
					ctx.fillStyle = p.light;
					const xx = Math.floor((noise(i + 501) * W + t * (config.weather === 'rain' ? 9 : 2)) % W);
					const yy = Math.floor(
						(noise(i + 601) * H + t * (config.weather === 'rain' ? 70 : 8)) % H
					);
					ctx.fillRect(xx, yy, 1, config.weather === 'rain' ? 4 : 1);
				}
			if (config.lighting) {
				ctx.fillStyle =
					config.adaptTime && phase === 'day'
						? '#f2e9bb'
						: phase === 'sunset' && config.adaptTime
							? '#e7a478'
							: '#11162d';
				ctx.globalAlpha = config.adaptTime && phase === 'day' ? 0.045 : 0.09;
				ctx.fillRect(0, 0, W, H);
			}
			ctx.globalAlpha = 1;
		}
		return {
			draw,
			dispose() {
				disposed = true;
				resources.forEach((surface) => {
					surface.width = 0;
					surface.height = 0;
				});
				canvas.width = 0;
				canvas.height = 0;
			}
		};
	} catch {
		resources.forEach((surface) => {
			surface.width = 0;
			surface.height = 0;
		});
		return null;
	}
}
