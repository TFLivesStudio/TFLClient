<!-- A bounded CSS scene: no particle engine, canvas, timers, or application render loop. -->
<div class="particles-layer" aria-hidden="true">
	{#each Array.from({ length: 12 }, (_, i) => i) as i (i)}<span
			style={`--x:${(i * 37) % 100}%;--y:${(i * 23) % 90}%;--delay:-${i * 1.3}s;--duration:${12 + (i % 5)}s`}
		></span>{/each}
</div>

<style>
	.particles-layer {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
	}
	.particles-layer span {
		position: absolute;
		left: var(--x);
		top: var(--y);
		width: 3px;
		height: 3px;
		background: var(--accent);
		opacity: 0.2;
		border-radius: 50%;
		animation: drift var(--duration) var(--delay) ease-in-out infinite alternate;
	}
	@keyframes drift {
		to {
			transform: translate3d(16px, -50px, 0);
			opacity: 0.5;
		}
	}
</style>
