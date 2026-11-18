<script lang="ts">
	import { selectionFallbackIcon } from '$lib/selection-identity';
	import { react } from '$lib/scenes/reactions.svelte';
	import { openSettingsAt } from '$lib/state/uiState.svelte';
	import { tr } from '$lib/i18n/experience';
	import { Pencil } from 'lucide-svelte';
	import { visual } from '$lib/visual/preferences.svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { ConfirmDialog, Tooltip } from '$lib/components/ui';
	import HomeEmptyState from '$lib/components/home/HomeEmptyState.svelte';
	import InstanceHero from '$lib/components/home/InstanceHero.svelte';
	import FavoriteServersPanel from '$lib/components/home/FavoriteServersPanel.svelte';
	import RecentActivityPanel from '$lib/components/home/RecentActivityPanel.svelte';
	import SystemStatusPanel from '$lib/components/home/SystemStatusPanel.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { openModal } from '$lib/state/uiState.svelte';
	import { appState } from '$lib/state/state.svelte';
	import { gameSession } from '$lib/state/gameSession.svelte';
	import { iconVersions } from '$lib/state/instanceState.svelte';
	import { multiInstanceWarningKind } from '$lib/state/launch';
	import {
		launchInstance,
		getCharacterProfile,
		getInstanceIconPath,
		patchSettings,
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
		const candidates = [featured].filter((i): i is InstanceData => i !== null);
		for (const inst of candidates) {
			const version = iconVersions[inst.name] ?? 0;
			const key = `${inst.name}:${version}`;
			if (iconLoaded[inst.uuid] === key) continue;
			iconLoaded[inst.uuid] = key;
			getInstanceIconPath(inst.name)
				.then((path) => {
					iconUrls[inst.uuid] = path
						? `${convertFileSrc(path)}?v=${version}`
						: selectionFallbackIcon(inst.selection);
				})
				.catch(() => {
					iconUrls[inst.uuid] = selectionFallbackIcon(inst.selection);
				});
		}
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
	let loadedProfile = $state<{ uuid: string; profile: MojangProfile } | null>(null);
	// Identity is checked during rendering too: effects run after the DOM update,
	// so merely clearing the old profile in an effect can show the wrong skin.
	const profile = $derived(
		loadedProfile?.uuid === appState.currentUser?.uuid ? (loadedProfile?.profile ?? null) : null
	);
	$effect(() => {
		const account = appState.currentUser;
		let stale = false;
		loadedProfile = null;
		if (!account) return;
		getCharacterProfile(account.uuid)
			.then((p) => {
				if (!stale) loadedProfile = { uuid: account.uuid, profile: p };
			})
			.catch(() => {
				if (!stale) loadedProfile = null;
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
		react('launch');
		launchError = null;
		try {
			await launchInstance(featured.name);
		} catch (e) {
			launchError = String(e);
			react('error');
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
				await patchSettings({ multi_instance_warning_dismissed: true });
				appState.settings.multi_instance_warning_dismissed = true;
			} catch {
				// si falla guardar la preferencia, no vale la pena bloquear el
				// lanzamiento por eso — se va a volver a preguntar la próxima.
			}
		}
		await doLaunch();
	}
</script>

<div class="home-shell">
	{#if clientInstances.length === 0}
		<HomeEmptyState />
	{:else}
		<div class="home-rich vr-stagger">
			{#if featured}
				<InstanceHero
					{featured}
					iconUrl={iconUrls[featured.uuid] ?? null}
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
		</div>

		{#if showMultiInstanceConfirm}
			<ConfirmDialog
				title={t('instanceDetail.multiInstanceTitle')}
				message={multiInstanceWarningKind(otherRunning, appState.currentUser?.uuid) ===
				'same-account'
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

	<div class="home-customize">
		<Tooltip content={tr('Personalizar Inicio', 'Customize Home')} position="left">
			<button
				type="button"
				class="vr-control"
				aria-label={tr('Personalizar Inicio', 'Customize Home')}
				onclick={() => openSettingsAt('home')}><Pencil size={16} /></button
			>
		</Tooltip>
	</div>
</div>

<style>
	.home-shell {
		position: relative;
		min-height: 100%;
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
	.home-customize {
		position: absolute;
		top: 10px;
		right: 12px;
		z-index: 2;
	}
	.home-customize button {
		width: 34px;
		height: 34px;
		padding: 8px;
		background: var(--surface-card);
		box-shadow: var(--shadow-sm);
	}
	.home-customize button:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 3px;
	}
</style>
