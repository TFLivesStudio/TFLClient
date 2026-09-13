<script lang="ts">
	import { onMount } from 'svelte';
	import TitleBar from '$lib/components/layout/TitleBar/TitleBar.svelte';
	import Sidebar from '$lib/components/layout/Sidebar/Sidebar.svelte';
	import Onboarding from '$lib/components/onboarding/Onboarding.svelte';
	import CreateInstanceModal from '$lib/components/library/CreateInstanceModal.svelte';
	import InstanceDetail from '$lib/components/library/InstanceDetail.svelte';
	import DownloadProgressBar from '$lib/components/library/DownloadProgressBar.svelte';
	import SettingsPanel from '$lib/components/settings/SettingsPanel.svelte';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { appState } from '$lib/state/state.svelte';
	import { initDownloadListener } from '$lib/state/downloadState.svelte';
	import { getCurrentUser, getInstances, getSettings, logout as apiLogout } from '$lib/api/tflApi';
	import type { InstanceData, MinecraftUser } from '$lib/types/types';
	import { Plus } from 'lucide-svelte';

	let loading = $state(true);
	let showCreateModal = $state(false);
	let showSettings = $state(false);

	async function refreshInstances() {
		appState.instances = await getInstances();
	}

	onMount(async () => {
		initDownloadListener();
		try {
			const [user, settings] = await Promise.all([getCurrentUser(), getSettings()]);
			appState.currentUser = user;
			appState.settings = settings;
			if (settings.theme === 'light' || settings.theme === 'dark') {
				document.documentElement.setAttribute('data-theme', settings.theme);
			}
			const accent = localStorage.getItem('tfl-accent');
			if (accent && accent !== 'orange') {
				document.documentElement.setAttribute('data-accent', accent);
			}
			await refreshInstances();
		} finally {
			loading = false;
		}
	});

	async function handleOnboardingDone(user: MinecraftUser) {
		appState.currentUser = user;
		appState.settings = await getSettings();
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
			<div class="ambient-bg"></div>
			<Sidebar
				instances={appState.instances}
				selected={appState.selectedInstance}
				user={appState.currentUser}
				onSelect={handleSelect}
				onCreate={() => (showCreateModal = true)}
				onLogout={handleLogout}
				onOpenSettings={() => (showSettings = true)}
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
						<div class="empty-mark"><Tfl width="46" height="46" /></div>
						<h2>Bienvenido a TFL Client</h2>
						<p>Elegí una instancia de la barra lateral o creá una nueva</p>
						<button type="button" class="empty-cta" onclick={() => (showCreateModal = true)}>
							<Plus size={16} strokeWidth={2.5} />
							Crear instancia
						</button>
					</div>
				{/if}
			</main>
		</div>
	{/if}
</div>

{#if showCreateModal}
	<CreateInstanceModal onClose={() => (showCreateModal = false)} onCreated={handleCreated} />
{/if}

{#if showSettings}
	<SettingsPanel onClose={() => (showSettings = false)} />
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
		gap: 6px;
		height: 100%;
		color: var(--text-secondary);
		text-align: center;
	}

	.empty-mark {
		color: var(--text-muted);
		opacity: 0.5;
		margin-bottom: 10px;
	}

	.empty-state h2 {
		color: var(--text-primary);
		font-size: 1.15rem;
	}

	.empty-state p {
		font-size: 0.85rem;
		margin-bottom: 18px;
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
	}
</style>
