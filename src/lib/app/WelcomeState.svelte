<script lang="ts">
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { SkinViewer } from 'skinview3d';
	import Tfl from '$lib/icons/Tfl.svelte';
	import Mascot from '$lib/components/ui/Mascot.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { getMascotFor } from '$lib/mascots';
	import { t } from '$lib/i18n/index.svelte';
	import { openModal } from '$lib/state/uiState.svelte';
	import { appState } from '$lib/state/state.svelte';
	import { gameSession } from '$lib/state/gameSession.svelte';
	import { selectInstance, iconVersions } from '$lib/state/instanceState.svelte';
	import { multiInstanceWarningKind } from '$lib/state/launch';
	import { launchInstance, getMojangProfile, getInstanceIconPath, updateSettings } from '$lib/api';
	import type { InstanceData, MojangProfile } from '$lib/types/types';
	import { Plus, Sparkles, PackageOpen, Zap, Play } from 'lucide-svelte';

	// La instancia a retomar: la de cliente (no servidor) jugada más
	// recientemente. Si ninguna se jugó todavía, cae igual en la primera —
	// sigue siendo útil como atajo para "la que acabo de crear".
	const clientInstances = $derived(appState.instances.filter((i) => !i.server_type));
	const sortedByRecent = $derived(
		[...clientInstances].sort((a, b) => b.last_played - a.last_played)
	);
	const featured: InstanceData | null = $derived(sortedByRecent[0] ?? null);
	const others = $derived(sortedByRecent.slice(1, 4));

	const playTimeLabel = $derived.by(() => {
		const secs = featured?.play_time_secs ?? 0;
		if (!featured || secs < 60) return null;
		const h = Math.floor(secs / 3600);
		const m = Math.floor((secs % 3600) / 60);
		return h > 0
			? t('instanceDetail.playTimeHours', { h, m })
			: t('instanceDetail.playTimeMinutes', { m });
	});

	// Íconos de la card destacada y de las "otras instancias" — mismo patrón
	// que Sidebar.svelte (cacheado por uuid, se vuelve a pedir si cambia la
	// versión del ícono).
	let iconUrls = $state<Record<string, string | null>>({});
	const iconLoaded: Record<string, string> = {};
	$effect(() => {
		const candidates = [featured, ...others].filter((i): i is InstanceData => i !== null);
		for (const inst of candidates) {
			const version = iconVersions[inst.name] ?? 0;
			const key = `${inst.name}:${version}`;
			if (iconLoaded[inst.uuid] === key) continue;
			iconLoaded[inst.uuid] = key;
			getInstanceIconPath(inst.name)
				.then((path) => {
					iconUrls[inst.uuid] = path ? `${convertFileSrc(path)}?v=${version}` : null;
				})
				.catch(() => {
					iconUrls[inst.uuid] = null;
				});
		}
	});

	// Personaje grande (pedido del cliente, estilo LabyMod). Cuentas
	// Microsoft: mismo visor 3D que ya usa SkinManagerModal.svelte (misma
	// librería, mismo ciclo de vida canvas/dispose). Cuentas offline: no
	// tienen skin real, así que la mascota grande en vez de un visor 3D vacío.
	const isPremium = $derived(
		appState.currentUser !== null && appState.currentUser.user_type !== 'Cracked'
	);

	let canvasEl = $state<HTMLCanvasElement | undefined>(undefined);
	let viewer: SkinViewer | undefined;
	let profile = $state<MojangProfile | null>(null);

	function syncViewer() {
		if (!viewer || !profile) return;
		const skin = profile.skins.find((s) => s.state === 'ACTIVE');
		const cape = profile.capes.find((c) => c.state === 'ACTIVE');
		if (skin) viewer.loadSkin(skin.url);
		if (cape) viewer.loadCape(cape.url);
		else viewer.loadCape(null);
	}

	$effect(() => {
		if (!isPremium || !canvasEl || viewer) return;
		viewer = new SkinViewer({ canvas: canvasEl, width: 220, height: 320 });
		viewer.autoRotate = true;
		viewer.autoRotateSpeed = 0.6;
		viewer.zoom = 0.9;
		syncViewer();
		return () => {
			viewer?.dispose();
			viewer = undefined;
		};
	});

	$effect(() => {
		if (!isPremium) {
			profile = null;
			return;
		}
		getMojangProfile()
			.then((p) => {
				profile = p;
				syncViewer();
			})
			.catch(() => {
				profile = null;
			});
	});

	// Lanzar desde la home — misma lógica que el botón "Jugar" de
	// InstanceDetail.svelte (aviso de multi-instancia incluido), pero sin
	// tocar ese componente: solo comparten la función de decisión
	// (multiInstanceWarningKind, en $lib/state/launch.ts).
	let launching = $state(false);
	let launchError = $state<string | null>(null);
	let showMultiInstanceConfirm = $state(false);
	let multiInstanceDontAskAgain = $state(false);

	const otherRunning = $derived(
		featured ? gameSession.running.filter((r) => r.name !== featured.name) : []
	);

	async function doLaunch() {
		if (!featured) return;
		launching = true;
		launchError = null;
		try {
			await launchInstance(featured.name);
		} catch (e) {
			launchError = String(e);
		} finally {
			launching = false;
		}
	}

	async function handlePlay() {
		if (!featured) return;
		const kind = multiInstanceWarningKind(otherRunning, appState.currentUser?.uuid);
		if (kind && appState.settings?.multi_instance_warning_dismissed !== true) {
			showMultiInstanceConfirm = true;
			return;
		}
		await doLaunch();
	}

	async function confirmMultiInstance() {
		showMultiInstanceConfirm = false;
		if (multiInstanceDontAskAgain && appState.settings) {
			try {
				await updateSettings({ ...appState.settings, multi_instance_warning_dismissed: true });
				appState.settings.multi_instance_warning_dismissed = true;
			} catch {
				// si falla guardar la preferencia, no vale la pena bloquear el
				// lanzamiento por eso — se va a volver a preguntar la próxima.
			}
		}
		await doLaunch();
	}
