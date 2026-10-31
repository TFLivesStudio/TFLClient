<script lang="ts">
	import { dragDrop } from '$lib/state/dragDrop.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { FolderInput } from 'lucide-svelte';
</script>

{#if dragDrop.over}
	<div class="drag-overlay">
		<div class="drag-card">
			<FolderInput size={36} strokeWidth={1.5} />
			<p>{t('importInstance.dropHint')}</p>
		</div>
	</div>
{/if}

<style>
	.drag-overlay {
		position: fixed;
		inset: 0;
		z-index: var(--z-toast);
		display: flex;
		align-items: center;
		justify-content: center;
		background: color-mix(in srgb, black 55%, transparent);
		backdrop-filter: blur(2px);
		pointer-events: none;
	}

	.drag-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 32px 48px;
		border: 2px dashed var(--accent);
		border-radius: 16px;
		background: var(--surface-elevated);
		color: var(--text-primary);
	}

	.drag-card p {
		margin: 0;
		font-size: 15px;
		font-weight: 600;
	}
</style>
