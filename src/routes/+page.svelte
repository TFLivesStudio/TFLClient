<script lang="ts">
	import { onMount } from 'svelte';
	import AppShell from '$lib/app/AppShell.svelte';
	import ModalHost from '$lib/app/ModalHost.svelte';
	import { isLogWindow, startApp } from '$lib/app/bootstrap';
	import InstanceLogWindow from '$lib/components/library/InstanceLogWindow.svelte';
	import CommandPalette from '$lib/components/layout/CommandPalette.svelte';
	import WhatsNewTips from '$lib/components/onboarding/WhatsNewTips.svelte';
	import NativeDialogModePrompt from '$lib/components/onboarding/NativeDialogModePrompt.svelte';
	import DownloadProgressBar from '$lib/components/library/DownloadProgressBar.svelte';
	import UpdateBadge from '$lib/components/library/UpdateBadge.svelte';
	import { appState } from '$lib/state/state.svelte';
	import { selectInstance } from '$lib/state/instanceState.svelte';
	import { flow, isCommandPaletteBlocked, openModal, ui } from '$lib/state/uiState.svelte';

	// Composición: la secuencia de arranque vive en $lib/app/bootstrap, el
	// layout en AppShell y los modales en ModalHost (abiertos vía uiState).
	onMount(() => {
		void startApp();
	});
</script>

{#if isLogWindow}
	<InstanceLogWindow />
{:else}
	<AppShell />
{/if}

<ModalHost />

{#if !isLogWindow}
	<CommandPalette
		instances={appState.instances}
		blocked={isCommandPaletteBlocked()}
		onSelectInstance={selectInstance}
		onCreate={() => openModal('createChooser')}
		onOpenSettings={() => openModal('settings')}
		onOpenTflSelection={() => openModal('tflSelection')}
		onJoinServer={() => openModal('joinServer')}
		onOpenSkinManager={() => openModal('skinManager')}
	/>
{/if}

{#if !isLogWindow && !flow.needsOnboarding}
	<WhatsNewTips />
{/if}

{#if !isLogWindow && !flow.needsOnboarding && !ui.justOnboarded && flow.needsDialogModePrompt}
	<NativeDialogModePrompt variant="banner" onDismiss={() => (ui.dialogPromptDismissed = true)} />
{/if}

<DownloadProgressBar />
<UpdateBadge />
