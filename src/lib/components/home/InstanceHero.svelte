<script lang="ts">
	import { Surface, Button, Badge } from '$lib/components/ui';
	import CharacterPanel from './CharacterPanel.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { viewport } from '$lib/state/viewport.svelte';
	import type { InstanceData, MojangProfile } from '$lib/types/types';
	import { Play } from 'lucide-svelte';

	/**
	 * Hero de la Home — instancia destacada + personaje + botón Jugar como
	 * UNA sola composición (sección 7.1 del encargo, textual explícito: "no
	 * un banner diminuto + un visor 3D aislado en una card genérica aparte").
	 * Antes eran dos bloques separados en WelcomeState.svelte (el `.banner`
	 * de las líneas 327-361 y el `.panel.character-panel` de las líneas
	 * 364-399) con fondos/radios propios cada uno; ahora comparten una sola
	 * `Surface` (nivel "elevated" — Entrega A, el escalón justo debajo de
	 * overlay, para que el hero se sienta por encima de los paneles
	 * secundarios de abajo) con el artwork de fondo detrás de los dos.
	 *
	 * A 800×600 (ventana mínima soportada) el personaje puede no entrar
	 * cómodo al lado de la identidad+Play sin forzar scroll para ver el
	 * botón — por eso la composición pasa a columna (personaje abajo,
	 * scrolleable) por debajo de `HERO_SPLIT_MIN_WIDTH`, en vez de un
	 * `@media` fijo: el ancho real disponible lo da la ventana completa
	 * (sidebar + main-content), que es justo lo que ya expone
	 * `viewport.width` (Entrega B) — ver comentario largo en
	 * viewport.svelte.ts sobre por qué este módulo prefiere esa fuente a un
	 * listener propio.
	 */
	const HERO_SPLIT_MIN_WIDTH = 1000;

	let {
		featured,
		iconUrl,
		bannerBg,
		lastPlayedLabel,
		playTimeLabel,
		isRunning,
		launching,
		launchError,
		onPlay,
		isPremium,
		accountUuid,
		profile,
		onOpenSkinManager
	}: {
		featured: InstanceData;
		iconUrl: string | null;
		bannerBg: string | null;
		lastPlayedLabel: string | null;
		playTimeLabel: string | null;
		isRunning: boolean;
		launching: boolean;
		launchError: string | null;
		onPlay: () => void;
		isPremium: boolean;
		accountUuid: string | null;
		profile: MojangProfile | null;
		onOpenSkinManager: () => void;
	} = $props();

	const showCharacterInline = $derived(viewport.width >= HERO_SPLIT_MIN_WIDTH);
</script>

<Surface level="elevated" as="section">
	<div
		class="hero"
		class:has-bg={!!bannerBg}
		class:stacked={!showCharacterInline}
		style={bannerBg ? `--hero-bg: url(${bannerBg})` : ''}
	>
		<div class="hero-identity">
			<span class="hero-eyebrow">{t('home.continuePlaying')}</span>
			<div class="hero-row">
				<span class="hero-icon">
					{#if iconUrl}
						<img src={iconUrl} alt="" />
					{:else}
						{featured.name.charAt(0).toUpperCase()}
					{/if}
				</span>
				<div class="hero-info">
					<span class="hero-name-row">
						<span class="hero-name">{featured.name}</span>
						{#if isRunning}
							<Badge variant="success">{t('home.runningBadge')}</Badge>
						{/if}
					</span>
					<span class="hero-meta">
						{featured.mc_version} · {featured.loader}
						· {lastPlayedLabel ?? t('home.neverPlayed')}
						{#if playTimeLabel}
							· {playTimeLabel}
						{/if}
					</span>
				</div>
			</div>

			<Button variant="primary" size="prominent" loading={launching} onclick={onPlay}>
				{#snippet icon()}
					<Play size={18} fill="currentColor" />
				{/snippet}
				{launching ? t('home.launching') : t('home.play')}
			</Button>

			{#if launchError}<p class="launch-error">{launchError}</p>{/if}
		</div>

		<div class="hero-character">
			<CharacterPanel {isPremium} {accountUuid} {profile} {onOpenSkinManager} />
		</div>
	</div>
</Surface>

<style>
	.hero {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 20px;
		padding: 26px 30px;
		border-radius: inherit;
		overflow: hidden;
	}

	/* El degradado de marca (sin captura) y el scrim de contraste (con
	   captura) viven en el mismo pseudo-elemento de fondo — así nunca
	   compiten por el mismo z-index que el contenido real. */
	.hero::before {
		content: '';
		position: absolute;
		inset: 0;
		z-index: 0;
		background:
			linear-gradient(
				120deg,
				color-mix(in srgb, var(--accent) 16%, transparent) 0%,
				transparent 55%
			),
			radial-gradient(
				circle at 85% 15%,
				color-mix(in srgb, var(--accent) 20%, transparent),
				transparent 60%
			);
	}

	/* Con captura real: recorte "inteligente" simple (foco un poco por
	   encima del centro, donde suele estar el horizonte/personaje en una
	   captura de Minecraft) + degradado de lectura de abajo hacia arriba,
	   lo bastante fuerte para que nombre+Play se lean sobre cualquier
	   captura clara u oscura sin depender de qué tan clara/oscura sea. */
	.hero.has-bg::before {
		background-image:
			linear-gradient(
				0deg,
				rgba(0, 0, 0, 0.82) 0%,
				rgba(0, 0, 0, 0.52) 45%,
				rgba(0, 0, 0, 0.22) 100%
			),
			var(--hero-bg, none);
		background-size: cover;
		background-position: center 38%;
	}

	.hero-identity {
		position: relative;
		z-index: 1;
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
		flex: 1;
	}

	.hero-eyebrow {
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: color-mix(in srgb, var(--accent) 70%, white 10%);
	}

	.hero-row {
		display: flex;
		align-items: center;
		gap: 14px;
	}

	.hero-icon {
		flex-shrink: 0;
		width: 52px;
		height: 52px;
		border-radius: var(--border-radius);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		font-size: var(--text-heading);
		font-weight: 800;
		color: var(--accent);
		background: color-mix(in srgb, var(--accent) 20%, var(--surface-card));
	}
	.hero-icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.hero-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}
	.hero-name-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.hero-name {
		font-size: var(--text-hero);
		line-height: var(--line-height-hero);
		font-weight: 800;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.hero-meta {
		font-size: var(--text-small);
		line-height: var(--line-height-small);
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.launch-error {
		font-size: var(--text-small);
		line-height: var(--line-height-small);
		color: var(--color-error);
	}

	.hero-character {
		position: relative;
		z-index: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		flex-shrink: 0;
	}

	/* A 800×600 (o cualquier ventana angosta): el personaje pasa a ocupar
	   una fila propia debajo de la identidad+Play, nunca al lado — eso es
	   lo que garantiza que el botón Jugar quede visible sin scroll inicial
	   (criterio de aceptación explícito, sección 2 del encargo), aunque el
	   personaje mismo termine requiriendo scroll para verse. */
	.hero.stacked {
		flex-direction: column;
		align-items: stretch;
	}
	.hero.stacked .hero-character {
		align-self: center;
	}
</style>
