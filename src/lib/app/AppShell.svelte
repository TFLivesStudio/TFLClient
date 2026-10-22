<script lang="ts">
	import { contentExplorer } from '$lib/state/uiState.svelte';
	import LivingScene from '$lib/scenes/LivingScene.svelte';
	import { reveal } from '$lib/visual/motion';
	import { startNotifications } from '$lib/visual/notifications.svelte';
	import LibraryView from '$lib/components/refresh/LibraryView.svelte';
	import DiscoverView from '$lib/components/refresh/DiscoverView.svelte';
	import { navigation } from '$lib/visual/navigation.svelte';
	import { restoreVisualPreferences } from '$lib/visual/preferences.svelte';
	import { startVisualRuntime, effects, runtime } from '$lib/visual/runtime.svelte';

	import { onMount } from 'svelte';
	import TitleBar from '$lib/components/layout/TitleBar/TitleBar.svelte';
	import Sidebar from '$lib/components/layout/Sidebar/Sidebar.svelte';
	import Onboarding from '$lib/components/onboarding/Onboarding.svelte';
	import NativeDialogModePrompt from '$lib/components/onboarding/NativeDialogModePrompt.svelte';
	import InstanceDetail from '$lib/components/library/InstanceDetail.svelte';
	import ParticlesBackground from '$lib/components/layout/ParticlesBackground.svelte';
	import WelcomeState from '$lib/app/WelcomeState.svelte';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { appState } from '$lib/state/state.svelte';
	import { appearance } from '$lib/state/appearanceState.svelte';
	import { initViewportListener } from '$lib/state/viewport.svelte';
	import { handleLogout, handleOnboardingDone } from '$lib/state/accountState.svelte';
	import {
		handleInstanceChanged,
		instanceNav,
		selectInstance
	} from '$lib/state/instanceState.svelte';
	import { flow, openInstanceContextMenu, openModal, ui, modals } from '$lib/state/uiState.svelte';

	$effect(() => {
		runtime.overlay = Object.values(modals).some(Boolean) || contentExplorer.open;
	});
	// Responsive del shell (Entrega B, sección 4) — ver viewport.svelte.ts.
	// Se inicializa una sola vez acá, al mismo nivel que TitleBar/Sidebar (los
	// dos lo consumen), en vez de en bootstrap.ts (fuera del alcance de esta
	// entrega) — es idempotente, no pasa nada si se llama de más.
	onMount(() => {
		const stopViewport = initViewportListener();
		restoreVisualPreferences();
		const stopVisual = startVisualRuntime();
		const stopNotifications = startNotifications();
		return () => {
			stopViewport?.();
			stopVisual();
			stopNotifications();
		};
	});
</script>

<div class="app-shell">
	<TitleBar />

	{#if ui.loading}
		<div class="loading-screen">
			<div class="loading-mark"><Tfl width="28" height="28" /></div>
		</div>
	{:else if flow.needsOnboarding}
		<Onboarding onDone={handleOnboardingDone} />
	{:else if ui.justOnboarded && flow.needsDialogModePrompt}
		<NativeDialogModePrompt variant="modal" onDone={() => (ui.justOnboarded = false)} />
	{:else}
		<div class="app-body">
			<div class="ambient-bg">
				{#if appearance.ambience === 'particles' && effects.current.ambient}
					<ParticlesBackground />
				{/if}
			</div>
			<LivingScene />
			<Sidebar
				instances={appState.instances}
				selected={appState.selectedInstance}
				user={appState.currentUser}
				onSelect={selectInstance}
				onCreate={() => openModal('createChooser')}
				onLogout={handleLogout}
				onOpenSettings={() => openModal('settings')}
				onJoinServer={() => openModal('joinServer')}
				onOpenSkinManager={() => openModal('skinManager')}
				onInstanceContextMenu={openInstanceContextMenu}
			/>
			<main
				class="main-content"
				use:reveal={(appState.selectedInstance?.uuid ?? navigation.section) +
					effects.current.motion}
			>
				{#if appState.selectedInstance}
					{#key appState.selectedInstance.uuid + '-' + instanceNav.navNonce}
						<InstanceDetail
							instance={appState.selectedInstance}
							onChanged={handleInstanceChanged}
							initialTab={instanceNav.pendingInitialTab}
						/>
					{/key}
				{:else if navigation.section === 'library'}
					<LibraryView />
				{:else if navigation.section === 'discover'}
					<DiscoverView />
				{:else}
					<WelcomeState />
				{/if}
			</main>
		</div>
	{/if}
</div>

<style>
	.app-shell {
		display: flex;
		flex-direction: column;
		height: 100vh;
		width: 100vw;
		/* Fondo general del shell — --surface-canvas (Entrega A) en vez de
		   --bg-main suelto, para que TitleBar/Sidebar (--surface-workspace) y
		   el contenido (--surface-canvas) vengan todos de la misma capa
		   semántica de superficies en vez de cada pieza eligiendo su propio
		   --bg-* por separado (sección 1 del encargo). Son el mismo valor en
		   global.css hoy — esto no cambia nada visualmente, solo la fuente. */
		background: var(--surface-canvas);
		overflow: hidden;
	}

	.loading-screen {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-canvas);
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
		min-width: 0;
	}

	.main-content {
		position: relative;
		z-index: var(--z-content);
		flex: 1;
		min-width: 0;
		overflow-y: auto;
		/* Nunca scroll horizontal, en ninguna de las dos vistas que vive este
		   nodo (WelcomeState/InstanceDetail, sección 16 del encargo) ni en la
		   ventana mínima soportada (800x600). */
		overflow-x: hidden;
		/* Evita el salto de layout al navegar Home↔InstanceDetail (sección
		   1 del encargo: "la navegación no debe provocar saltos innecesarios
		   del layout") cuando una vista tiene scrollbar vertical y la otra
		   no — sin esto, el ancho de .main-content cambia en ~15px apenas
		   aparece/desaparece la scrollbar nativa al cambiar de vista. */
		scrollbar-gutter: stable;
	}
</style>
