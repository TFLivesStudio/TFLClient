<script lang="ts">
	import { getServerVersions, createServerInstance } from '$lib/api/tflApi';
	import type { InstanceData, ServerType } from '$lib/types/types';
	import { t } from '$lib/i18n/index.svelte';
	import { Loader2 } from 'lucide-svelte';

	let {
		onClose,
		onCreated
	}: {
		onClose: () => void;
		onCreated: (instance: InstanceData) => void;
	} = $props();

	const SERVER_TYPES: { id: ServerType; label: string; descKey: 'vanillaDesc' | 'paperDesc' | 'purpurDesc' }[] = [
		{ id: 'vanilla', label: 'Vanilla', descKey: 'vanillaDesc' },
		{ id: 'paper', label: 'Paper', descKey: 'paperDesc' },
		{ id: 'purpur', label: 'Purpur', descKey: 'purpurDesc' }
	];

	let name = $state('');
	let selectedType = $state<ServerType>('paper');
	let versions = $state<string[]>([]);
	let selectedVersion = $state('');
	let loadingVersions = $state(true);
	let creating = $state(false);
	let error = $state<string | null>(null);

	async function loadVersions() {
		loadingVersions = true;
		try {
			versions = await getServerVersions(selectedType);
			selectedVersion = versions[0] ?? '';
		} catch (e) {
			error = String(e);
			versions = [];
		} finally {
			loadingVersions = false;
		}
	}

	$effect(() => {
		selectedType;
		loadVersions();
	});

	async function handleCreate() {
		if (!name.trim() || !selectedVersion) return;
		creating = true;
		error = null;
		try {
			const instance = await createServerInstance(name.trim(), selectedVersion, selectedType);
			onCreated(instance);
		} catch (e) {
			error = String(e);
		} finally {
			creating = false;
		}
	}
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && !creating && onClose()} />

<div
	class="overlay"
	onclick={() => !creating && onClose()}
	onkeydown={(e) => e.key === 'Escape' && !creating && onClose()}
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
		<h2>{t('createServer.title')}</h2>

		<label for="server-name">{t('createInstance.nameLabel')}</label>
		<input
			id="server-name"
			type="text"
			bind:value={name}
			placeholder={t('createInstance.namePlaceholder')}
			autocomplete="off"
			disabled={creating}
		/>

		<label for="server-type">{t('createServer.serverTypeLabel')}</label>
		<div class="type-grid" id="server-type">
			{#each SERVER_TYPES as st (st.id)}
				<button
					type="button"
					class="type-btn"
					class:active={selectedType === st.id}
					disabled={creating}
					onclick={() => (selectedType = st.id)}
				>
					<span class="type-name">{st.label}</span>
					<span class="type-desc">{t(`createServer.${st.descKey}`)}</span>
				</button>
			{/each}
		</div>

		<label for="server-version">{t('createServer.versionLabel')}</label>
		{#if loadingVersions}
			<div class="loading">
				<Loader2 size={16} class="spin" /> {t('createServer.loadingVersions')}
			</div>
		{:else if versions.length === 0}
			<p class="hint">{t('createServer.noVersionsForType')}</p>
		{:else}
			<select id="server-version" bind:value={selectedVersion} disabled={creating}>
				{#each versions as v (v)}
					<option value={v}>{v}</option>
				{/each}
			</select>
		{/if}

		{#if error}
			<p class="error">{error}</p>
		{/if}

		<div class="actions">
			<button type="button" class="btn" onclick={onClose} disabled={creating}>{t('common.cancel')}</button>
			<button
				type="button"
				class="btn primary"
				disabled={creating || !name.trim() || !selectedVersion}
				onclick={handleCreate}
			>
				{#if creating}<Loader2 size={16} class="spin" />{/if}
				{creating ? t('createServer.creating') : t('createServer.create')}
			</button>
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

	.type-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 6px;
	}

	.type-btn {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 3px;
		padding: 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		text-align: left;
		cursor: pointer;
		transition:
			background 0.15s,
			border-color 0.15s,
			color 0.15s;
	}

	.type-btn:hover:not(:disabled) {
		border-color: var(--accent);
	}

	.type-btn.active {
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		border-color: var(--accent);
		color: var(--text-primary);
	}

	.type-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.type-name {
		font-size: 0.78rem;
		font-weight: 700;
	}

	.type-desc {
		font-size: 0.64rem;
		color: var(--text-muted);
		line-height: 1.3;
	}

	.hint {
		font-size: 0.75rem;
		color: var(--text-muted);
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
