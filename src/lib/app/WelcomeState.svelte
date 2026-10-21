<script lang="ts">
	import { ArrowUpRight } from 'lucide-svelte';
	import { visual } from '$lib/visual/preferences.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { navigate } from '$lib/visual/navigation.svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { Button, ConfirmDialog } from '$lib/components/ui';
	import HomeEmptyState from '$lib/components/home/HomeEmptyState.svelte';
	import InstanceHero from '$lib/components/home/InstanceHero.svelte';
	import FavoriteServersPanel from '$lib/components/home/FavoriteServersPanel.svelte';
	import RecentActivityPanel from '$lib/components/home/RecentActivityPanel.svelte';
	import SystemStatusPanel from '$lib/components/home/SystemStatusPanel.svelte';
	import InstanceCarousel from '$lib/components/home/InstanceCarousel.svelte';
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
	import { Plus, Sparkles } from 'lucide-svelte';

	/**
	 * Home — Entrega C. Este archivo queda como ORQUESTADOR (sección 1/17
	 * del encargo): todo el fetching de datos que ya tenía (efectos de
	 * abajo, mismas llamadas de `$lib/api` con la misma firma, cero
	 * duplicados) sigue viviendo acá tal cual estaba; lo que se extrajo a
	 * `$lib/components/home/` es el TEMPLATE/estilos de cada sección, que
	 * ahora recibe los datos ya cargados por props. No se tocó ninguna
	 * lógica de negocio (lanzamiento, aviso multi-instancia, cálculo de
	 * "destacada") — ver el diff contra la versión anterior de este mismo
	 * archivo para confirmarlo línea por línea si hace falta.
	 */

	/** "Hoy" / "Ayer" / "Hace N días" — mismo cálculo que
	 * `instanceDetail.lastPlayedLabel`, generalizado acá porque se usa para
	 * el hero Y para cada fila de actividad reciente. */
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
	// `null` → "nunca jugada" (sección 4 del encargo: nada de fecha
	// inventada). InstanceHero decide qué mostrar en ese caso.
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

	// Fondo del hero: la captura más reciente de la instancia destacada (si
	// tiene alguna) — nada de arte inventado. Sin capturas, InstanceHero
	// cae en su degradado de marca (ver CSS ahí).
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
	// `systemStatusLoading` distingue "todavía no respondió" de "respondió
	// null/falló" — SystemStatusPanel necesita esto para no mostrarse mudo
	// ni como error mientras el pedido sigue en vuelo (sección 6).
	let systemStatus = $state<SystemStatus | null>(null);
	let systemStatusLoading = $state(true);
	function loadSystemStatus() {
		systemStatusLoading = true;
		getSystemStatus()
			.then((s) => {
				systemStatus = s;
			})
			.catch(() => {
				systemStatus = null;
			})
			.finally(() => {
				systemStatusLoading = false;
			});
	}
	$effect(() => {
		loadSystemStatus();
	});

	// Personaje grande (pedido del cliente, estilo LabyMod). El ciclo de
	// vida de skinview3d vive en `$lib/components/ui/SkinViewer3D.svelte`
	// (compartido con SkinManagerModal.svelte) — acá solo queda el fetch
	// del perfil de Mojang, con la misma firma de antes.
	const isPremium = $derived(
		appState.currentUser !== null && appState.currentUser.user_type !== 'Cracked'
	);
	let profile = $state<MojangProfile | null>(null);
	$effect(() => {
		void appState.currentUser?.uuid;
		let stale = false;
		profile = null;
		if (!isPremium) {
			profile = null;
			return;
		}
		getMojangProfile()
			.then((p) => {
				if (!stale) profile = p;
			})
			.catch(() => {
				if (!stale) profile = null;
			});
		return () => {
			stale = true;
		};
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
	// Presentación pura sobre un dato que ya se pedía (`gameSession.running`,
	// usado más arriba para `otherRunning`) — ninguna lógica nueva, solo le
	// da a InstanceHero con qué mostrar el badge "Corriendo" (sección 6).
	const featuredIsRunning = $derived(
		featured ? gameSession.running.some((r) => r.name === featured.name) : false
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
	<HomeEmptyState
		onCreateInstance={() => openModal('createChooser')}
		onExploreTflSelection={() => navigate('discover')}
	/>
{:else}
	<div class="home-rich vr-stagger">
		<header class="home-greeting">
			<span>{vr('welcome')}, <strong>{appState.currentUser?.username ?? 'TFL'}</strong>.</span
			><button onclick={() => navigate('library')}
				>{vr('library')} <ArrowUpRight size={15} /></button
			>
		</header>
		{#if featured}
			<InstanceHero
				{featured}
				iconUrl={iconUrls[featured.uuid] ?? null}
				{bannerBg}
				{lastPlayedLabel}
				{playTimeLabel}
				isRunning={featuredIsRunning}
				{launching}
				{launchError}
				onPlay={handlePlay}
				{isPremium}
				accountUuid={appState.currentUser?.uuid ?? null}
				{profile}
				onOpenSkinManager={() => openModal('skinManager')}
			/>
		{/if}

		<div class="dash-grid">
			{#if visual.home.servers}<FavoriteServersPanel
					{servers}
					onSeeAll={() => openModal('joinServer')}
				/>{/if}
			{#if visual.home.activity}<RecentActivityPanel
					{activity}
					iconUrls={activityIconUrls}
					{relativeDayLabel}
					{durationLabel}
				/>{/if}
			{#if visual.home.system}<SystemStatusPanel
					status={systemStatus}
					loading={systemStatusLoading}
					onRetry={loadSystemStatus}
				/>{/if}
		</div>

		{#if visual.home.recent && others.length > 0}
			<InstanceCarousel instances={others} {iconUrls} onSelect={selectInstance} />
		{/if}

		<div class="quick-actions">
			<Button variant="secondary" onclick={() => openModal('createChooser')}>
				{#snippet icon()}<Plus size={15} strokeWidth={2.5} />{/snippet}
				{t('home.createInstance')}
			</Button>
			<Button variant="secondary" onclick={() => openModal('tflSelection')}>
				{#snippet icon()}<Sparkles size={15} />{/snippet}
				{t('home.exploreTflSelection')}
			</Button>
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
	.home-greeting {
		display: flex;
		align-items: center;
		justify-content: space-between;
		color: var(--text-secondary);
		font-size: var(--text-small);
		margin-bottom: 4px;
		gap: 16px;
	}
	.home-greeting strong {
		color: var(--text-primary);
	}
	.home-greeting button {
		background: none;
		border: 0;
		color: var(--text-secondary);
		font-size: var(--text-small);
		padding: 8px;
	}
	/* Home rediseñada (con instancias) — hero unificado (instancia +
	   personaje + Play, ver InstanceHero.svelte) arriba, paneles
	   secundarios compactos abajo, estilo LabyMod/Prism pedido por el
	   cliente (sección 7-8 del encargo). */
	.home-rich {
		display: flex;
		flex-direction: column;
		gap: 18px;
		min-height: 100%;
		max-width: 1500px;
		margin: auto;
		padding: 28px clamp(20px, 3vw, 44px);
		overflow: visible;
	}

	/* Grilla de paneles secundarios: servidores, actividad, estado del
	   sistema — se acomodan en columnas según el ancho disponible. */
	.dash-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 14px;
	}

	.quick-actions {
		display: flex;
		gap: 10px;
	}

	@media (max-width: 640px) {
		.quick-actions {
			flex-direction: column;
		}
	}

	/* Entrega D, sección 5.3: "transiciones de navegación, aparición... de
	   paneles" tiene que notarse desde el primer contacto en Experience, no
	   ser cosmético-mínimo — hoy (post A/B/C) la Home no diferenciaba nada
	   entre perfiles más allá de lo que ya hacían SkinViewer3D/Button/etc.
	   Entrada en cascada de los bloques de nivel superior (hero, grilla de
	   paneles, carrusel, accesos rápidos) solo en Experience — Balanced ya
	   tiene su propio feedback de hover (Button/IconButton de Entrega A) y
	   Lite no debe tener movimiento decorativo (sección 5.1). CSS puro vía
	   atributo en <html> (sección 5.5: nada de un nuevo `if (quality===...)`
	   en este script) y usa --duration-panel/--easing-decelerate en vez de
	   un valor propio, así reduce-motion (ver global.css, guard sin
	   excepción para Experience tras el fix de esta misma entrega) y el
	   perfil de calidad lo siguen controlando igual que a cualquier otra
	   animación. */
	:global(html[data-quality='experience']) .home-rich > :global(*) {
		animation: tfl-home-panel-in var(--duration-panel) var(--easing-decelerate) both;
	}
	:global(html[data-quality='experience']) .home-rich > :global(*:nth-child(2)) {
		animation-delay: 60ms;
	}
	:global(html[data-quality='experience']) .home-rich > :global(*:nth-child(3)) {
		animation-delay: 120ms;
	}
	:global(html[data-quality='experience']) .home-rich > :global(*:nth-child(4)) {
		animation-delay: 180ms;
	}

	@keyframes tfl-home-panel-in {
		from {
			opacity: 0;
			transform: translateY(6px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
