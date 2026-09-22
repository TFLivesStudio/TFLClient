<script lang="ts">
	import { onMount } from 'svelte';
	import TitleBar from '$lib/components/layout/TitleBar/TitleBar.svelte';
	import Sidebar from '$lib/components/layout/Sidebar/Sidebar.svelte';
	import Onboarding from '$lib/components/onboarding/Onboarding.svelte';
	import CreateInstanceModal from '$lib/components/library/CreateInstanceModal.svelte';
	import InstanceDetail from '$lib/components/library/InstanceDetail.svelte';
	import InstanceLogWindow from '$lib/components/library/InstanceLogWindow.svelte';
	import ParticlesBackground from '$lib/components/layout/ParticlesBackground.svelte';
	import CommandPalette from '$lib/components/layout/CommandPalette.svelte';
	import WhatsNewTips from '$lib/components/onboarding/WhatsNewTips.svelte';
	import DownloadProgressBar from '$lib/components/library/DownloadProgressBar.svelte';
	import SettingsPanel from '$lib/components/settings/SettingsPanel.svelte';
	import TflSelection from '$lib/components/library/TflSelection.svelte';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { appState } from '$lib/state/state.svelte';
	import { initDownloadListener } from '$lib/state/downloadState.svelte';
	import { initGameSessionListener } from '$lib/state/gameSession.svelte';
	import { initNetworkListener } from '$lib/state/network.svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import {
		getCurrentUser,
		getInstances,
		getSettings,
		getCustomWallpaperPath,
		logout as apiLogout
	} from '$lib/api/tflApi';
	import type { InstanceData, MinecraftUser } from '$lib/types/types';
	import { Plus, Sparkles, PackageOpen, Zap } from 'lucide-svelte';

	// Esta misma index.html también se usa para la ventana emergente del log
	// en vivo — Tauri la abre con un initialization_script que setea esta
	// variable global ANTES de que cargue cualquier script de la página
	// (no por query string: WebviewUrl::App toma el string entero como
	// path de archivo literal, no lo parsea como URL+query). Si está
	// presente, esta ventana es la de log — se salta toda la carga normal
	// del launcher (cuentas, instancias, ajustes…).
	const isLogWindow =
		typeof window !== 'undefined' &&
		typeof (window as unknown as { __TFL_LOG_INSTANCE__?: string }).__TFL_LOG_INSTANCE__ === 'string';

	let loading = $state(true);
	let showCreateModal = $state(false);
	let showSettings = $state(false);
	let showTflSelection = $state(false);

	// SettingsPanel cambia data-ambience directo en <html> (localStorage +
	// setAttribute), no hay ningún store — para saber acá cuándo mostrar
	// las partículas hace falta observar el atributo.
	let ambience = $state(
		typeof document !== 'undefined' ? (document.documentElement.getAttribute('data-ambience') ?? 'aurora') : 'aurora'
	);
	let reduceMotion = $state(false);

	async function refreshInstances() {
		appState.instances = await getInstances();
	}

	async function restoreAppearance() {
		const root = document.documentElement;
		const preferences = [
			['tfl-accent', 'data-accent', 'orange'],
			['tfl-surface', 'data-surface', 'obsidian'],
			['tfl-ambience', 'data-ambience', 'aurora'],
			['tfl-density', 'data-density', 'comfortable'],
			['tfl-wallpaper', 'data-wallpaper', 'none'],
			['tfl-card-style', 'data-card-style', 'rich']
		] as const;
		for (const [storageKey, attribute, defaultValue] of preferences) {
			let value = localStorage.getItem(storageKey) ?? defaultValue;
			// OLED no tiene variante clara. Estados viejos (de antes del fix
			// de exclusión mutua OLED/claro) pueden tener surface='oled' con
			// theme='light' guardado a la vez — se normaliza acá una sola
			// vez, en vez de arrastrar el estado inválido en cada arranque.
			if (attribute === 'data-surface' && value === 'oled' && root.getAttribute('data-theme') === 'light') {
				value = defaultValue;
				localStorage.setItem(storageKey, defaultValue);
			}
			if (value === defaultValue) root.removeAttribute(attribute);
			else root.setAttribute(attribute, value);
		}
		// El wallpaper propio no tiene regla fija en la hoja de estilos (la
		// URL es dinámica) — SettingsPanel lo setea inline al elegirlo, pero
		// eso no sobrevive un reinicio. Hay que reconstruirlo acá.
		if (localStorage.getItem('tfl-wallpaper') === 'custom') {
			try {
				const path = await getCustomWallpaperPath();
				if (path) root.style.setProperty('--wallpaper-bg', `url("${convertFileSrc(path)}")`);
			} catch {
				// sin wallpaper propio disponible, se queda con el fondo vacío
			}
		}
	}

	function applyQualityVisuals(settings: NonNullable<typeof appState.settings>) {
		const root = document.documentElement;
		root.setAttribute('data-quality', settings.quality_profile.toLowerCase());
		root.toggleAttribute('data-reduce-motion', settings.quality_profile === 'Lite');
		root.toggleAttribute('data-no-blur', settings.disable_blur_effects);
		// Único consumidor real de disable_infinite_animations (antes se
		// guardaba en el backend pero ninguna CSS lo leía — por eso Balanced
		// y Experience se veían idénticos). Con el atributo ausente, la capa
		// de resplandor pasivo de Experience puede correr (ver global.css).
		root.toggleAttribute('data-no-infinite-fx', settings.disable_infinite_animations);
	}

	onMount(async () => {
		if (isLogWindow) {
			loading = false;
			return;
		}

		const root = document.documentElement;
		const syncAppearanceState = () => {
			ambience = root.getAttribute('data-ambience') ?? 'aurora';
			reduceMotion = root.hasAttribute('data-reduce-motion');
		};
		syncAppearanceState();
		const appearanceObserver = new MutationObserver(syncAppearanceState);
		appearanceObserver.observe(root, {
			attributes: true,
			attributeFilter: ['data-ambience', 'data-reduce-motion']
		});

		initDownloadListener();
		initGameSessionListener();
		initNetworkListener();
		try {
			const [user, settings] = await Promise.all([getCurrentUser(), getSettings()]);
			appState.currentUser = user;
			appState.settings = settings;
			if (settings.theme === 'light' || settings.theme === 'dark') {
				document.documentElement.setAttribute('data-theme', settings.theme);
			}
			await restoreAppearance();
			applyQualityVisuals(settings);
			await refreshInstances();
		} finally {
			loading = false;
		}
	});

	async function handleOnboardingDone(user: MinecraftUser) {
		appState.currentUser = user;
		appState.settings = await getSettings();
		if (appState.settings) applyQualityVisuals(appState.settings);
	}

	async function handleLogout() {
		await apiLogout();
		appState.currentUser = await getCurrentUser();
		appState.settings = await getSettings();
	}

	function handleSelect(instance: InstanceData) {
		appState.selectedInstance = instance;
	}

	async function handleCreated(instance: InstanceData) {
		showCreateModal = false;
		await refreshInstances();
		appState.selectedInstance = instance;
	}

	async function handleInstanceChanged(instance: InstanceData | null) {
		await refreshInstances();
		appState.selectedInstance = instance;
	}

	const needsOnboarding = $derived(appState.settings?.onboarded !== true);
