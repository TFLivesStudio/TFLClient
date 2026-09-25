<script lang="ts">
	import { onMount } from 'svelte';
	import type { InstanceData, WorldInfo } from '$lib/types/types';
	import { listServerWorlds, deleteServerWorld } from '$lib/api/tflApi';
	import { t } from '$lib/i18n/index.svelte';
	import { Globe, Trash2, Loader2 } from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();

	let worlds = $state<WorldInfo[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let deletingName = $state<string | null>(null);
	let confirmDelete = $state<string | null>(null);

	async function refresh() {
		loading = true;
		error = null;
		try {
			worlds = await listServerWorlds(instance.name);
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	onMount(refresh);

	async function handleDelete(name: string) {
		confirmDelete = null;
		deletingName = name;
		try {
			await deleteServerWorld(instance.name, name);
			await refresh();
		} catch (e) {
			error = String(e);
		} finally {
			deletingName = null;
		}
	}

	function fmtSize(bytes: number): string {
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}
</script>

<div class="worlds-panel">
	<p class="hint">{t('serverWorlds.hint')}</p>

	{#if error}<p class="error">{error}</p>{/if}

	{#if loading}
		<p class="hint">{t('instanceDetail.loading')}</p>
	{:else if worlds.length === 0}
		<div class="empty">
			<Globe size={22} />
			<p>{t('serverWorlds.empty')}</p>
		</div>
	{:else}
		<div class="worlds-list">
			{#each worlds as world (world.name)}
				<div class="world-row">
					<Globe size={16} class="world-icon" />
					<div class="world-text">
						<span class="world-name">{world.name}</span>
						<span class="world-size">{fmtSize(world.size_bytes)}</span>
					</div>
					{#if confirmDelete === world.name}
						<button type="button" class="confirm-btn" onclick={() => handleDelete(world.name)}>
							{t('serverWorlds.confirmDelete')}
						</button>
						<button type="button" class="icon-btn" onclick={() => (confirmDelete = null)}>
							{t('common.cancel')}
						</button>
					{:else}
						<button
							type="button"
							class="icon-btn danger"
							disabled={deletingName === world.name}
							onclick={() => (confirmDelete = world.name)}
							aria-label={t('serverWorlds.delete')}
						>
							{#if deletingName === world.name}<Loader2 size={13} class="spin" />{:else}<Trash2 size={13} />{/if}
						</button>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.worlds-panel {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
	}

	.error {
		color: var(--color-error);
		font-size: 0.78rem;
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 32px 0;
		color: var(--text-muted);
		text-align: center;
	}

	.empty p {
		font-size: 0.78rem;
	}

	.worlds-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.world-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
	}

	.world-row :global(.world-icon) {
		color: var(--accent);
		flex-shrink: 0;
	}

	.world-text {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}

	.world-name {
		font-size: 0.83rem;
		font-weight: 600;
	}

	.world-size {
		font-size: 0.68rem;
		color: var(--text-muted);
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
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
		padding: 5px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--color-error);
		background: var(--color-error);
		color: #fff;
		font-size: 0.72rem;
		font-weight: 600;
		cursor: pointer;
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
