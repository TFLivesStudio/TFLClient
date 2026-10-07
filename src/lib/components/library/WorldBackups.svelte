<script lang="ts">
	import type { InstanceData, WorldBackup } from '$lib/types/types';
	import {
		getWorldBackups,
		createWorldBackup,
		restoreWorldBackup,
		deleteWorldBackup
	} from '$lib/api/tflApi';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { t, i18nState } from '$lib/i18n/index.svelte';
	import { Archive, Loader2, RotateCcw, Trash2 } from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();

	let backups = $state<WorldBackup[]>([]);
	let loaded = $state(false);
	let creating = $state(false);
	let busyId = $state<string | null>(null);
	let message = $state<string | null>(null);
	let error = $state<string | null>(null);
	let confirmRestore = $state<WorldBackup | null>(null);

	async function load() {
		try {
			backups = await getWorldBackups(instance.name);
		} catch (e) {
			error = String(e);
		} finally {
			loaded = true;
		}
	}

	$effect(() => {
		void instance.name;
		load();
	});

	async function handleCreate() {
		creating = true;
		error = null;
		message = null;
		try {
			const created = await createWorldBackup(instance.name);
			message = created ? t('worldBackups.created') : t('worldBackups.noWorlds');
			await load();
		} catch (e) {
			error = String(e);
		} finally {
			creating = false;
		}
	}

	async function doRestore(b: WorldBackup) {
		confirmRestore = null;
		busyId = b.id;
		error = null;
		message = null;
		try {
			await restoreWorldBackup(instance.name, b.id);
			message = t('worldBackups.restored');
			await load();
		} catch (e) {
			error = String(e);
		} finally {
			busyId = null;
		}
	}

	async function handleDelete(b: WorldBackup) {
		busyId = b.id;
		error = null;
		message = null;
		try {
			await deleteWorldBackup(instance.name, b.id);
			await load();
		} catch (e) {
			error = String(e);
		} finally {
			busyId = null;
		}
	}

	function formatDate(secs: number): string {
		return new Date(secs * 1000).toLocaleString(i18nState.locale, {
			dateStyle: 'medium',
			timeStyle: 'short'
		});
	}

	function formatSize(bytes: number): string {
		if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}
</script>

<section class="backups">
	<div class="head">
		<span class="section-label">{t('worldBackups.title')}</span>
		<button type="button" class="create-btn" disabled={creating} onclick={handleCreate}>
			{#if creating}<Loader2 size={13} class="spin" />{:else}<Archive size={13} />{/if}
			{t('worldBackups.create')}
		</button>
	</div>
	<p class="hint">{t('worldBackups.hint')}</p>

	{#if message}<p class="ok">{message}</p>{/if}
	{#if error}<p class="error">{error}</p>{/if}

	{#if loaded && backups.length === 0}
		<p class="empty">{t('worldBackups.empty')}</p>
	{:else}
		<div class="list">
			{#each backups as b (b.id)}
				<div class="row">
					<div class="info">
						<span class="date">{formatDate(b.created_at)}</span>
						<span class="size">{formatSize(b.size_bytes)}</span>
					</div>
					<button
						type="button"
						class="row-btn"
						disabled={busyId === b.id}
						onclick={() => (confirmRestore = b)}
					>
						{#if busyId === b.id}<Loader2 size={12} class="spin" />{:else}<RotateCcw
								size={12}
							/>{/if}
						{t('worldBackups.restore')}
					</button>
					<button
						type="button"
						class="icon-btn"
						disabled={busyId === b.id}
						onclick={() => handleDelete(b)}
						aria-label={t('worldBackups.delete')}
						title={t('worldBackups.delete')}
					>
						<Trash2 size={13} />
					</button>
				</div>
			{/each}
		</div>
	{/if}
</section>

{#if confirmRestore}
	<ConfirmDialog
		title={t('worldBackups.restoreTitle')}
		message={t('worldBackups.restoreMessage', { date: formatDate(confirmRestore.created_at) })}
		confirmLabel={t('worldBackups.restore')}
		danger
		onConfirm={() => confirmRestore && doRestore(confirmRestore)}
		onCancel={() => (confirmRestore = null)}
	/>
{/if}

<style>
	.backups {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		padding: 16px;
		margin-top: 12px;
	}

	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
	}

	.section-label {
		display: block;
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.hint {
		font-size: 0.72rem;
		color: var(--text-muted);
		margin: 4px 0 12px;
	}

	.create-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 11px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.74rem;
		cursor: pointer;
	}

	.create-btn:hover:not(:disabled) {
		border-color: var(--accent);
	}

	.create-btn:disabled {
		opacity: 0.6;
		cursor: default;
	}

	.ok {
		font-size: 0.74rem;
		color: var(--color-success, #4ade80);
		margin: 0 0 8px;
	}

	.error {
		font-size: 0.74rem;
		color: var(--color-error);
		margin: 0 0 8px;
	}

	.empty {
		font-size: 0.74rem;
		color: var(--text-muted);
		margin: 0;
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
	}

	.info {
		flex: 1;
		display: flex;
		align-items: baseline;
		gap: 10px;
		min-width: 0;
	}

	.date {
		font-size: 0.78rem;
		color: var(--text-primary);
	}

	.size {
		font-size: 0.68rem;
		color: var(--text-muted);
	}

	.row-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 4px 9px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: transparent;
		color: var(--text-primary);
		font-size: 0.72rem;
		cursor: pointer;
	}

	.row-btn:hover:not(:disabled) {
		border-color: var(--accent);
	}

	.row-btn:disabled,
	.icon-btn:disabled {
		opacity: 0.55;
		cursor: default;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: var(--border-radius-sm);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.icon-btn:hover:not(:disabled) {
		color: var(--color-error);
	}
</style>
