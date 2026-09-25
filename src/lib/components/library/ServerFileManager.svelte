<script lang="ts">
	import type { InstanceData, FileEntry } from '$lib/types/types';
	import {
		listInstanceDir,
		readInstanceTextFile,
		writeInstanceTextFile,
		deleteInstancePath,
		createInstanceDir
	} from '$lib/api/tflApi';
	import { t } from '$lib/i18n/index.svelte';
	import { Folder, FileText, Trash2, ArrowLeft, FolderPlus, Loader2, Save } from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();

	let currentPath = $state('');
	let entries = $state<FileEntry[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let confirmDeleteName = $state<string | null>(null);

	let editingFile = $state<string | null>(null);
	let editingContent = $state('');
	let editingError = $state<string | null>(null);
	let savingFile = $state(false);

	function join(base: string, name: string): string {
		return base ? `${base}/${name}` : name;
	}

	async function refresh() {
		loading = true;
		error = null;
		try {
			entries = await listInstanceDir(instance.name, currentPath);
		} catch (e) {
			error = String(e);
			entries = [];
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		currentPath;
		refresh();
	});

	function openDir(name: string) {
		currentPath = join(currentPath, name);
	}

	function goUp() {
		const parts = currentPath.split('/').filter(Boolean);
		parts.pop();
		currentPath = parts.join('/');
	}

	async function openFile(name: string) {
		editingError = null;
		try {
			editingContent = await readInstanceTextFile(instance.name, join(currentPath, name));
			editingFile = name;
		} catch (e) {
			editingError = String(e);
		}
	}

	async function saveFile() {
		if (!editingFile) return;
		savingFile = true;
		try {
			await writeInstanceTextFile(instance.name, join(currentPath, editingFile), editingContent);
			editingFile = null;
		} catch (e) {
			editingError = String(e);
		} finally {
			savingFile = false;
		}
	}

	async function handleDelete(name: string) {
		confirmDeleteName = null;
		try {
			await deleteInstancePath(instance.name, join(currentPath, name));
			await refresh();
		} catch (e) {
			error = String(e);
		}
	}

	let creatingDir = $state(false);
	async function handleNewFolder() {
		const name = prompt(t('serverFileManager.newFolderPrompt'));
		if (!name?.trim()) return;
		creatingDir = true;
		try {
			await createInstanceDir(instance.name, join(currentPath, name.trim()));
			await refresh();
		} catch (e) {
			error = String(e);
		} finally {
			creatingDir = false;
		}
	}

	function fmtSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	const breadcrumbs = $derived.by(() => {
		const parts = currentPath.split('/').filter(Boolean);
		let acc = '';
		return parts.map((p) => {
			acc = acc ? `${acc}/${p}` : p;
			return { name: p, path: acc };
		});
	});
</script>

{#if editingFile}
	<div class="editor">
		<div class="editor-header">
			<span class="editor-filename">{editingFile}</span>
			<div class="editor-actions">
				<button type="button" class="icon-btn" onclick={() => (editingFile = null)} disabled={savingFile}>
					{t('common.cancel')}
				</button>
				<button type="button" class="save-btn" onclick={saveFile} disabled={savingFile}>
					{#if savingFile}<Loader2 size={13} class="spin" />{:else}<Save size={13} />{/if}
					{t('serverFileManager.save')}
				</button>
			</div>
		</div>
		{#if editingError}<p class="error">{editingError}</p>{/if}
		<textarea bind:value={editingContent} spellcheck="false"></textarea>
	</div>
{:else}
	<div class="file-manager">
		<div class="toolbar">
			<div class="breadcrumbs">
				<button type="button" class="crumb" onclick={() => (currentPath = '')}>{instance.name}</button>
				{#each breadcrumbs as crumb (crumb.path)}
					<span class="sep">/</span>
					<button type="button" class="crumb" onclick={() => (currentPath = crumb.path)}>{crumb.name}</button>
				{/each}
			</div>
			<button type="button" class="tool-btn" onclick={handleNewFolder} disabled={creatingDir}>
				{#if creatingDir}<Loader2 size={13} class="spin" />{:else}<FolderPlus size={13} />{/if}
				{t('serverFileManager.newFolder')}
			</button>
		</div>

		{#if error}<p class="error">{error}</p>{/if}

		{#if currentPath}
			<button type="button" class="up-row" onclick={goUp}>
				<ArrowLeft size={14} /> {t('serverFileManager.up')}
			</button>
		{/if}

		{#if loading}
			<p class="hint">{t('instanceDetail.loading')}</p>
		{:else if entries.length === 0}
			<p class="hint">{t('serverFileManager.empty')}</p>
		{:else}
			<div class="entries">
				{#each entries as entry (entry.name)}
					<div class="entry-row">
						<button
							type="button"
							class="entry-main"
							onclick={() => (entry.is_dir ? openDir(entry.name) : openFile(entry.name))}
						>
							{#if entry.is_dir}<Folder size={15} class="entry-icon" />{:else}<FileText size={15} class="entry-icon" />{/if}
							<span class="entry-name">{entry.name}</span>
							{#if !entry.is_dir}<span class="entry-size">{fmtSize(entry.size_bytes)}</span>{/if}
						</button>
						{#if confirmDeleteName === entry.name}
							<button type="button" class="confirm-btn" onclick={() => handleDelete(entry.name)}>
								{t('serverWorlds.confirmDelete')}
							</button>
						{:else}
							<button
								type="button"
								class="icon-btn danger"
								onclick={() => (confirmDeleteName = entry.name)}
								aria-label={t('contentManager.remove')}
							>
								<Trash2 size={13} />
							</button>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
{/if}

<style>
	.file-manager {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}

	.breadcrumbs {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 0.78rem;
		color: var(--text-secondary);
		overflow-x: auto;
	}

	.crumb {
		background: transparent;
		border: none;
		color: inherit;
		cursor: pointer;
		white-space: nowrap;
		padding: 2px 4px;
		border-radius: 4px;
	}

	.crumb:hover {
		color: var(--accent);
		background: var(--bg-input);
	}

	.sep {
		color: var(--text-muted);
	}

	.tool-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.76rem;
		font-weight: 600;
		cursor: pointer;
		white-space: nowrap;
	}

	.up-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 8px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 0.78rem;
		cursor: pointer;
		align-self: flex-start;
	}

	.up-row:hover {
		color: var(--accent);
	}

	.hint {
		font-size: 0.78rem;
		color: var(--text-muted);
	}

	.error {
		font-size: 0.78rem;
		color: var(--color-error);
	}

	.entries {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.entry-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.entry-main {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 7px 10px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: var(--bg-input);
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
		min-width: 0;
	}

	.entry-main:hover {
		background: color-mix(in srgb, var(--accent) 8%, var(--bg-input));
	}

	.entry-main :global(.entry-icon) {
		flex-shrink: 0;
		color: var(--text-muted);
	}

	.entry-name {
		flex: 1;
		font-size: 0.82rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.entry-size {
		font-size: 0.68rem;
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
	}

	.icon-btn.danger:hover {
		color: var(--color-error);
	}

	.confirm-btn {
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--color-error);
		background: var(--color-error);
		color: #fff;
		font-size: 0.72rem;
		font-weight: 600;
		cursor: pointer;
		flex-shrink: 0;
	}

	.editor {
		display: flex;
		flex-direction: column;
		gap: 8px;
		height: 460px;
	}

	.editor-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.editor-filename {
		font-size: 0.85rem;
		font-weight: 700;
		font-family: ui-monospace, 'SF Mono', 'Cascadia Code', monospace;
	}

	.editor-actions {
		display: flex;
		gap: 8px;
	}

	.editor-actions .icon-btn {
		width: auto;
		padding: 0 10px;
		font-size: 0.78rem;
	}

	.save-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: var(--accent);
		color: var(--accent-text);
		font-size: 0.78rem;
		font-weight: 600;
		cursor: pointer;
	}

	.save-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.editor textarea {
		flex: 1;
		padding: 12px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: #0a0a0a;
		color: #d4d4d8;
		font-family: ui-monospace, 'SF Mono', 'Cascadia Code', monospace;
		font-size: 0.78rem;
		line-height: 1.5;
		resize: none;
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
