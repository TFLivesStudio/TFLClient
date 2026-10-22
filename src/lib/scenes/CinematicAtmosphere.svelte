<script lang="ts">
	import type { SceneId } from './model';
	let {
		preset,
		fps = 24,
		depth = 0.4
	}: { preset: SceneId; fps?: number; depth?: number } = $props();
</script>

<div
	class="atmosphere"
	data-biome={preset}
	style:--depth={depth}
	style:--fog-steps={Math.round(fps * 28)}
	style:--water-steps={Math.round(fps * 16)}
	aria-hidden="true"
>
	<div class="mist distant"></div>
	{#if depth > 0.5}<div class="mist close"></div>
		<div class="rays"></div>{/if}
	<div class="water"></div>
</div>

<style>
	.atmosphere {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
		--fog: #d3eee3;
	}
	.mist {
		position: absolute;
		left: -8%;
		width: 116%;
		height: 35%;
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
		opacity: 0.5;
		animation: mist-drift 28s steps(var(--fog-steps), end) infinite alternate;
	}
	.distant {
		top: 32%;
		mix-blend-mode: screen;
	}
	.close {
		top: 62%;
		animation-direction: alternate-reverse;
		opacity: 0.25;
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
		opacity: 0.5;
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
		animation: water-drift 16s steps(var(--water-steps), end) infinite alternate;
		opacity: 0.5;
		mix-blend-mode: screen;
	}
	[data-biome='embers'] {
		--fog: #d84a2e;
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
	@keyframes mist-drift {
		from {
			transform: translate3d(calc(-3% * var(--depth)), 0, 0);
		}
		to {
			transform: translate3d(calc(3% * var(--depth)), calc(-2% * var(--depth)), 0);
		}
	}
	@keyframes water-drift {
		from {
			transform: translate3d(calc(-1% * var(--depth)), 0, 0);
		}
		to {
			transform: translate3d(calc(1% * var(--depth)), 0, 0);
		}
	}
</style>
