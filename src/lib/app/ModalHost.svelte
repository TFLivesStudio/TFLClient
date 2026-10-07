<script lang="ts">
	import CreateInstanceModal from '$lib/components/library/CreateInstanceModal.svelte';
	import CreateInstanceChooser from '$lib/components/library/CreateInstanceChooser.svelte';
	import CreateServerModal from '$lib/components/library/CreateServerModal.svelte';
	import ImportInstanceModal from '$lib/components/library/ImportInstanceModal.svelte';
	import JoinServerModal from '$lib/components/library/JoinServerModal.svelte';
	import SkinManagerModal from '$lib/components/library/SkinManagerModal.svelte';
	import ServerConnectionInfoModal from '$lib/components/library/ServerConnectionInfoModal.svelte';
	import SettingsPanel from '$lib/components/settings/SettingsPanel.svelte';
	import TflSelection from '$lib/components/library/TflSelection.svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';
	import { buildInstanceContextMenuItems } from '$lib/app/instanceContextMenu';
	import { appState } from '$lib/state/state.svelte';
	import {
		handleCreated,
		handleServerCreated,
		refreshInstances
	} from '$lib/state/instanceState.svelte';
	import {
		closeInstanceContextMenu,
		closeModal,
		modals,
		openModal,
		ui
	} from '$lib/state/uiState.svelte';
</script>

<!-- El orden del DOM es el de apilado entre modales del mismo z-index: no reordenar. -->

{#if modals.createChooser}
	<CreateInstanceChooser
		onClose={() => closeModal('createChooser')}
		onChooseCustom={() => {
			closeModal('createChooser');
			openModal('create');
		}}
		onChooseModpack={() => {
			closeModal('createChooser');
			openModal('tflSelection');
		}}
		onChooseServer={() => {
			closeModal('createChooser');
			openModal('createServer');
		}}
		onChooseImport={() => {
			closeModal('createChooser');
			openModal('importInstance');
		}}
	/>
{/if}

{#if modals.importInstance}
	<ImportInstanceModal
		onClose={() => closeModal('importInstance')}
		onImported={async (instance) => {
			closeModal('importInstance');
			await handleCreated(instance);
		}}
	/>
{/if}

{#if modals.create}
	<CreateInstanceModal onClose={() => closeModal('create')} onCreated={handleCreated} />
{/if}

{#if modals.createServer}
	<CreateServerModal onClose={() => closeModal('createServer')} onCreated={handleServerCreated} />
{/if}

{#if modals.serverConnectionInfo}
	<ServerConnectionInfoModal onDone={() => closeModal('serverConnectionInfo')} />
{/if}

{#if modals.skinManager}
	<SkinManagerModal onClose={() => closeModal('skinManager')} />
{/if}

{#if ui.contextMenu}
	<ContextMenu
		x={ui.contextMenu.x}
		y={ui.contextMenu.y}
		items={buildInstanceContextMenuItems(ui.contextMenu.instance)}
		onClose={closeInstanceContextMenu}
	/>
{/if}

{#if modals.joinServer}
	<JoinServerModal
		instances={appState.instances}
		currentUser={appState.currentUser}
		onClose={() => closeModal('joinServer')}
		onLaunched={async () => {
			closeModal('joinServer');
			await refreshInstances();
		}}
	/>
{/if}

{#if modals.settings}
	<SettingsPanel onClose={() => closeModal('settings')} />
{/if}

{#if modals.tflSelection}
	<TflSelection
		instances={appState.instances}
		onClose={() => closeModal('tflSelection')}
		onInstanceCreated={() => refreshInstances()}
	/>
{/if}
