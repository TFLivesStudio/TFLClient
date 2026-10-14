<script lang="ts">
	import { Button } from '$lib/components/ui';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { Plus, Sparkles, PackageOpen, Zap } from 'lucide-svelte';

	/** Estado "sin instancias todavía" de la Home — Entrega C. Markup/
	 * estilos extraídos de WelcomeState.svelte (líneas 294-324 + 562-711
	 * originales). No es una de las 6 secciones listadas en el encargo
	 * para extraer (esas son del dashboard con instancias), pero sacarla
	 * igual a su propio componente evita que el orquestador vuelva a
	 * cargarse de markup que no necesita leer para seguir el flujo
	 * principal (sección 17: "no debe seguir creciendo como archivo
	 * monolítico"). Sin props: no depende de ningún dato cargado, solo
	 * dispara los dos callbacks que ya existían. */
	let {
		onCreateInstance,
		onExploreTflSelection
	}: {
		onCreateInstance: () => void;
		onExploreTflSelection: () => void;
	} = $props();
</script>

<div class="empty-state">
	<div class="welcome-orb"><Tfl width="42" height="42" /></div>
	<div class="welcome-copy">
		<span class="eyebrow">TFL Client</span>
		<h2>{t('home.title')}</h2>
		<p>{t('home.subtitle')}</p>
	</div>
	<div class="welcome-actions">
		<Button variant="primary" size="normal" onclick={onCreateInstance}>
			{#snippet icon()}<Plus size={16} strokeWidth={2.5} />{/snippet}
			{t('sidebar.createInstance')}
		</Button>
		<Button variant="secondary" size="normal" onclick={onExploreTflSelection}>
			{#snippet icon()}<Sparkles size={15} />{/snippet}
			{t('home.exploreTflSelection')}
		</Button>
	</div>
	<div class="welcome-grid">
		<div class="welcome-card">
			<PackageOpen size={16} /><span>{t('home.modsCardTitle')}</span><small
				>{t('home.modsCardHint')}</small
			>
		</div>
		<div class="welcome-card">
			<Zap size={16} /><span>{t('home.javaCardTitle')}</span><small>{t('home.javaCardHint')}</small>
		</div>
	</div>
</div>

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 14px;
		height: 100%;
		max-width: 620px;
		margin: auto;
		padding: 48px 28px;
		text-align: center;
	}

	.welcome-orb {
		display: grid;
		place-items: center;
		width: 76px;
		height: 76px;
		border: 1px solid color-mix(in srgb, var(--accent) 48%, var(--border));
		border-radius: 26px;
		color: var(--accent);
		background:
			linear-gradient(145deg, color-mix(in srgb, var(--accent) 25%, transparent), transparent),
			var(--surface-card);
		box-shadow:
			0 20px 55px rgba(var(--accent-rgb), 0.18),
			var(--shadow-md);
	}

	.welcome-copy {
		display: grid;
		gap: 8px;
	}
	.eyebrow {
		color: var(--accent);
		font-size: var(--text-caption);
		font-weight: 800;
		letter-spacing: 0.13em;
		text-transform: uppercase;
		text-shadow: 0 1px 12px var(--surface-canvas);
	}
	.empty-state h2 {
		color: var(--text-primary);
		font-size: clamp(1.7rem, 4vw, 2.5rem);
		line-height: 1.08;
		letter-spacing: -0.045em;
		white-space: pre-line;
		/* El scrim de --wallpaper-scrim ya normaliza el fondo hacia el tono
		   del tema, pero acá el texto flota sin ningún panel/--surface-card
		   detrás — una sombra extra ata el contraste al fondo real de
		   pantalla (--surface-canvas) en vez de depender solo de eso. */
		text-shadow: 0 2px 24px var(--surface-canvas);
	}

	.empty-state p {
		max-width: 410px;
		font-size: var(--text-small);
		line-height: var(--line-height-small);
		color: var(--text-secondary);
		text-shadow: 0 1px 16px var(--surface-canvas);
	}

	.welcome-actions {
		display: flex;
		gap: 10px;
		margin-top: 6px;
	}

	.welcome-grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(150px, 1fr));
		gap: 10px;
		width: min(100%, 430px);
		margin-top: 10px;
	}
	.welcome-card {
		display: grid;
		grid-template-columns: auto 1fr;
		column-gap: 9px;
		align-items: center;
		padding: 13px;
		text-align: left;
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		background: var(--surface-card);
	}
	.welcome-card :global(svg) {
		grid-row: span 2;
		color: var(--accent);
	}
	.welcome-card span {
		font-size: var(--text-small);
		font-weight: 750;
	}
	.welcome-card small {
		color: var(--text-muted);
		font-size: var(--text-caption);
		margin-top: 2px;
	}

	@media (max-width: 640px) {
		.welcome-actions,
		.welcome-grid {
			grid-template-columns: 1fr;
			width: 100%;
		}
		.welcome-actions {
			flex-direction: column;
		}
	}
</style>
