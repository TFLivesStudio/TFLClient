<script lang="ts">
	import { onMount } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { invoke } from '$lib/api/invoke';
	import { dayPhase, sceneCapabilities, type SceneConfig } from './model';
	import SceneFrame from './SceneFrame.svelte';
	import ShaderLayer from './ShaderLayer.svelte';
	import { tr } from '$lib/i18n/experience';
	let {
		config,
		quality = 'balanced',
		reduced = false,
		active = false,
		gameRunning = false,
		preview = false
	}: {
		config: SceneConfig;
		quality?: 'lite' | 'balanced' | 'experience';
		reduced?: boolean;
		active?: boolean;
		gameRunning?: boolean;
		preview?: boolean;
	} = $props();
	const cap = $derived(sceneCapabilities({ quality, reduced, active, gameRunning, config }));
	let video = $state<HTMLVideoElement>();
	let failed = $state(false),
		source = $state(''),
		x = $state(0),
		y = $state(0),
		hour = $state(new Date().getHours());
	const phase = $derived(dayPhase(hour));
	type Media = { id: string; path: string; bytes: number; mime: string };
	$effect(() => {
		const id = config.mediaId;
		failed = false;
		source = '';
		let cancelled = false;
		if (id?.startsWith('builtin-')) source = `/scenes/${id.slice(8)}`;
		else if (id)
			invoke<Media[]>('list_scene_media')
				.then((list) => {
					if (!cancelled) {
						const media = list.find((m) => m.id === id);
						if (media) source = convertFileSrc(media.path);
						else failed = true;
					}
				})
				.catch(() => {
					if (!cancelled) failed = true;
				});
		return () => {
			cancelled = true;
		};
	});
	$effect(() => {
		if (video) {
			if (cap.video && !failed)
				void video.play().catch(() => {
					failed = true;
				});
			else video.pause();
		}
	});
	$effect(() => {
		if (!cap.parallax) {
			x = 0;
			y = 0;
			return;
		}
		const move = (e: PointerEvent) => {
			x = (e.clientX / innerWidth - 0.5) * 2;
			y = (e.clientY / innerHeight - 0.5) * 2;
		};
		window.addEventListener('pointermove', move, { passive: true });
		return () => window.removeEventListener('pointermove', move);
	});
	onMount(() => {
		const timer = setInterval(() => {
			hour = new Date().getHours();
		}, 60000);
		return () => {
			clearInterval(timer);
			video?.pause();
		};
	});
</script>

<div
	class="stage"
	class:preview
	data-motion={cap.motion}
	data-video-playing={cap.video && !failed && !!source}
>
	<SceneFrame {config} {x} {y} {phase} />
	{#if cap.video && source && !failed}
		<video
			bind:this={video}
			class="scene-video"
			autoplay
			muted
			loop
			playsinline
			preload="metadata"
			onerror={() => (failed = true)}
			aria-hidden="true"
		>
			{#if source.startsWith('/scenes/')}<source src={`${source}.webm`} type="video/webm" /><source
					src={`${source}.mp4`}
					type="video/mp4"
				/>{:else}<source src={source} />{/if}
		</video>
	{/if}
	{#if cap.shader}<ShaderLayer fps={cap.fps} dpr={cap.dpr} />{/if}
	{#if cap.particles}<div class="particles" aria-hidden="true">
			{#each Array.from({ length: cap.maxParticles }, (_, i) => i) as i (i)}<i
					style={`--i:${i};--left:${(i * 37 + 13) % 100}%;--delay:${-i * 0.9}s;--speed:${9 + (i % 6)}s`}
				></i>{/each}
		</div>{/if}
	{#if config.weather !== 'clear'}<div
			class={`weather ${config.weather}`}
			class:animate={cap.motion}
			aria-hidden="true"
		></div>{/if}
	{#if preview && failed}<span class="fallback" role="status"
			>{tr(
				'Vista estática · el vídeo no se puede reproducir',
				'Static view · video cannot be played'
			)}</span
		>{/if}
</div>

<style>
	.stage {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
	}
	.scene-video {
		position: absolute;
		width: 100%;
		height: 100%;
		object-fit: cover;
		opacity: 0.72;
	}
	.particles {
		position: absolute;
		inset: 0;
		overflow: hidden;
	}
	.particles i {
		position: absolute;
		bottom: -8px;
		left: var(--left);
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: #e2f7c6;
		box-shadow: 0 0 8px #e2f7c677;
		opacity: 0;
		animation: drift var(--speed) var(--delay) infinite linear;
	}
	@keyframes drift {
		0% {
			transform: translateY(0);
			opacity: 0;
		}
		20% {
			opacity: 0.5;
		}
		80% {
			opacity: 0.3;
		}
		100% {
			transform: translate(25px, -60vh);
			opacity: 0;
		}
	}
	.weather {
		position: absolute;
		inset: -10%;
		opacity: 0.15;
	}
	.mist {
		background: radial-gradient(ellipse at 20% 70%, #d1e7f7, transparent 60%);
	}
	.rain {
		background: repeating-linear-gradient(
			110deg,
			transparent 0 45px,
			#bcdce5 46px,
			transparent 47px
		);
	}
	.snow {
		background-image: radial-gradient(circle, #fff 1px, transparent 2px);
		background-size: 83px 67px;
	}
	.weather.animate {
		animation: weather 18s linear infinite;
	}
	.rain.animate {
		animation-duration: 2s;
	}
	@keyframes weather {
		to {
			transform: translate(5%, 10%);
		}
	}
	.fallback {
		position: absolute;
		bottom: 10px;
		left: 10px;
		background: #101820dc;
		color: #fff;
		font-size: 11px;
		padding: 6px 8px;
		border-radius: 4px;
	}
	.preview {
		border-radius: inherit;
	}
</style>
