<script lang="ts">
	import { onMount } from 'svelte';
	import { subscribeFrame } from './frameClock';
	import type { SceneId } from './model';
	let {
		preset,
		fps = 24,
		depth = 0.4
	}: { preset: SceneId; fps?: number; depth?: number } = $props();
	let root: HTMLDivElement;
	onMount(() =>
		subscribeFrame((_, now) => {
			const t = now / 1000;
			root.style.setProperty('--drift', `${Math.sin(t * 0.075) * 3 * depth}%`);
			root.style.setProperty('--breathe', `${0.65 + Math.sin(t * 0.24) * 0.2}`);
			root.style.setProperty('--rise', `${Math.cos(t * 0.09) * 2 * depth}%`);
		}, fps)
	);
</script>

<div bind:this={root} class="atmosphere" data-biome={preset} aria-hidden="true">
	<div class="mist distant"></div>
	<div class="mist close"></div>
	<div class="rays"></div>
	<div class="water"></div>
	{#if preset === 'pixel'}<div class="torch a"></div>
		<div class="torch b"></div>{/if}
</div>

<style>
	.atmosphere {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
		--drift: 0%;
		--rise: 0%;
		--breathe: 0.7;
		--fog: #d3eee3;
		--glow: #ffdd9a;
	}
	.mist {
		position: absolute;
		left: -12%;
		width: 125%;
		height: 40%;
		background:
			radial-gradient(
				ellipse at 30% 70%,
				color-mix(in srgb, var(--fog) 18%, transparent),
				transparent 65%
			),
			radial-gradient(
				ellipse at 80% 30%,
				color-mix(in srgb, var(--fog) 12%, transparent),
				transparent 60%
			);
		transform: translate3d(var(--drift), var(--rise), 0);
		opacity: var(--breathe);
	}
	.distant {
		top: 30%;
		mix-blend-mode: screen;
	}
	.close {
		top: 62%;
		transform: translate3d(calc(var(--drift) * -1), 0, 0);
		opacity: 0.3;
	}
	.rays {
		position: absolute;
		inset: -5%;
		background: conic-gradient(
			from 170deg at 83% 20%,
			transparent 0deg,
			#ffe2a311 8deg,
			transparent 14deg,
			#fff1ba09 22deg,
			transparent 30deg
		);
		mix-blend-mode: screen;
		opacity: var(--breathe);
		transform: translate3d(var(--drift), 0, 0);
	}
	.water {
		position: absolute;
		inset: 65% 0 0;
		background: repeating-linear-gradient(
			177deg,
			transparent 0 13px,
			#d7f8ff09 14px,
			transparent 16px
		);
		clip-path: polygon(62% 0, 86% 0, 100% 100%, 24% 100%);
		transform: translate3d(var(--rise), 0, 0);
		opacity: var(--breathe);
		mix-blend-mode: screen;
	}
	[data-biome='embers'] {
		--fog: #d84a2e;
		--glow: #ff811b;
	}
	[data-biome='embers'] .water {
		background: radial-gradient(ellipse at 50% 10%, #ff771a33, transparent 68%);
		clip-path: none;
	}
	[data-biome='embers'] .rays {
		background: radial-gradient(ellipse at 68% 40%, #f8491320, transparent 60%);
	}
	[data-biome='void'] {
		--fog: #ae83f5;
	}
	[data-biome='void'] .water {
		display: none;
	}
	[data-biome='void'] .rays {
		background: radial-gradient(ellipse at 68% 18%, #c3a7f92b, transparent 45%);
	}
	[data-biome='aurora'] {
		--fog: #63e9c6;
	}
	[data-biome='aurora'] .rays {
		background: radial-gradient(ellipse at 40% 10%, #5ee8b71a, transparent 60%);
	}
	[data-biome='pixel'] .mist {
		opacity: 0.18;
	}
	[data-biome='pixel'] .rays {
		opacity: 0.25;
	}
	.torch {
		position: absolute;
		width: 4%;
		height: 6%;
		background: radial-gradient(ellipse, #ffb52f66, transparent 70%);
		opacity: var(--breathe);
		mix-blend-mode: screen;
	}
	.torch.a {
		left: 89%;
		top: 53%;
	}
	.torch.b {
		left: 93%;
		top: 57%;
	}
</style>