</script>

{#if clientInstances.length === 0}
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
				<Zap size={16} /><span>{t('home.javaCardTitle')}</span><small
					>{t('home.javaCardHint')}</small
				>
			</div>
		</div>
	</div>
{:else}
	<div class="home-rich">
		<div class="character-panel">
			{#if isPremium}
				<canvas bind:this={canvasEl} width="220" height="320"></canvas>
			{:else if appState.currentUser}
				<div class="mascot-big">
					<Mascot id={getMascotFor(appState.currentUser.uuid)} size={160} />
				</div>
			{:else}
				<div class="welcome-orb big"><Tfl width="42" height="42" /></div>
			{/if}
		</div>

		<div class="home-main">
			{#if featured}
				<div class="continue-card">
					<span class="continue-label">{t('home.continuePlaying')}</span>
					<div class="continue-row">
						<span class="continue-icon">
							{#if iconUrls[featured.uuid]}
								<img src={iconUrls[featured.uuid]} alt="" />
							{:else}
								{featured.name.charAt(0).toUpperCase()}
							{/if}
						</span>
						<div class="continue-info">
							<span class="continue-name">{featured.name}</span>
							<span class="continue-meta">
								{featured.mc_version} · {featured.loader}
								{#if playTimeLabel}
									· {playTimeLabel}
								{/if}
							</span>
						</div>
						<button type="button" class="play-btn" disabled={launching} onclick={handlePlay}>
							<Play size={15} />
							{launching ? t('home.launching') : t('home.play')}
						</button>
					</div>
					{#if launchError}<p class="launch-error">{launchError}</p>{/if}
				</div>
			{/if}

			{#if others.length > 0}
				<div class="others-block">
					<span class="others-label">{t('home.otherInstances')}</span>
					<div class="others-grid">
						{#each others as instance (instance.uuid)}
							<button type="button" class="other-card" onclick={() => selectInstance(instance)}>
								<span class="other-icon">
									{#if iconUrls[instance.uuid]}
										<img src={iconUrls[instance.uuid]} alt="" />
									{:else}
										{instance.name.charAt(0).toUpperCase()}
									{/if}
								</span>
								<span class="other-name">{instance.name}</span>
							</button>
						{/each}
					</div>
				</div>
			{/if}

			<div class="quick-actions">
				<button type="button" class="secondary-cta" onclick={() => openModal('createChooser')}>
					<Plus size={15} strokeWidth={2.5} />
					{t('home.createInstance')}
				</button>
				<button type="button" class="secondary-cta" onclick={() => openModal('tflSelection')}>
					<Sparkles size={15} />
					{t('home.exploreTflSelection')}
				</button>
			</div>
		</div>
	</div>

	{#if showMultiInstanceConfirm}
		<ConfirmDialog
			title={t('instanceDetail.multiInstanceTitle')}
			message={multiInstanceWarningKind(otherRunning, appState.currentUser?.uuid) === 'same-account'
				? t('instanceDetail.multiInstanceSameAccount')
				: t('instanceDetail.multiInstanceDifferentAccount')}
			confirmLabel={t('instanceDetail.multiInstanceProceed')}
			checkboxLabel={t('instanceDetail.multiInstanceDontAskAgain')}
			bind:checked={multiInstanceDontAskAgain}
			onConfirm={confirmMultiInstance}
			onCancel={() => (showMultiInstanceConfirm = false)}
		/>
	{/if}
{/if}

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

	.welcome-orb.big {
		width: 160px;
		height: 160px;
		border-radius: 40px;
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

	/* Home rediseñada (con instancias) — personaje grande a un lado, card de
	   "seguir jugando" lista para usar, estilo LabyMod pedido por el
	   cliente. */
	.home-rich {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 48px;
		height: 100%;
		max-width: 920px;
		margin: auto;
		padding: 48px 28px;
		flex-wrap: wrap;
	}

	.character-panel {
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.character-panel canvas {
		filter: drop-shadow(0 20px 40px rgba(0, 0, 0, 0.45));
	}

	.mascot-big {
		filter: drop-shadow(0 20px 40px rgba(0, 0, 0, 0.45));
	}

	.home-main {
		display: flex;
		flex-direction: column;
		gap: 20px;
		width: min(100%, 420px);
	}

	.continue-card {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 18px;
		border-radius: var(--border-radius-lg);
		border: 1px solid var(--border);
		background: color-mix(in srgb, var(--bg-card) 90%, transparent);
		box-shadow: var(--shadow-md);
	}

	.continue-label {
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.continue-row {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.continue-icon {
		flex-shrink: 0;
		width: 48px;
		height: 48px;
		border-radius: var(--border-radius);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		font-size: 1.1rem;
		font-weight: 800;
		color: var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
	}
	.continue-icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.continue-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}
	.continue-name {
		font-size: 1rem;
		font-weight: 700;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.continue-meta {
		font-size: 0.72rem;
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.play-btn {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		flex-shrink: 0;
		background: var(--accent);
		color: var(--accent-text);
		border: none;
		padding: 10px 18px;
		border-radius: var(--border-radius-sm);
		font-weight: 700;
		font-size: 0.82rem;
		cursor: pointer;
		box-shadow: var(--shadow-md);
		transition: transform 0.12s ease;
	}
	.play-btn:hover:not(:disabled) {
		transform: translateY(-1px);
	}
	.play-btn:disabled {
		opacity: 0.7;
		cursor: default;
	}

	.launch-error {
		font-size: 0.74rem;
		color: var(--color-error);
	}

	.others-block {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.others-label {
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
	}
	.others-grid {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
	.other-card {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px 8px 8px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: color-mix(in srgb, var(--bg-card) 88%, transparent);
		color: var(--text-primary);
		cursor: pointer;
		max-width: 160px;
	}
	.other-card:hover {
		border-color: color-mix(in srgb, var(--accent) 40%, var(--border));
	}
	.other-icon {
		flex-shrink: 0;
		width: 26px;
		height: 26px;
		border-radius: var(--border-radius-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		font-size: 0.7rem;
		font-weight: 800;
		color: var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
	}
	.other-icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.other-name {
		font-size: 0.78rem;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.quick-actions {
		display: flex;
		gap: 10px;
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
		.home-rich {
			flex-direction: column;
			gap: 24px;
		}
	}
</style>
