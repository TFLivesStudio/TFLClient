<script lang="ts">
	import { cinematicPoster } from './cinematic';
	import { artworkUrl } from '$lib/visual/policy';
	import { dayPhase, type SceneConfig } from './model';
	import { sceneVectorLayers } from './layeredArt';
	import { LANDSCAPE_PERIOD } from './landscape';
	import { createLayerMotion } from './motion';
	import { subscribeFrame } from './frameClock';
	let {
		config,
		phase = 'day',
		fps = 0,
		depth = 0
	}: {
		config: SceneConfig;
		phase?: ReturnType<typeof dayPhase>;
		fps?: number;
		depth?: number;
	} = $props();
	let container = $state<HTMLDivElement>();
	let scale = $state(1);
	let imageFailed = $state(false);
	$effect(() => {
		void config.preset;
		void config.poster;
		imageFailed = false;
	});
	const layers = $derived(sceneVectorLayers(config.preset));
	const layered = $derived(
		(config.poster === 'auto' || (config.poster === 'cinematic' && imageFailed)) &&
			layers.length > 0
	);
	$effect(() => {
		const node = container;
		if (!node) return;
		const resize = () => {
			scale = Math.max(1, Math.ceil(node.clientWidth / 384), Math.ceil(node.clientHeight / 216));
		};
		resize();
		const observer = new ResizeObserver(resize);
		observer.observe(node);
		return () => observer.disconnect();
	});
	let motions: ReturnType<typeof createLayerMotion>[] = [];
	let motionId = '';
	$effect(() => {
		const node = container,
			rate = fps,
			current = layers,
			id = config.preset;
		if (!node || !layered || rate <= 0 || config.poster !== 'auto') return;
		const surfaces = [...node.querySelectorAll<HTMLElement>('.travel-layer')];
		if (motionId !== id) {
			motions = current.map((layer) => createLayerMotion(LANDSCAPE_PERIOD, layer.depth, true));
			motionId = id;
		}
		return subscribeFrame((delta) => {
			for (const [i, surface] of surfaces.entries())
				surface.style.backgroundPositionX = `${motions[i].advance(delta, depth) * scale}px`;
		}, rate);
	});
</script>

<div
	bind:this={container}
	class="scene-frame"
	class:night={config.adaptTime && phase === 'night'}
	class:sunset={config.adaptTime && phase === 'sunset'}
	data-scene={config.preset}
	data-layered={layered}
>
	{#if artworkUrl(config.poster)}
		<img src={artworkUrl(config.poster)!} alt="" draggable="false" />
	{:else if config.poster === 'custom' || config.preset === 'legacy'}
		<div class="legacy"></div>
	{:else if config.poster === 'cinematic' && cinematicPoster(config.preset) && !imageFailed}
		<img
			src={cinematicPoster(config.preset)!}
			alt=""
			draggable="false"
			onerror={() => (imageFailed = true)}
		/>
	{:else if layered}
		<img src={`/scenes/${config.preset}-sky.svg`} alt="" draggable="false" />
		{#each layers as layer, i (i)}<div
				class="travel-layer"
				style:background-image={`url("${layer.url}")`}
				style:background-size={`${LANDSCAPE_PERIOD * scale}px ${216 * scale}px`}
			></div>{/each}
	{/if}
	{#if config.lighting}<div class="light"></div>{/if}
</div>

<style>
	.scene-frame,
	img,
	.legacy,
	.light,
	.travel-layer {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
	}
	.scene-frame {
		overflow: hidden;
		background: #102029;
	}
	img {
		object-fit: cover;
	}
	.travel-layer {
		background-repeat: repeat-x;
		background-position: 0 center;
		image-rendering: pixelated;
	}
	.legacy {
		background: var(--wallpaper-bg, linear-gradient(135deg, #152130, #253545));
	}
	.light {
		background: linear-gradient(0deg, #08101a24, transparent 60%);
	}
	.night .light {
		background: #10214514;
	}
	.sunset .light {
		background: linear-gradient(140deg, #e7805714, transparent, #45255c12);
	}
</style>
