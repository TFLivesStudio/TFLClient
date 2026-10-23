<script lang="ts">
	import { reaction } from '$lib/scenes/reactions.svelte';
	import { downloadTasks } from '$lib/state/downloadState.svelte';
	import { activityState } from '$lib/activity/state.svelte';
	import { terminal } from '$lib/activity/contract';
	import { effects } from '$lib/visual/runtime.svelte';
	import { scenes } from '$lib/scenes/preferences.svelte';
	const downloading = $derived(
		Object.values(downloadTasks).some((t) => !t.failed) ||
			activityState.snapshot.tasks.some((task) => !terminal(task.status))
	);

	import { SkinViewer3D, Mascot } from '$lib/components/ui';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { getMascotFor } from '$lib/mascots';
	import { t } from '$lib/i18n/index.svelte';
	import { Shirt } from 'lucide-svelte';
	import type { MojangProfile } from '$lib/types/types';

	/**
	 * Personaje de la Home — Entrega C, sección 1 del encargo. Antes vivía
	 * inline en WelcomeState.svelte (efecto 215-226 original): acá solo
	 * queda la PRESENTACIÓN (visor 3D compartido + mascota/orbe de
	 * fallback + fila de miniaturas de skin); el fetching del perfil de
	 * Mojang sigue en WelcomeState.svelte tal cual estaba (sección 1:
	 * "no es esta entrega para extraer TODO el data-fetching").
	 *
	 * Tres estados posibles (sección 6 del encargo), en el mismo orden que
	 * ya tenía el código original:
	 *  - cuenta premium (MS/Yggdrasil) → skinview3d con el skin/capa real.
	 *  - cuenta offline → mascota (no tiene skin real, nada que mostrar en
	 *    3D).
	 *  - sin cuenta en absoluto → orbe con el logo, con una pista de qué
	 *    hacer (antes quedaba mudo).
	 */
	let {
		isPremium,
		accountUuid,
		profile,
		onOpenSkinManager,
		launching = false
	}: {
		isPremium: boolean;
		accountUuid: string | null;
		profile: MojangProfile | null;
		onOpenSkinManager: () => void;
		launching?: boolean;
	} = $props();

	const activeSkin = $derived(profile?.skins.find((s) => s.state === 'ACTIVE') ?? null);
	const activeCape = $derived(profile?.capes.find((c) => c.state === 'ACTIVE') ?? null);

	// Fallo de carga de skin (sección 10.4) — no se queda pegado: en cuanto
	// cambia la URL del skin activo (otra cuenta, skin recién subido) se
	// vuelve a intentar sin arrastrar el error anterior.
	let skinError = $state(false);
	$effect(() => {
		void activeSkin?.url;
		skinError = false;
	});
</script>

<div
	class="character-figure"
	class:reactive={effects.current.cinematic && scenes.global.reactive}
	class:working={downloading || launching}
	data-reaction={reaction.kind}
>
	{#if isPremium}
		<SkinViewer3D
			priority="hero"
			{launching}
			width={220}
			height={320}
			autoRotateSpeed={0.6}
			zoom={0.9}
			skinUrl={activeSkin?.url ?? null}
			capeUrl={activeCape?.url ?? null}
			onSkinError={() => (skinError = true)}
		/>
	{:else if accountUuid}
		<div class="mascot-big">
			<Mascot id={getMascotFor(accountUuid)} size={165} />
		</div>
	{:else}
		<div class="no-account-orb"><Tfl width="36" height="36" /></div>
	{/if}
</div>

{#if isPremium && skinError}
	<p class="skin-error">{t('home.skinLoadError')}</p>
{:else if !isPremium && !accountUuid}
	<p class="no-account-caption">{t('home.noAccountCaption')}</p>
{/if}

{#if isPremium}
	<div class="skins-row">
		{#each (profile?.skins ?? []).slice(0, 3) as skin (skin.id)}
			<button
				type="button"
				class="skin-slot"
				title={t('home.openSkinManager')}
				onclick={onOpenSkinManager}
			>
				<span class="skin-layer base" style="background-image: url({skin.url})"></span>
				<span class="skin-layer overlay" style="background-image: url({skin.url})"></span>
			</button>
		{/each}
		<button
			type="button"
			class="skin-slot skin-slot-add"
			title={t('home.openSkinManager')}
			onclick={onOpenSkinManager}
		>
			<Shirt size={14} />
		</button>
	</div>
{/if}

<style>
	.character-figure {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 150px;
	}
	.character-figure :global(canvas) {
		filter: drop-shadow(0 14px 28px rgba(0, 0, 0, 0.45));
	}
	.mascot-big {
		filter: drop-shadow(0 14px 28px rgba(0, 0, 0, 0.45));
	}

	.no-account-orb {
		display: grid;
		place-items: center;
		width: 160px;
		height: 160px;
		border: 1px solid color-mix(in srgb, var(--accent) 48%, var(--border));
		border-radius: 40px;
		color: var(--accent);
		background:
			linear-gradient(145deg, color-mix(in srgb, var(--accent) 25%, transparent), transparent),
			var(--surface-card);
		box-shadow:
			0 20px 55px rgba(var(--accent-rgb), 0.18),
			var(--shadow-md);
	}

	.no-account-caption,
	.skin-error {
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		color: var(--text-muted);
		text-align: center;
	}
	.skin-error {
		color: var(--color-error);
	}

	.skins-row {
		display: flex;
		gap: 6px;
	}
	.skin-slot {
		position: relative;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--surface-interactive);
		overflow: hidden;
		cursor: pointer;
		padding: 0;
	}
	.skin-slot:hover {
		border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
	}
	/* Mismo recorte de cara (64x64 → cabeza, base + capa "hat") que ya usa
	   Sidebar.svelte para la cabecita del usuario. */
	.skin-slot .skin-layer {
		position: absolute;
		inset: 0;
		background-repeat: no-repeat;
		background-size: 256px 256px;
		image-rendering: pixelated;
	}
	.skin-slot .skin-layer.base {
		background-position: -32px -32px;
	}
	.skin-slot .skin-layer.overlay {
		background-position: -160px -32px;
	}
	.skin-slot-add {
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
	}
	.skin-slot-add:hover {
		color: var(--accent);
	}
	.reactive[data-reaction='success'] .mascot-big {
		animation: mascot-hello 1.1s ease-in-out;
	}
	.reactive[data-reaction='error'] .mascot-big {
		animation: mascot-shake 0.5s ease-in-out;
	}
	.reactive.working .mascot-big {
		animation: mascot-work 2.4s ease-in-out infinite;
	}
	@keyframes mascot-hello {
		25%,
		65% {
			transform: translateY(-10px) rotate(-4deg);
		}
		45%,
		85% {
			transform: translateY(-4px) rotate(4deg);
		}
	}
	@keyframes mascot-shake {
		25%,
		75% {
			transform: translateX(-4px);
		}
		50% {
			transform: translateX(4px);
		}
	}
	@keyframes mascot-work {
		50% {
			transform: translateY(-4px) rotate(2deg);
		}
	}
</style>
