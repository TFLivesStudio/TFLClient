<script lang="ts">
	import { invoke } from '$lib/api/invoke';
	import type { SceneConfig } from '$lib/scenes/model';
	import { resolveAddonScene } from './state.svelte';
	import { advanceAddonLayer } from './model';
	let {
		config,
		fps = 0,
		depth = 0
	}: { config: SceneConfig; fps?: number; depth?: number } = $props();
	let canvas = $state<HTMLCanvasElement>();
	let failed = $state(false);
	let images = $state.raw<HTMLImageElement[]>([]);
	let offsets: number[] = [];
	const selected = $derived(resolveAddonScene(config.preset));
	$effect(() => {
		const found = selected;
		images = [];
		offsets = [];
		failed = false;
		if (!found) return;
		let stopped = false;
		void invoke<Record<string, string>>('addons_scene_resources', {
			id: found.addon.manifest.id,
			sceneId: found.scene.id
		})
			.then((resources) =>
				Promise.all(
					found.scene.layers.map(async (layer) => {
						const source = resources[layer.path];
						if (!/^data:image\/png;base64,[A-Za-z0-9+/=]+$/.test(source))
							throw new Error('Invalid addon image');
						const image = new Image();
						image.src = source;
						await image.decode();
						return image;
					})
				)
			)
			.then((loaded) => {
				if (stopped) {
					for (const image of loaded) image.src = '';
					return;
				}
				offsets = loaded.map(() => 0);
				images = loaded;
			})
			.catch(() => {
				if (!stopped) failed = true;
			});
		return () => {
			stopped = true;
		};
	});
	$effect(() => {
		const target = canvas;
		const found = selected;
		const loaded = images;
		const rate = fps;
		const intensity = depth;
		if (!target) return;
		const context = target.getContext('2d', { alpha: false });
		if (!context) return;
		if (!found) {
			context.fillStyle = '#111827';
			context.fillRect(0, 0, 384, 216);
			return;
		}
		let stopped = false;
		let frame = 0;
		let previous = 0;
		let elapsed = 0;
		const patterns = loaded.map((image) => context.createPattern(image, 'repeat-x'));
		function paint(delta: number) {
			if (!context) return;
			context.fillStyle = found!.scene.background;
			context.fillRect(0, 0, 384, 216);
			context.imageSmoothingEnabled = false;
			for (const [index, image] of loaded.entries()) {
				const layer = found!.scene.layers[index];
				const scale = 216 / image.naturalHeight;
				const period = image.naturalWidth * scale;
				// Per-layer bounded phase. Positive x is right; paused frames retain their phase.
				offsets[index] = advanceAddonLayer(offsets[index], delta, layer.speed, intensity, period);
				const pattern = patterns[index];
				if (!pattern) continue;
				pattern.setTransform(new DOMMatrix().translate(Math.floor(offsets[index]), 0).scale(scale));
				context.globalAlpha = layer.opacity;
				context.fillStyle = pattern;
				context.fillRect(0, 0, 384, 216);
			}
			context.globalAlpha = 1;
		}
		function animate(now: number) {
			if (stopped) return;
			if (!previous) previous = now;
			const delta = Math.min(0.1, (now - previous) / 1000);
			previous = now;
			elapsed += delta;
			if (elapsed >= 1 / rate) {
				paint(elapsed);
				elapsed %= 1 / rate;
			}
			frame = requestAnimationFrame(animate);
		}
		paint(0);
		if (rate > 0 && loaded.length) frame = requestAnimationFrame(animate);
		return () => {
			stopped = true;
			cancelAnimationFrame(frame);
		};
	});
</script>

<div
	class="addon-scene"
	style:background={selected?.scene.background ?? '#111827'}
	data-addon-scene={config.preset}
	data-addon-fallback={failed || !selected}
>
	<canvas bind:this={canvas} width="384" height="216" aria-hidden="true"></canvas>
</div>

<style>
	.addon-scene {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
	}
	canvas {
		width: 100%;
		height: 100%;
		object-fit: cover;
		image-rendering: pixelated;
	}
</style>
