<script lang="ts">
	import Tfl from '$lib/icons/Tfl.svelte';
	import { tr } from '$lib/i18n/experience';
</script>

<section class="empty-state" aria-labelledby="empty-home-title">
	<div class="welcome-mark" aria-hidden="true"><Tfl width="36" height="36" /></div>
	<h1 id="empty-home-title">
		{tr('Tu próximo mundo empieza aquí', 'Your next world starts here')}
	</h1>
	<p>
		{tr(
			'Usa la barra lateral para crear o importar una instancia, o para explorar TFL Selection. Tus juegos aparecerán aquí cuando estén listos.',
			'Use the sidebar to create or import an instance, or explore TFL Selection. Your games will appear here when ready.'
		)}
	</p>
</section>

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 18px;
		min-height: 360px;
		max-width: 620px;
		margin: auto;
		padding: 60px 28px;
		text-align: center;
	}
	.welcome-mark {
		display: grid;
		place-items: center;
		width: 72px;
		height: 72px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		background: var(--surface-card);
		color: var(--accent);
	}
	h1 {
		font-size: clamp(1.5rem, 3vw, 2.25rem);
		line-height: 1.2;
		letter-spacing: -0.035em;
		color: var(--text-primary);
		text-shadow: 0 1px 12px var(--surface-canvas);
	}
	p {
		max-width: 440px;
		font-size: var(--text-small);
		line-height: 1.65;
		color: var(--text-secondary);
		padding: 12px 16px;
		border-radius: var(--border-radius);
		background: var(--surface-card);
	}
</style>
