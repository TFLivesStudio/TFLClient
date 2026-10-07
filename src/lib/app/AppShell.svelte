<script lang="ts">
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
	import { handleLogout, handleOnboardingDone } from '$lib/state/accountState.svelte';
	import {
		handleInstanceChanged,
		instanceNav,
		selectInstance
	} from '$lib/state/instanceState.svelte';
	import { flow, openInstanceContextMenu, openModal, ui } from '$lib/state/uiState.svelte';
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
				{#if appearance.ambience === 'particles' && !appearance.reduceMotion}
					<ParticlesBackground />
				{/if}
			</div>
			<Sidebar
				instances={appState.instances}
				selected={appState.selectedInstance}
				user={appState.currentUser}
				onSelect={selectInstance}
				onCreate={() => openModal('createChooser')}
				onLogout={handleLogout}
				onOpenSettings={() => openModal('settings')}
				onOpenTflSelection={() => openModal('tflSelection')}
				onJoinServer={() => openModal('joinServer')}
				onOpenSkinManager={() => openModal('skinManager')}
				onInstanceContextMenu={openInstanceContextMenu}
			/>
			<main class="main-content">
				{#if appState.selectedInstance}
					{#key appState.selectedInstance.uuid + '-' + instanceNav.navNonce}
						<InstanceDetail
							instance={appState.selectedInstance}
							onChanged={handleInstanceChanged}
							initialTab={instanceNav.pendingInitialTab}
						/>
					{/key}
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
</style>
