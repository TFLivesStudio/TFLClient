<script lang="ts">
	import { getAvailableVersions, createInstance, launchInstance } from '$lib/api/tflApi';
	import type { InstanceData, MinecraftUser, MinecraftVersion } from '$lib/types/types';
	import { t } from '$lib/i18n/index.svelte';
	import { Loader2 } from 'lucide-svelte';

	let {
		instances,
		currentUser,
		onClose,
		onLaunched
	}: {
		instances: InstanceData[];
		currentUser: MinecraftUser | null;
		onClose: () => void;
		onLaunched: () => void;
	} = $props();

	// Unirse a un servidor es multijugador — las cuentas offline/cracked no
	// pueden (ver [[project_gameplay_rules]] / account_policy.rs, misma
	// regla ya aplicada del lado del backend, esto es solo para no dejar
	// ni intentarlo desde acá).
	const blocked = $derived(currentUser?.user_type === 'Cracked');

	// Solo instancias de cliente — una instancia de servidor no tiene
	// sentido "unirla" a otro servidor.
	const joinableInstances = $derived(instances.filter((i) => !i.server_type));

	let mode = $state<'existing' | 'quick'>('quick');
	let address = $state('');
	let selectedInstance = $state('');
	let modeInitialized = $state(false);
	$effect(() => {
		if (modeInitialized) return;
		if (joinableInstances.length > 0) {
			mode = 'existing';
			selectedInstance = joinableInstances[0].name;
		}
		modeInitialized = true;
	});

	let allVersions = $state<MinecraftVersion[]>([]);
	let selectedVersion = $state('');
	let loadingVersions = $state(true);
	const releaseVersions = $derived(allVersions.filter((v) => v.type === 'release'));

	let joining = $state(false);
	let error = $state<string | null>(null);

	getAvailableVersions()
		.then((v) => {
			allVersions = v;
			selectedVersion = v.find((x) => x.type === 'release')?.id ?? v[0]?.id ?? '';
		})
		.finally(() => (loadingVersions = false));

	async function handleJoin() {
		const trimmed = address.trim();
		if (!trimmed || blocked) return;
		joining = true;
		error = null;
		try {
			if (mode === 'existing') {
				if (!selectedInstance) return;
				await launchInstance(selectedInstance, trimmed);
			} else {
				if (!selectedVersion) return;
				// Reusa una instancia Vanilla que ya tenga esta versión en vez
				// de crear una nueva cada vez que se hace quick join.
				const existing = instances.find((i) => i.loader === 'vanilla' && i.mc_version === selectedVersion && !i.server_type);
				const instanceName = existing?.name ?? (await createInstance(`Quick Join ${selectedVersion}`, selectedVersion, 'vanilla')).name;
				await launchInstance(instanceName, trimmed);
			}
			onLaunched();
		} catch (e) {
			error = String(e);
		} finally {
			joining = false;
		}
	}
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && !joining && onClose()} />

<div
	class="overlay"
	onclick={() => !joining && onClose()}
	onkeydown={(e) => e.key === 'Escape' && !joining && onClose()}
	role="button"
	tabindex="-1"
>
	<div
		class="modal"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<h2>{t('joinServer.title')}</h2>

		{#if blocked}
			<p class="error">{t('joinServer.crackedBlocked')}</p>
		{:else}
			<label for="join-address">{t('joinServer.addressLabel')}</label>
			<input
				id="join-address"
				type="text"
				bind:value={address}
				placeholder={t('joinServer.addressPlaceholder')}
				autocomplete="off"
				disabled={joining}
			/>

			<div class="mode-tabs">
				<button
					type="button"
					class="mode-tab"
					class:active={mode === 'existing'}
					disabled={joining || joinableInstances.length === 0}
					onclick={() => (mode = 'existing')}
				>
					{t('joinServer.modeExisting')}
				</button>
				<button
					type="button"
					class="mode-tab"
					class:active={mode === 'quick'}
					disabled={joining}
					onclick={() => (mode = 'quick')}
				>
					{t('joinServer.modeQuick')}
				</button>
			</div>

			{#if mode === 'existing'}
				{#if joinableInstances.length === 0}
					<p class="hint">{t('joinServer.noInstances')}</p>
				{:else}
					<select bind:value={selectedInstance} disabled={joining}>
						{#each joinableInstances as inst (inst.uuid)}
							<option value={inst.name}>{inst.name} · {inst.mc_version}</option>
						{/each}
					</select>
				{/if}
			{:else if loadingVersions}
				<div class="loading">
					<Loader2 size={16} class="spin" /> {t('createInstance.loadingVersions')}
				</div>
			{:else}
				<select bind:value={selectedVersion} disabled={joining}>
					{#each releaseVersions as v (v.id)}
						<option value={v.id}>{v.id}</option>
					{/each}
				</select>
				<p class="hint">{t('joinServer.quickHint')}</p>
			{/if}

			{#if error}
				<p class="error">{error}</p>
			{/if}
		{/if}

		<div class="actions">
			<button type="button" class="btn" onclick={onClose} disabled={joining}>{t('common.cancel')}</button>
			{#if !blocked}
				<button
					type="button"
					class="btn primary"
					disabled={joining ||
						!address.trim() ||
						(mode === 'existing' ? !selectedInstance : !selectedVersion)}
					onclick={handleJoin}
				>
					{#if joining}<Loader2 size={16} class="spin" />{/if}
					{joining ? t('joinServer.joining') : t('joinServer.join')}
				</button>
			{/if}
		</div>
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.modal {
		width: 440px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		box-shadow: var(--shadow-lg);
	}

	.modal h2 {
		font-size: var(--text-lg);
		margin-bottom: 8px;
	}

	label {
		font-size: 0.75rem;
		color: var(--text-secondary);
		margin-top: 8px;
	}

	input,
	select {
		padding: 10px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.88rem;
	}

	.mode-tabs {
		display: flex;
		gap: 6px;
		margin-top: 10px;
	}

	.mode-tab {
		flex: 1;
		padding: 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.78rem;
		font-weight: 600;
		cursor: pointer;
	}

	.mode-tab.active {
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		border-color: var(--accent);
		color: var(--text-primary);
	}

	.mode-tab:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.hint {
		font-size: 0.75rem;
		color: var(--text-muted);
		margin-top: 4px;
	}

	.error {
		font-size: 0.75rem;
		color: var(--color-error);
	}

	.loading {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.8rem;
		color: var(--text-secondary);
		margin-top: 6px;
	}

	.actions {
		display: flex;
		gap: 8px;
		margin-top: 16px;
	}

	.btn {
		flex: 1;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-weight: 600;
		font-size: 0.82rem;
		cursor: pointer;
	}

	.btn.primary {
		background: var(--accent);
		color: var(--accent-text);
		border-color: var(--accent);
	}

	.btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	:global(.spin) {
		animation: tfl-spin 0.8s linear infinite;
	}

	@keyframes tfl-spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
