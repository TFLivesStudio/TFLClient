<script lang="ts">
	import { Play, ArrowUpRight, Clock, Box } from 'lucide-svelte';
	import { Button, Badge } from '$lib/components/ui';
	import CharacterPanel from './CharacterPanel.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { instanceVisual } from '$lib/visual/preferences.svelte';
	import { artworkUrl } from '$lib/visual/policy';
	import { effects } from '$lib/visual/runtime.svelte';
	import { parallax } from '$lib/visual/motion';
	import { selectInstance } from '$lib/state/instanceState.svelte';
	import LivingScene from '$lib/scenes/LivingScene.svelte';
	import type { InstanceData, MojangProfile } from '$lib/types/types';
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
	const artwork = $derived(instanceVisual(featured.uuid).artwork);
	const image = $derived(artwork === 'none' ? null : (artworkUrl(artwork) ?? bannerBg));
</script>

<section
	class="hero"
	class:launching
	class:running={isRunning}
	use:parallax={effects.current.parallax}
	style={image ? `--hero-image:url("${image}")` : artwork === 'none' ? '--hero-image:none' : ''}
>
	<div class="hero-art" aria-hidden="true"></div>
	<div class="hero-scene" aria-hidden="true"><LivingScene instance={featured} /></div>
	<div class="hero-light" aria-hidden="true"></div>
	<div class="hero-identity">
		<span class="vr-eyebrow"><span class="hero-dot"></span>{vr('continue')}</span>
		<div class="hero-edition" data-instance-artwork={featured.uuid}>
			{#if iconUrl}<img src={iconUrl} alt="" />{:else}<Box size={20} />{/if}<span
				>MINECRAFT <span class="edition-divider">/</span> {featured.mc_version}</span
			>
		</div>
		<h1 class="hero-name">{featured.name}</h1>
		<div class="hero-meta">
			<Badge>{featured.loader}</Badge>{#if isRunning}<Badge variant="success"
					>{t('home.runningBadge')}</Badge
				>{/if}<span><Clock size={14} />{lastPlayedLabel ?? t('home.neverPlayed')}</span
			>{#if playTimeLabel}<span>{playTimeLabel}</span>{/if}
		</div>
		<div class="hero-actions">
			<Button variant="primary" size="prominent" loading={launching} onclick={onPlay}
				>{#snippet icon()}<Play size={21} fill="currentColor" />{/snippet}{launching
					? t('home.launching')
					: t('home.play')}</Button
			><button
				class="hero-detail"
				onclick={() => selectInstance(featured)}
				aria-label={`${vr('open')} ${featured.name}`}><ArrowUpRight size={21} /></button
			>
		</div>
		{#if launching}<p class="launch-caption" role="status">{vr('launch')}</p>{/if}
		{#if launchError}<p class="launch-error" role="alert">{launchError}</p>{/if}
	</div>
	<div class="hero-character">
		<div class="character-stage" aria-hidden="true"></div>
		<CharacterPanel {isPremium} {accountUuid} {profile} {onOpenSkinManager} {launching} />
	</div>
	<span class="hero-coordinate" aria-hidden="true">TFL / {featured.loader.toUpperCase()}</span>
</section>

<style>
	.hero {
		position: relative;
		display: flex;
		align-items: center;
		isolation: isolate;
		min-height: 370px;
		padding: 40px 44px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		overflow: hidden;
		background: #111820;
		color: #fff;
		--hero-image: url('/wallpapers/mc-wallpaper-6.jpg');
		--pointer-x: 0px;
		--pointer-y: 0px;
	}
	.hero-art {
		position: absolute;
		inset: -12px;
		z-index: -3;
		background: var(--hero-image);
		background-size: cover;
		background-position: center 42%;
		transform: translate3d(var(--pointer-x), var(--pointer-y), 0);
		transition: transform 180ms ease-out;
	}
	/* Encima de .hero-art (reemplaza la imagen fija por la escena viva
	   —vídeo/atmósfera— cuando hay una configurada; si no, .hero-art se ve
	   sin tapar). Mismo inset que .hero-art para que el parallax coincida. */
	.hero-scene {
		position: absolute;
		inset: -12px;
		z-index: -3;
		overflow: hidden;
		border-radius: inherit;
		transform: translate3d(var(--pointer-x), var(--pointer-y), 0);
		transition: transform 180ms ease-out;
	}
	.hero:after {
		content: '';
		position: absolute;
		inset: 0;
		z-index: -2;
		background:
			linear-gradient(90deg, #091019f5 0%, #091019c7 43%, #09101938 100%),
			linear-gradient(0deg, #091019ba, transparent 65%);
	}
	.hero-light {
		position: absolute;
		inset: 0;
		z-index: -1;
		background: radial-gradient(
			ellipse at 75% 30%,
			color-mix(in srgb, var(--accent) 22%, transparent),
			transparent 65%
		);
		opacity: 0.75;
		transition: opacity var(--duration-panel);
	}
	.hero.launching .hero-light {
		opacity: 1;
		background: radial-gradient(
			ellipse at 65% 45%,
			color-mix(in srgb, var(--accent) 40%, transparent),
			transparent 70%
		);
	}
	.hero-identity {
		flex: 1;
		min-width: 0;
		z-index: 1;
		max-width: 65%;
	}
	.hero .vr-eyebrow {
		color: #e2e5ec;
		font-size: 11px;
		letter-spacing: 0.18em;
	}
	.hero-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--accent);
	}
	.hero-edition {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-top: 34px;
		color: #b8c5d4;
		font-size: 12px;
		letter-spacing: 0.07em;
	}
	.hero-edition img {
		width: 26px;
		height: 26px;
		border-radius: 6px;
	}
	.edition-divider {
		margin: 0 8px;
		opacity: 0.5;
	}
	.hero-name {
		font-size: clamp(2.2rem, 4.5vw, 4.6rem);
		line-height: 1.04;
		letter-spacing: -0.065em;
		margin: 14px 0 18px;
		font-weight: 700;
		overflow-wrap: anywhere;
		max-width: 15ch;
	}
	.hero-meta {
		display: flex;
		gap: 12px;
		align-items: center;
		flex-wrap: wrap;
		color: #c4cbd5;
		font-size: var(--text-small);
	}
	.hero-meta > span {
		display: inline-flex;
		gap: 6px;
		align-items: center;
	}
	.hero-actions {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-top: 30px;
	}
	.hero-actions :global(.tfl-btn) {
		min-width: 180px;
		min-height: 52px;
		letter-spacing: 0.01em;
		font-size: 1.1rem;
	}
	.hero-detail {
		height: 52px;
		width: 52px;
		display: grid;
		place-items: center;
		border: 1px solid #ffffff30;
		border-radius: var(--border-radius);
		background: #ffffff0a;
		color: #fff;
	}
	.hero-detail:hover {
		background: #ffffff1a;
	}
	.hero-character {
		position: relative;
		width: 32%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 290px;
		transform: translate3d(calc(var(--pointer-x) * -1), calc(var(--pointer-y) * -1), 0);
	}
	.character-stage {
		pointer-events: none;
		position: absolute;
		bottom: 20px;
		width: 90%;
		height: 32px;
		border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
		border-radius: 50%;
		background: radial-gradient(
			ellipse,
			color-mix(in srgb, var(--accent) 25%, transparent),
			transparent 65%
		);
		transform: perspective(200px) rotateX(30deg);
	}
	.hero-coordinate {
		position: absolute;
		bottom: 14px;
		right: 22px;
		color: #d1d9e67a;
		font-family: 'JetBrains Mono', monospace;
		font-size: 10px;
		letter-spacing: 0.12em;
	}
	.launch-error {
		color: #ffb4b4;
		font-size: var(--text-small);
		line-height: 1.5;
		margin-top: 14px;
	}
	.launch-caption {
		color: #d6dee8;
		font-size: var(--text-small);
		margin-top: 12px;
	}
	:global(html[data-visual='pixel']) .hero {
		border-width: 2px;
		box-shadow: 6px 6px 0 #0005;
	}
	:global(html[data-visual='pixel']) .hero-name {
		font-family: 'Monocraft', monospace;
		font-size: clamp(1.8rem, 3.2vw, 3.3rem);
		line-height: 1.35;
		letter-spacing: -0.04em;
	}
	:global(html[data-visual='pixel']) .character-stage {
		border-radius: 0;
		transform: perspective(200px) rotateX(50deg) rotateZ(10deg);
	}
	@media (max-width: 1100px) {
		.hero {
			padding: 30px;
			min-height: 320px;
		}
		.hero-name {
			font-size: 2.7rem;
		}
		.hero-identity {
			max-width: 67%;
		}
		.hero-character {
			width: 30%;
		}
	}
	@media (max-width: 850px) {
		.hero {
			padding: 24px;
			min-height: 300px;
		}
		.hero-name {
			font-size: 2.5rem;
		}
		.hero-edition {
			margin-top: 24px;
		}
		.hero-character {
			min-height: 220px;
			transform: scale(0.85);
			transform-origin: center;
		}
		.hero-meta {
			gap: 8px;
			font-size: 12px;
		}
		.hero-actions {
			margin-top: 22px;
		}
		.hero-actions :global(.tfl-btn) {
			min-width: 140px;
			min-height: 46px;
		}
	}
</style>