</script>

{#if isLogWindow}
	<InstanceLogWindow />
{:else}
<div class="app-shell">
	<TitleBar />

	{#if loading}
		<div class="loading-screen">
			<div class="loading-mark"><Tfl width="28" height="28" /></div>
		</div>
	{:else if needsOnboarding}
		<Onboarding onDone={handleOnboardingDone} />
	{:else}
		<div class="app-body">
			<div class="ambient-bg">
				{#if ambience === 'particles' && !reduceMotion}
					<ParticlesBackground />
				{/if}
			</div>
			<Sidebar
				instances={appState.instances}
				selected={appState.selectedInstance}
				user={appState.currentUser}
				onSelect={handleSelect}
				onCreate={() => (showCreateModal = true)}
				onLogout={handleLogout}
				onOpenSettings={() => (showSettings = true)}
				onOpenTflSelection={() => (showTflSelection = true)}
			/>
			<main class="main-content">
				{#if appState.selectedInstance}
					{#key appState.selectedInstance.uuid}
						<InstanceDetail
							instance={appState.selectedInstance}
							onChanged={handleInstanceChanged}
						/>
					{/key}
				{:else}
					<div class="empty-state">
						<div class="welcome-orb"><Tfl width="42" height="42" /></div>
						<div class="welcome-copy">
							<span class="eyebrow">TFL Client</span>
							<h2>Tu biblioteca Minecraft,<br />bien organizada.</h2>
							<p>Creá una instancia para jugar, instalar contenido y ajustar cada perfil.</p>
						</div>
						<div class="welcome-actions">
							<button type="button" class="empty-cta" onclick={() => (showCreateModal = true)}>
								<Plus size={16} strokeWidth={2.5} /> Crear instancia
							</button>
							<button type="button" class="secondary-cta" onclick={() => (showTflSelection = true)}>
								<Sparkles size={15} /> Explorar TFL Selection
							</button>
						</div>
						<div class="welcome-grid">
							<div class="welcome-card"><PackageOpen size={16} /><span>Mods y modpacks</span><small>Por instancia</small></div>
							<div class="welcome-card"><Zap size={16} /><span>Java automático</span><small>Sin configuración manual</small></div>
						</div>
					</div>
				{/if}
			</main>
		</div>
	{/if}
</div>
{/if}

{#if showCreateModal}
	<CreateInstanceModal onClose={() => (showCreateModal = false)} onCreated={handleCreated} />
{/if}

{#if !isLogWindow}
	<CommandPalette
		instances={appState.instances}
		blocked={showCreateModal || showSettings || showTflSelection}
		onSelectInstance={handleSelect}
		onCreate={() => (showCreateModal = true)}
		onOpenSettings={() => (showSettings = true)}
		onOpenTflSelection={() => (showTflSelection = true)}
	/>
{/if}

{#if !isLogWindow && !needsOnboarding}
	<WhatsNewTips />
{/if}

{#if showSettings}
	<SettingsPanel onClose={() => (showSettings = false)} />
{/if}

{#if showTflSelection}
	<TflSelection
		instances={appState.instances}
		onClose={() => (showTflSelection = false)}
		onInstanceCreated={() => refreshInstances()}
	/>
{/if}

<DownloadProgressBar />

<style>
	.app-shell {
		display: flex;
		flex-direction: column;
		height: 100vh;
		width: 100vw;
		background: var(--bg-main);
	}

	.loading-screen {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-main);
	}

	.loading-mark {
		color: var(--accent);
		opacity: 0.5;
		animation: tfl-pulse 1.6s ease-in-out infinite;
	}

	@keyframes tfl-pulse {
		0%,
		100% {
			opacity: 0.35;
			transform: scale(0.94);
		}
		50% {
			opacity: 0.8;
			transform: scale(1);
		}
	}

	.app-body {
		position: relative;
		display: flex;
		flex: 1;
		min-height: 0;
	}

	.main-content {
		position: relative;
		z-index: 1;
		flex: 1;
		min-width: 0;
		overflow-y: auto;
	}

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
		box-shadow: 0 20px 55px rgba(var(--accent-rgb), 0.18), var(--shadow-md);
	}

	.welcome-copy { display: grid; gap: 8px; }
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

	.welcome-actions { display: flex; gap: 10px; margin-top: 6px; }

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
		transition: border-color .16s, color .16s, transform .16s;
	}
	.secondary-cta:hover { color: var(--text-primary); border-color: color-mix(in srgb, var(--accent) 45%, var(--border)); transform: translateY(-1px); }

	.welcome-grid { display: grid; grid-template-columns: repeat(2, minmax(150px, 1fr)); gap: 10px; width: min(100%, 430px); margin-top: 10px; }
	.welcome-card { display: grid; grid-template-columns: auto 1fr; column-gap: 9px; align-items: center; padding: 13px; text-align: left; border: 1px solid var(--border); border-radius: var(--border-radius); background: color-mix(in srgb, var(--bg-card) 88%, transparent); }
	.welcome-card :global(svg) { grid-row: span 2; color: var(--accent); }
	.welcome-card span { font-size: var(--text-sm); font-weight: 750; }
	.welcome-card small { color: var(--text-muted); font-size: var(--text-xs); margin-top: 2px; }

	@media (max-width: 640px) { .welcome-actions, .welcome-grid { grid-template-columns: 1fr; width: 100%; } .welcome-actions { flex-direction: column; } .empty-cta, .secondary-cta { justify-content: center; } }
</style>
