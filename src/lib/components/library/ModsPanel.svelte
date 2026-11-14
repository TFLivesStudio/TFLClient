<script lang="ts">
	import { isSelectionInstance } from '$lib/selection-identity';
	import type { InstanceData } from '$lib/types/types';
	import ContentManager from './ContentManager.svelte';
	import { t } from '$lib/i18n/index.svelte';

	let { instance }: { instance: InstanceData } = $props();
</script>

<div class="mods-panel">
	{#if instance.loader === 'vanilla'}
		<p class="hint">
			{t('modsPanel.vanillaHint')}
		</p>
	{:else if isSelectionInstance(instance)}
		<p class="hint">
			{t('modsPanel.selectionHint')}
		</p>
	{:else}
		<ContentManager kind="mod" {instance} />
	{/if}
</div>

<style>
	.mods-panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.82rem;
	}
</style>
