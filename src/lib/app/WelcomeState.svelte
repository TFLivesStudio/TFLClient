<script lang="ts">
	import Tfl from '$lib/icons/Tfl.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { openModal } from '$lib/state/uiState.svelte';
	import { Plus, Sparkles, PackageOpen, Zap } from 'lucide-svelte';
</script>

<div class="empty-state">
	<div class="welcome-orb"><Tfl width="42" height="42" /></div>
	<div class="welcome-copy">
		<span class="eyebrow">TFL Client</span>
		<h2>{t('home.title')}</h2>
		<p>{t('home.subtitle')}</p>
	</div>
	<div class="welcome-actions">
		<button type="button" class="empty-cta" onclick={() => openModal('createChooser')}>
			<Plus size={16} strokeWidth={2.5} />
			{t('sidebar.createInstance')}
		</button>
		<button type="button" class="secondary-cta" onclick={() => openModal('tflSelection')}>
			<Sparkles size={15} />
			{t('home.exploreTflSelection')}
		</button>
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
			var(--bg-card);
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
		font-size: var(--text-xs);
		font-weight: 800;
		letter-spacing: 0.13em;
		text-transform: uppercase;
		text-shadow: 0 1px 12px var(--bg-main);
	}
	.empty-state h2 {
		color: var(--text-primary);
		font-size: clamp(1.7rem, 4vw, 2.5rem);
		line-height: 1.08;
		letter-spacing: -0.045em;
		white-space: pre-line;
		/* El scrim de --wallpaper-scrim ya normaliza el fondo hacia el tono
		   del tema, pero acá el texto flota sin ningún panel/--bg-card
		   detrás — una sombra extra ata el contraste al fondo real de
		   pantalla (--bg-main) en vez de depender solo de eso. */
		text-shadow: 0 2px 24px var(--bg-main);
	}

	.empty-state p {
		max-width: 410px;
		font-size: 0.85rem;
		line-height: 1.55;
		color: var(--text-secondary);
		text-shadow: 0 1px 16px var(--bg-main);
	}

	.welcome-actions {
		display: flex;
		gap: 10px;
		margin-top: 6px;
	}

	.empty-cta {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		background: var(--accent);
		color: var(--accent-text);
		border: none;
		padding: 10px 22px;
		border-radius: var(--border-radius-sm);
		font-weight: 700;
		font-size: 0.82rem;
		cursor: pointer;
		box-shadow: var(--shadow-md);
		transition: transform 0.12s ease;
	}

	.empty-cta:hover {
		transform: translateY(-1px);
		box-shadow: 0 10px 26px rgba(var(--accent-rgb), 0.26);
	}

	.secondary-cta {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		padding: 10px 14px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--bg-card) 84%, transparent);
		color: var(--text-secondary);
		font-size: 0.82rem;
		font-weight: 700;
		cursor: pointer;
		transition:
			border-color 0.16s,
			color 0.16s,
			transform 0.16s;
	}
	.secondary-cta:hover {
		color: var(--text-primary);
		border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
		transform: translateY(-1px);
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
		background: color-mix(in srgb, var(--bg-card) 88%, transparent);
	}
	.welcome-card :global(svg) {
		grid-row: span 2;
		color: var(--accent);
	}
	.welcome-card span {
		font-size: var(--text-sm);
		font-weight: 750;
	}
	.welcome-card small {
		color: var(--text-muted);
		font-size: var(--text-xs);
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
		.empty-cta,
		.secondary-cta {
			justify-content: center;
		}
	}
</style>
