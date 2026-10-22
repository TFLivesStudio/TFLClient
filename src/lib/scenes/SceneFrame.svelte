<script lang="ts">
	import { cinematicPoster } from './cinematic';
	import { artworkUrl } from '$lib/visual/policy';
	import { dayPhase, type SceneConfig } from './model';
	let {
		config,
		x = 0,
		y = 0,
		phase = 'day'
	}: {
		config: SceneConfig;
		x?: number;
		y?: number;
		phase?: ReturnType<typeof dayPhase>;
	} = $props();
	let imageFailed = $state(false);
	$effect(() => {
		void config.preset;
		imageFailed = false;
	});
	const offset = $derived(config.motion === 'full' ? 12 : 5);
</script>

<div
	class="scene-frame"
	class:pixel={config.preset === 'pixel'}
	class:night={config.adaptTime && phase === 'night'}
	class:sunset={config.adaptTime && phase === 'sunset'}
	data-scene={config.preset}
>
	{#if artworkUrl(config.poster)}<img
			src={artworkUrl(config.poster)!}
			alt=""
			class="sky"
			draggable="false"
		/>{:else if config.poster === 'custom'}<div
			class="legacy"
		></div>{:else if cinematicPoster(config.preset) && !imageFailed}<img
			src={cinematicPoster(config.preset)!}
			alt=""
			class="sky cinematic-art"
			draggable="false"
			onerror={() => (imageFailed = true)}
			style:transform={`translate3d(${x * offset * 0.25}px,${y * offset * 0.2}px,0)`}
		/>
	{:else if config.preset !== 'legacy'}
		<img src={`/scenes/${config.preset}-sky.svg`} alt="" class="sky" draggable="false" />
		<img
			src={`/scenes/${config.preset}-far.svg`}
			alt=""
			class="far"
			draggable="false"
			style:transform={`translate3d(${x * offset * 0.4}px,${y * offset * 0.3}px,0)`}
		/>
		<img
			src={`/scenes/${config.preset}-near.svg`}
			alt=""
			class="near"
			draggable="false"
			style:transform={`translate3d(${x * offset}px,${y * offset * 0.5}px,0)`}
		/>
	{:else}<div class="legacy"></div>{/if}
	{#if config.lighting}<div class="light"></div>{/if}
</div>

<style>
	.scene-frame,
	.scene-frame img,
	.legacy,
	.light {
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
	.scene-frame img {
		object-fit: cover;
		scale: 1.04;
		transition: transform 0.4s ease-out;
	}
	.cinematic-art {
		object-position: 64% center;
	}
	.legacy {
		background: var(--wallpaper-bg, linear-gradient(135deg, #152130, #253545));
	}
	.light {
		background: linear-gradient(0deg, #08101a4d, transparent 60%);
		mix-blend-mode: multiply;
	}
	.night .light {
		background: #10214566;
	}
	.sunset .light {
		background: linear-gradient(140deg, #e7805744, transparent, #45255c22);
	}
	.pixel img {
		image-rendering: pixelated;
	}
	@media (prefers-reduced-motion: reduce) {
		.scene-frame img {
			transition: none;
		}
	}
</style>
