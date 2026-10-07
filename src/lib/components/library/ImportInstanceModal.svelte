<script lang="ts">
	import { onMount } from 'svelte';
	import type { ImportCandidate, InstanceData } from '$lib/types/types';
	import { detectImportableInstances, importExternalInstance } from '$lib/api/tflApi';
	import { t } from '$lib/i18n/index.svelte';
	import { Loader2, Download } from 'lucide-svelte';

	let {
		onClose,
		onImported
	}: { onClose: () => void; onImported: (instance: InstanceData) => void } = $props();

	let candidates = $state<ImportCandidate[]>([]);
	let loading = $state(true);
	let selectedPath = $state<string | null>(null);
	let nameInput = $state('');
	let importing = $state(false);
	let error = $state<string | null>(null);

	const SOURCE_LABEL: Record<ImportCandidate['source'], string> = {
		prism: 'Prism Launcher',
		multimc: 'MultiMC',
		polymc: 'PolyMC',
		curseforge: 'CurseForge'
	};

	onMount(async () => {
		try {
			candidates = await detectImportableInstances();
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	});

	function select(c: ImportCandidate) {
		if (!c.mc_version || importing) return;
		selectedPath = c.path;
		nameInput = c.name;
		error = null;
	}

	async function handleImport() {
		const name = nameInput.trim();
		if (!selectedPath || !name) return;
		importing = true;
		error = null;
		try {
			const instance = await importExternalInstance(selectedPath, name);
			onImported(instance);
		} catch (e) {
			error = String(e);
			importing = false;
		}
	}
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && !importing && onClose()} />

<div
	class="overlay"
	onclick={() => !importing && onClose()}
	onkeydown={(e) => e.key === 'Escape' && !importing && onClose()}
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
		<h2>{t('importInstance.title')}</h2>
		<p class="subtitle">{t('importInstance.subtitle')}</p>

		{#if loading}
			<div class="state"><Loader2 size={16} class="spin" /> {t('importInstance.searching')}</div>
		{:else if candidates.length === 0}
			<p class="empty">{t('importInstance.none')}</p>
		{:else}
			<div class="list">
				{#each candidates as c (c.path)}
					<button
						type="button"
						class="candidate"
						class:selected={selectedPath === c.path}
						disabled={!c.mc_version}
						onclick={() => select(c)}
					>
						<span class="c-main">
							<span class="c-name">{c.name}</span>
							<span class="c-meta">
								{c.mc_version ?? t('importInstance.unknownVersion')} · {c.loader} · {t(
									'importInstance.modCount',
									{
										count: c.mod_count
									}
								)}
							</span>
						</span>
						<span class="c-source">{SOURCE_LABEL[c.source]}</span>
					</button>
				{/each}
			</div>
		{/if}

		{#if selectedPath}
			<label class="name-field">
				{t('importInstance.nameLabel')}
				<input type="text" bind:value={nameInput} disabled={importing} maxlength="60" />
			</label>
			<p class="hint">{t('importInstance.hint')}</p>
		{/if}

		{#if error}<p class="error">{error}</p>{/if}

		<div class="actions">
			<button type="button" class="secondary" disabled={importing} onclick={onClose}>
				{t('common.cancel')}
			</button>
			<button
				type="button"
				class="primary"
				disabled={!selectedPath || !nameInput.trim() || importing}
				onclick={handleImport}
			>
				{#if importing}<Loader2 size={14} class="spin" />{t(
						'importInstance.importing'
					)}{:else}<Download size={14} />{t('importInstance.import')}{/if}
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
		width: 480px;
		max-height: 80vh;
		overflow-y: auto;
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
	}

	.subtitle {
		font-size: 0.8rem;
		color: var(--text-secondary);
		margin-bottom: 8px;
	}

	.state,
	.empty {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.8rem;
		color: var(--text-muted);
		margin: 8px 0;
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-height: 260px;
		overflow-y: auto;
	}

	.candidate {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 12px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
	}

	.candidate:hover:not(:disabled) {
		border-color: var(--accent);
	}

	.candidate.selected {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 10%, var(--bg-input));
	}

	.candidate:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.c-main {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.c-name {
		font-size: 0.86rem;
		font-weight: 700;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.c-meta {
		font-size: 0.72rem;
		color: var(--text-secondary);
	}

	.c-source {
		flex-shrink: 0;
		font-size: 0.66rem;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--text-muted);
	}

	.name-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-top: 8px;
		font-size: 0.72rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--text-muted);
	}

	.name-field input {
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.86rem;
		text-transform: none;
		letter-spacing: 0;
		font-weight: 500;
	}

	.hint {
		font-size: 0.72rem;
		color: var(--text-muted);
		margin: 0;
	}

	.error {
		font-size: 0.76rem;
		color: var(--color-error);
		margin: 0;
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 12px;
	}

	.actions button {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 9px 16px;
		border-radius: var(--border-radius-sm);
		font-weight: 600;
		font-size: 0.82rem;
		cursor: pointer;
	}

	.secondary {
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
	}

	.primary {
		border: none;
		background: var(--accent);
		color: var(--accent-contrast, #fff);
	}

	.actions button:disabled {
		opacity: 0.55;
		cursor: default;
	}
</style>
