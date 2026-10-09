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
	import {
		launchInstance,
		getMojangProfile,
		getInstanceIconPath,
		updateSettings,
		getInstanceScreenshots,
		getFavoriteServers,
		getRecentActivity,
		getSystemStatus
	} from '$lib/api';
	import type {
		InstanceData,
		MojangProfile,
		FavoriteServer,
		ActivityEntry,
		SystemStatus
	} from '$lib/types/types';
	import {
		Plus,
		Sparkles,
		PackageOpen,
		Zap,
		Play,
		Shirt,
		Server as ServerIcon,
		History,
		Coffee,
		Cpu,
		MemoryStick,
		HardDrive,
		ChevronLeft,
		ChevronRight
	} from 'lucide-svelte';

	/** "Hoy" / "Ayer" / "Hace N días" — mismo cálculo que
	 * `instanceDetail.lastPlayedLabel`, generalizado acá porque se usa para
	 * el banner Y para cada fila de actividad reciente. */
	function relativeDayLabel(unixSecs: number): string {
		const diffMs = Date.now() - unixSecs * 1000;
		const days = Math.floor(diffMs / 86_400_000);
		if (days <= 0) return t('instanceDetail.playedToday');
		if (days === 1) return t('instanceDetail.playedYesterday');
		return t('instanceDetail.playedDaysAgo', { days });
	}

	function durationLabel(secs: number): string {
		const h = Math.floor(secs / 3600);
		const m = Math.floor((secs % 3600) / 60);
		return h > 0
			? t('instanceDetail.playTimeHours', { h, m })
			: t('instanceDetail.playTimeMinutes', { m });
	}

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
		return durationLabel(secs);
	});
	const lastPlayedLabel = $derived(
		featured && featured.last_played ? relativeDayLabel(featured.last_played) : null
	);

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

	// Fondo del banner: la captura más reciente de la instancia destacada (si
	// tiene alguna) — nada de arte inventado. Sin capturas, el banner se ve
	// con el degradado de siempre (ver CSS de .banner).
	let bannerBg = $state<string | null>(null);
	$effect(() => {
		bannerBg = null;
		if (!featured) return;
		const name = featured.name;
		getInstanceScreenshots(name)
			.then((shots) => {
				if (featured?.name !== name || shots.length === 0) return;
				const latest = shots.reduce((a, b) => (b.modified_ms > a.modified_ms ? b : a));
				bannerBg = convertFileSrc(latest.path);
			})
			.catch(() => {
				bannerBg = null;
			});
	});

	// Servidores favoritos — no hay forma de saber a qué servidor se unió
	// alguien DENTRO del juego, así que esto son los guardados a mano
	// (mismos que "Unirse a servidor"), no un historial real de conexiones.
	let servers = $state<FavoriteServer[]>([]);
	$effect(() => {
		getFavoriteServers()
			.then((list) => {
				servers = list.slice(0, 4);
			})
			.catch(() => {
				servers = [];
			});
	});

	// Actividad reciente — sesiones de juego terminadas, lo más nuevo
	// primero (ver `services::activity_log` en el backend).
	let activity = $state<ActivityEntry[]>([]);
	$effect(() => {
		getRecentActivity(5)
			.then((list) => {
				activity = list;
			})
			.catch(() => {
				activity = [];
			});
	});
	// Mapa separado del de arriba (ese usa uuid; acá solo se tiene el nombre
	// guardado en el log de actividad, y la instancia pudo haberse borrado).
	let activityIconUrls = $state<Record<string, string | null>>({});
	const activityIconLoaded: Record<string, true> = {};
	$effect(() => {
		for (const a of activity) {
			if (activityIconLoaded[a.instance_name]) continue;
			activityIconLoaded[a.instance_name] = true;
			getInstanceIconPath(a.instance_name)
				.then((path) => {
					activityIconUrls[a.instance_name] = path ? convertFileSrc(path) : null;
				})
				.catch(() => {
					activityIconUrls[a.instance_name] = null;
				});
		}
	});

	// Estado del sistema — solo números (Java/RAM/Disco), sin heurística de
	// "problema detectado": eso lo decide quien mira los números.
	let systemStatus = $state<SystemStatus | null>(null);
	$effect(() => {
		getSystemStatus()
			.then((s) => {
				systemStatus = s;
			})
			.catch(() => {
				systemStatus = null;
			});
	});
	function formatGb(gb: number): string {
		return `${gb.toFixed(gb < 10 ? 1 : 0)} GB`;
	}

	// Carrusel de "otras instancias" — scroll horizontal nativo, los botones
	// solo mueven el scroll (sin librería nueva).
	let carouselEl = $state<HTMLDivElement | undefined>(undefined);
	function scrollCarousel(dir: 1 | -1) {
		carouselEl?.scrollBy({ left: dir * 220, behavior: 'smooth' });
	}

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
		{#if featured}
			<div
				class="banner"
				class:has-bg={!!bannerBg}
				style={bannerBg ? `--banner-bg: url(${bannerBg})` : ''}
			>
				<span class="banner-eyebrow">{t('home.continuePlaying')}</span>
				<div class="banner-row">
					<span class="banner-icon">
						{#if iconUrls[featured.uuid]}
							<img src={iconUrls[featured.uuid]} alt="" />
						{:else}
							{featured.name.charAt(0).toUpperCase()}
						{/if}
					</span>
					<div class="banner-info">
						<span class="banner-name">{featured.name}</span>
						<span class="banner-meta">
							{featured.mc_version} · {featured.loader}
							{#if lastPlayedLabel}
								· {lastPlayedLabel}
							{/if}
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

		<div class="dash-grid">
			<div class="panel character-panel">
				<div class="character-figure">
					{#if isPremium}
						<canvas bind:this={canvasEl} width="150" height="220"></canvas>
					{:else if appState.currentUser}
						<div class="mascot-big">
							<Mascot id={getMascotFor(appState.currentUser.uuid)} size={110} />
						</div>
					{:else}
						<div class="welcome-orb big"><Tfl width="36" height="36" /></div>
					{/if}
				</div>
				{#if isPremium}
					<div class="skins-row">
						{#each (profile?.skins ?? []).slice(0, 3) as skin (skin.id)}
							<button
								type="button"
								class="skin-slot"
								title={t('home.openSkinManager')}
								onclick={() => openModal('skinManager')}
							>
								<span class="skin-layer base" style="background-image: url({skin.url})"></span>
								<span class="skin-layer overlay" style="background-image: url({skin.url})"></span>
							</button>
						{/each}
						<button
							type="button"
							class="skin-slot skin-slot-add"
							title={t('home.openSkinManager')}
							onclick={() => openModal('skinManager')}
						>
							<Shirt size={14} />
						</button>
					</div>
				{/if}
			</div>

			<div class="panel">
				<div class="panel-title">
					<ServerIcon size={13} />
					<span>{t('home.yourServers')}</span>
					<button type="button" class="panel-link" onclick={() => openModal('joinServer')}>
						{t('home.seeAll')}
					</button>
				</div>
				{#if servers.length === 0}
					<p class="panel-empty">{t('home.noServersSaved')}</p>
				{:else}
					<ul class="panel-list">
						{#each servers as server (server.id)}
							<li class="server-row">
								<span class="server-name">{server.name}</span>
								<span class="server-address">{server.address}</span>
							</li>
						{/each}
					</ul>
				{/if}
			</div>

			<div class="panel">
				<div class="panel-title">
					<History size={13} />
					<span>{t('home.recentActivity')}</span>
				</div>
				{#if activity.length === 0}
					<p class="panel-empty">{t('home.noActivityYet')}</p>
				{:else}
					<ul class="panel-list">
						{#each activity as entry (entry.id)}
							<li class="activity-row">
								<span class="activity-icon">
									{#if activityIconUrls[entry.instance_name]}
										<img src={activityIconUrls[entry.instance_name]} alt="" />
									{:else}
										{entry.instance_name.charAt(0).toUpperCase()}
									{/if}
								</span>
								<div class="activity-info">
									<span class="activity-name">{entry.instance_name}</span>
									<span class="activity-meta"
										>{relativeDayLabel(entry.ended_at)} · {durationLabel(entry.duration_secs)}</span
									>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</div>

			<div class="panel">
				<div class="panel-title">
					<Cpu size={13} />
					<span>{t('home.systemStatus')}</span>
				</div>
				{#if systemStatus}
					<ul class="status-list">
						<li>
							<Coffee size={13} />
							<span
								>{systemStatus.java_major
									? t('home.javaInstalled', { major: systemStatus.java_major })
									: t('home.javaMissing')}</span
							>
						</li>
						<li>
							<MemoryStick size={13} />
							<span
								>{t('home.ramStatus', {
									assigned: (systemStatus.ram_assigned_mb / 1024).toFixed(1),
									total: (systemStatus.ram_total_mb / 1024).toFixed(1)
								})}</span
							>
						</li>
						<li>
							<HardDrive size={13} />
							<span
								>{t('home.diskStatus', {
									free: formatGb(systemStatus.disk_free_gb),
									total: formatGb(systemStatus.disk_total_gb)
								})}</span
							>
						</li>
					</ul>
				{/if}
			</div>
		</div>

		{#if others.length > 0}
			<div class="others-block">
				<div class="others-head">
					<span class="others-label">{t('home.otherInstances')}</span>
					<div class="others-nav">
						<button
							type="button"
							class="carousel-btn"
							onclick={() => scrollCarousel(-1)}
							aria-label={t('home.scrollLeft')}
						>
							<ChevronLeft size={14} />
						</button>
						<button
							type="button"
							class="carousel-btn"
							onclick={() => scrollCarousel(1)}
							aria-label={t('home.scrollRight')}
						>
							<ChevronRight size={14} />
						</button>
					</div>
				</div>
				<div class="others-grid" bind:this={carouselEl}>
					{#each others as instance (instance.uuid)}
						<button type="button" class="other-card" onclick={() => selectInstance(instance)}>
							<span class="other-icon">
								{#if iconUrls[instance.uuid]}
									<img src={iconUrls[instance.uuid]} alt="" />
								{:else}
									{instance.name.charAt(0).toUpperCase()}
								{/if}
							</span>
							<span class="other-text">
								<span class="other-name">{instance.name}</span>
								<span class="other-meta">{instance.mc_version} · {instance.loader}</span>
							</span>
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
		flex-direction: column;
		gap: 18px;
		height: 100%;
		max-width: 1040px;
		margin: auto;
		padding: 36px 28px;
		overflow-y: auto;
	}

	/* Banner "seguir jugando" — fondo real (última captura de la instancia)
	   si hay una, degradado si no. El degradado oscuro encima es lo que
	   garantiza que el texto se lea con cualquier captura de fondo. */
	.banner {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 20px 22px;
		border-radius: var(--border-radius-lg);
		border: 1px solid var(--border);
		background:
			linear-gradient(
				120deg,
				color-mix(in srgb, var(--bg-card) 92%, transparent) 0%,
				color-mix(in srgb, var(--accent) 10%, var(--bg-card) 92%) 100%
			),
			var(--bg-card);
		box-shadow: var(--shadow-md);
		background-size: cover;
		background-position: center;
	}
	.banner.has-bg {
		background-image:
			linear-gradient(0deg, rgba(0, 0, 0, 0.78), rgba(0, 0, 0, 0.42)), var(--banner-bg, none);
	}

	.banner-eyebrow {
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: color-mix(in srgb, var(--accent) 70%, white 10%);
	}

	.banner-row {
		display: flex;
		align-items: center;
		gap: 14px;
	}

	.banner-icon {
		flex-shrink: 0;
		width: 52px;
		height: 52px;
		border-radius: var(--border-radius);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		font-size: 1.2rem;
		font-weight: 800;
		color: var(--accent);
		background: color-mix(in srgb, var(--accent) 20%, var(--bg-card));
	}
	.banner-icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.banner-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}
	.banner-name {
		font-size: 1.15rem;
		font-weight: 800;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.banner-meta {
		font-size: 0.75rem;
		color: var(--text-secondary);
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
		padding: 11px 20px;
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

	/* Grilla de paneles: personaje+skins, servidores, actividad, estado del
	   sistema — se acomodan en columnas según el ancho disponible. */
	.dash-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 14px;
	}

	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: color-mix(in srgb, var(--bg-card) 88%, transparent);
		min-width: 0;
	}

	.panel-title {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.4px;
		text-transform: uppercase;
		color: var(--text-muted);
	}
	.panel-title :global(svg) {
		color: var(--accent);
		flex-shrink: 0;
	}
	.panel-link {
		margin-left: auto;
		background: none;
		border: none;
		padding: 0;
		color: var(--accent);
		font-size: 0.68rem;
		font-weight: 700;
		text-transform: none;
		letter-spacing: normal;
		cursor: pointer;
	}
	.panel-link:hover {
		text-decoration: underline;
	}

	.panel-empty {
		font-size: 0.74rem;
		color: var(--text-muted);
	}

	.panel-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.character-panel {
		align-items: center;
	}
	.character-figure {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 150px;
	}
	.character-figure canvas {
		filter: drop-shadow(0 14px 28px rgba(0, 0, 0, 0.45));
	}
	.mascot-big {
		filter: drop-shadow(0 14px 28px rgba(0, 0, 0, 0.45));
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
		background: var(--bg-input);
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

	.server-row {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.server-name {
		font-size: 0.8rem;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.server-address {
		font-size: 0.68rem;
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.activity-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.activity-icon {
		flex-shrink: 0;
		width: 26px;
		height: 26px;
		border-radius: var(--border-radius-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		font-size: 0.68rem;
		font-weight: 800;
		color: var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
	}
	.activity-icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.activity-info {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}
	.activity-name {
		font-size: 0.78rem;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.activity-meta {
		font-size: 0.66rem;
		color: var(--text-muted);
	}

	.status-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.status-list li {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.74rem;
		color: var(--text-secondary);
	}
	.status-list :global(svg) {
		flex-shrink: 0;
		color: var(--text-muted);
	}

	.others-block {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.others-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.others-label {
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
	}
	.others-nav {
		display: flex;
		gap: 4px;
	}
	.carousel-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-card);
		color: var(--text-muted);
		cursor: pointer;
	}
	.carousel-btn:hover {
		color: var(--text-primary);
	}
	.others-grid {
		display: flex;
		gap: 10px;
		overflow-x: auto;
		scroll-snap-type: x proximity;
		padding-bottom: 4px;
		scrollbar-width: thin;
	}
	.other-card {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px 10px 10px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: color-mix(in srgb, var(--bg-card) 88%, transparent);
		color: var(--text-primary);
		cursor: pointer;
		flex-shrink: 0;
		min-width: 170px;
		scroll-snap-align: start;
	}
	.other-card:hover {
		border-color: color-mix(in srgb, var(--accent) 40%, var(--border));
	}
	.other-icon {
		flex-shrink: 0;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		font-size: 0.75rem;
		font-weight: 800;
		color: var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
	}
	.other-icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.other-text {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}
	.other-name {
		font-size: 0.78rem;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.other-meta {
		font-size: 0.66rem;
		color: var(--text-muted);
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
		.banner-row {
			flex-wrap: wrap;
		}
		.play-btn {
			width: 100%;
			justify-content: center;
		}
	}
</style>
