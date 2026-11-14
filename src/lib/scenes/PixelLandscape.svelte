<script lang="ts">
	import { untrack } from 'svelte';
	import { perfEnabled, perfRecord } from '$lib/perf';
	import { subscribeFrame } from './frameClock';
	import { isLandscape, landscapePoster } from './landscape';
	import { createLandscapeRenderer } from './landscapeRenderer';
	import type { SceneConfig } from './model';
	let {
		config,
		fps = 0,
		depth = 0,
		x = 0,
		y = 0,
		phase = 'night'
	}: {
		config: SceneConfig;
		fps?: number;
		depth?: number;
		x?: number;
		y?: number;
		phase?: string;
	} = $props();
	let canvas = $state<HTMLCanvasElement>();
	let container = $state<HTMLDivElement>();
	let scale = $state(1);
	let left = $state(0),
		top = $state(0);
	let failed = $state(false);
	let time = 0;
	const poster = $derived(
		landscapePoster(config.preset, config.dithering, {
			weather: config.weather,
			lighting: config.lighting,
			adaptTime: config.adaptTime,
			phase,
			particles: config.particles
		})
	);
	$effect(() => {
		const node = container;
		if (!node) return;
		const resize = () => {
			scale = Math.max(1, Math.ceil(node.clientWidth / 384), Math.ceil(node.clientHeight / 216));
			left = Math.floor((node.clientWidth - 384 * scale) / 2);
			top = Math.floor((node.clientHeight - 216 * scale) / 2);
		};
		resize();
		const observer = new ResizeObserver(resize);
		observer.observe(node);
		return () => observer.disconnect();
	});
	$effect(() => {
		const node = canvas,
			id = config.preset,
			dither = config.dithering,
			rate = fps;
		if (!node || !isLandscape(id) || rate <= 0) return;
		failed = false;
		let renderer: ReturnType<typeof createLandscapeRenderer> = null;
		let stop = () => {};
		let frameMs = 0,
			frameCount = 0;
		const start = () => {
			stop();
			renderer?.dispose();
			const started = perfEnabled ? performance.now() : 0;
			renderer = createLandscapeRenderer(node, id, dither);
			if (perfEnabled) perfRecord(`landscape.cache.${id}`, performance.now() - started);
			failed = !renderer;
			if (!renderer) return;
			renderer.draw(time, x, y, depth, config, phase);
			stop = subscribeFrame((delta) => {
				time += delta;
				const started = perfEnabled ? performance.now() : 0;
				renderer?.draw(time, x, y, depth, config, phase);
				if (perfEnabled) {
					frameMs += performance.now() - started;
					frameCount++;
				}
			}, rate);
		};
		untrack(start);
		const lost = (event: Event) => {
			event.preventDefault();
			stop();
			failed = true;
		};
		node.addEventListener('contextlost', lost);
		node.addEventListener('contextrestored', start);
		return () => {
			stop();
			if (perfEnabled && frameCount) perfRecord(`landscape.frame.${id}`, frameMs / frameCount);
			node.removeEventListener('contextlost', lost);
			node.removeEventListener('contextrestored', start);
			renderer?.dispose();
		};
	});
</script>

<div
	bind:this={container}
	class="pixel-landscape"
	style={`--landscape-width:${384 * scale}px;--landscape-height:${216 * scale}px;--landscape-left:${left}px;--landscape-top:${top}px`}
	data-landscape={config.preset}
	data-animated={fps > 0 && !failed}
	aria-hidden="true"
>
	<img src={poster} alt="" draggable="false" />
	{#if fps > 0}<canvas bind:this={canvas} class:failed></canvas>{/if}
</div>

<style>
	.pixel-landscape,
	img,
	canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
	}
	.pixel-landscape {
		overflow: hidden;
		background: #10162e;
	}
	img,
	canvas {
		inset: var(--landscape-top) auto auto var(--landscape-left);
		width: var(--landscape-width);
		height: var(--landscape-height);
		image-rendering: pixelated;
	}
	.failed {
		display: none;
	}
</style>
