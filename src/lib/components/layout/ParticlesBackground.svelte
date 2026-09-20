<script lang="ts">
	import Particles, { initParticlesEngine } from '@tsparticles/svelte';
	import { loadSlim } from '@tsparticles/slim';
	import type { ISourceOptions } from '@tsparticles/engine';

	let ready = $state(false);

	initParticlesEngine(async (engine) => {
		await loadSlim(engine);
	}).then(() => {
		ready = true;
	});

	// Motas de polvo/luz flotando despacio — decoración sutil, no un efecto
	// "de red" con líneas. Color del acento activo, leído una vez del CSS
	// (el usuario cambia de acento y reabre/reelige el modo Partículas —
	// no vale la pena una reactividad en vivo para un detalle de fondo).
	const accentColor =
		typeof document !== 'undefined'
			? getComputedStyle(document.documentElement).getPropertyValue('--accent').trim() || '#ff7a2e'
			: '#ff7a2e';

	const options: ISourceOptions = {
		fullScreen: { enable: false },
		fpsLimit: 30,
		// Corta la animación sola en cuanto la ventana pierde foco o queda
		// fuera de pantalla (minimizada) — sin esto seguía consumiendo CPU/GPU
		// de fondo aunque nadie la estuviera viendo.
		pauseOnBlur: true,
		pauseOnOutsideViewport: true,
		particles: {
			number: { value: 24, density: { enable: true, width: 1600, height: 900 } },
			color: { value: accentColor },
			opacity: {
				value: { min: 0.08, max: 0.35 },
				animation: { enable: true, speed: 0.4, sync: false }
			},
			size: { value: { min: 1, max: 3 } },
			move: {
				enable: true,
				speed: 0.3,
				direction: 'top',
				random: true,
				straight: false,
				outModes: { default: 'out' }
			},
			links: { enable: false }
		},
		interactivity: {
			events: { onHover: { enable: false }, onClick: { enable: false } }
		},
		// El buffer del canvas en pantallas retina/HiDPI se multiplica por el
		// devicePixelRatio (2x-3x más píxeles reales) — para 24 motas
		// decorativas no vale ese costo de memoria, se nota igual en 1x.
		detectRetina: false
	};
</script>

{#if ready}
	<Particles id="tfl-particles" {options} class="particles-layer" />
{/if}

<style>
	:global(.particles-layer) {
		position: absolute;
		inset: 0;
	}
</style>
