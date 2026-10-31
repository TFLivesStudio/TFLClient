<script lang="ts">
	import { modalFocus } from '$lib/visual/modalFocus';
	import { onMount } from 'svelte';
	import type { ImportCandidate, InstanceData, MrpackPreview } from '$lib/types/types';
	import {
		detectImportableInstances,
		importExternalInstance,
		inspectMrpack,
		importMrpackFile,
		pickContentFiles
	} from '$lib/api';
	import { appState } from '$lib/state/state.svelte';
	import { mrpackDropIntent } from '$lib/state/uiState.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { Loader2, Download, FolderInput } from 'lucide-svelte';

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
		curseforge: 'CurseForge',
		modrinth_app: 'Modrinth App'
	};

	// Importar un .mrpack suelto (exportado de Prism, de la app de Modrinth,
	// de CurseForge, o bajado de cualquier lado) — vista aparte de la lista
	// de arriba, que solo detecta instalaciones de OTROS launchers en el
	// equipo. Mismo modal porque las dos terminan en lo mismo: una
	// instancia nueva vía `onImported`.
	let view = $state<'list' | 'mrpack'>('list');
	let mrpackPath = $state<string | null>(null);
	let mrpackPreview = $state<MrpackPreview | null>(null);
	let mrpackName = $state('');
	let mrpackBusy = $state<'pick' | 'inspect' | 'import' | null>(null);
	let mrpackError = $state<string | null>(null);

	// El diálogo nativo de archivos crashea el launcher en macOS con la
	// firma ad-hoc del build (ver CHANGELOG_macos-dialog-crash-java26.txt) —
	// mismo gate que `pick_content_files` en ContentManager/InstanceIconPicker.
	let dialogsBlocked = $derived(appState.settings?.native_dialog_mode !== 'manual');

	const LOADER_LABEL: Record<MrpackPreview['loader'], string> = {
		vanilla: 'Vanilla',
		fabric: 'Fabric',
		quilt: 'Quilt',
		forge: 'Forge',
		neoforge: 'NeoForge'
	};

	// Compartido entre elegir el archivo a mano (pickMrpack) y soltarlo
	// arrastrado sobre la ventana (ver mrpackDropIntent más abajo) — las dos
	// vías terminan en la misma vista de confirmación.
	async function inspectAndShow(path: string) {
		mrpackBusy = 'inspect';
		mrpackError = null;
		try {
			const preview = await inspectMrpack(path);
			mrpackPath = path;
			mrpackPreview = preview;
			mrpackName = preview.suggested_name;
			view = 'mrpack';
		} catch (e) {
			mrpackError = String(e);
		} finally {
			mrpackBusy = null;
		}
	}

	async function pickMrpack() {
		if (dialogsBlocked || mrpackBusy) return;
		mrpackError = null;
		mrpackBusy = 'pick';
		const paths = await pickContentFiles('mrpack', 'Modpack (.mrpack)');
		if (paths.length === 0) {
			mrpackBusy = null;
			return;
		}
		await inspectAndShow(paths[0]);
	}

	async function confirmMrpackImport() {
		const name = mrpackName.trim();
		if (!mrpackPath || !name || mrpackBusy) return;
		mrpackBusy = 'import';
		mrpackError = null;
		try {
			const instance = await importMrpackFile(mrpackPath, name);
			onImported(instance);
		} catch (e) {
			mrpackError = String(e);
			mrpackBusy = null;
		}
	}

	function backToList() {
		if (mrpackBusy) return;
		view = 'list';
		mrpackPath = null;
		mrpackPreview = null;
		mrpackName = '';
		mrpackError = null;
	}

	// Si se soltó un .mrpack sobre la ventana (ver initDragDropListener) —
	// al abrir el modal o con el modal ya abierto en la lista — salta directo
	// a la vista de confirmación con ese archivo.
	$effect(() => {
		const path = mrpackDropIntent.path;
		if (!path) return;
		mrpackDropIntent.path = null;
		void inspectAndShow(path);
	});

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

<svelte:window
	onkeydown={(e) =>
		e.key === 'Escape' &&
		!importing &&
		!mrpackBusy &&
		(view === 'mrpack' ? backToList() : onClose())}
/>

<div
	class="overlay"
	onclick={() => !importing && !mrpackBusy && onClose()}
	onkeydown={(e) => e.key === 'Escape' && !importing && !mrpackBusy && onClose()}
	role="button"
	tabindex="-1"
>
	<div
		class="modal"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		use:modalFocus={onClose}
		aria-modal="true"
		tabindex="-1"
	>
		{#if view === 'list'}
			<h2>{t('importInstance.title')}</h2>
			<p class="subtitle">{t('importInstance.subtitle')}</p>

			{#if loading}
				<div class="state">
					<Loader2 size={16} class="spin" />
					{t('importInstance.searching')}
				</div>
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

			<div class="divider"><span>{t('importInstance.orDivider')}</span></div>

			<button
				type="button"
				class="mrpack-btn"
				disabled={dialogsBlocked || !!mrpackBusy}
				title={dialogsBlocked ? t('contentManager.manualModeRequired') : undefined}
				onclick={pickMrpack}
			>
				{#if mrpackBusy === 'pick' || mrpackBusy === 'inspect'}
					<Loader2 size={14} class="spin" />
				{:else}
					<FolderInput size={14} />
				{/if}
				{t('importInstance.mrpackButton')}
			</button>
			{#if dialogsBlocked}
				<p class="hint">{t('importInstance.mrpackDialogsBlocked')}</p>
			{/if}
			{#if mrpackError}<p class="error">{mrpackError}</p>{/if}

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
		{:else if mrpackPreview}
			<h2>{t('importInstance.mrpackPreviewTitle')}</h2>
			<p class="subtitle">
				{mrpackPreview.mc_version} · {LOADER_LABEL[mrpackPreview.loader]}
				{#if mrpackPreview.loader_version}
					{mrpackPreview.loader_version}
				{/if}
				· {t('importInstance.mrpackFileCount', { count: mrpackPreview.file_count })}
			</p>
			{#if mrpackPreview.summary}<p class="hint">{mrpackPreview.summary}</p>{/if}

			<label class="name-field">
				{t('importInstance.nameLabel')}
				<input
					type="text"
					bind:value={mrpackName}
					disabled={mrpackBusy === 'import'}
					maxlength="60"
				/>
			</label>
			<p class="hint">{t('importInstance.hint')}</p>

			{#if mrpackError}<p class="error">{mrpackError}</p>{/if}

			<div class="actions">
				<button
					type="button"
					class="secondary"
					disabled={mrpackBusy === 'import'}
					onclick={backToList}
				>
					{t('common.back')}
				</button>
				<button
					type="button"
					class="primary"
					disabled={!mrpackName.trim() || mrpackBusy === 'import'}
					onclick={confirmMrpackImport}
				>
					{#if mrpackBusy === 'import'}<Loader2 size={14} class="spin" />{t(
							'importInstance.mrpackImporting'
						)}{:else}<Download size={14} />{t('importInstance.mrpackConfirm')}{/if}
				</button>
			</div>
		{/if}
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

	.divider {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 6px 0;
		font-size: 0.68rem;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--text-muted);
	}

	.divider::before,
	.divider::after {
		content: '';
		flex: 1;
		height: 1px;
		background: var(--border);
	}

	.mrpack-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 10px 12px;
		border-radius: var(--border-radius);
		border: 1px dashed var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.82rem;
		font-weight: 600;
		cursor: pointer;
	}

	.mrpack-btn:hover:not(:disabled) {
		border-color: var(--accent);
		color: var(--accent);
	}

	.mrpack-btn:disabled {
		opacity: 0.5;
		cursor: default;
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
