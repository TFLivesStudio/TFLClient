<script lang="ts">
	import type { InstanceData } from '$lib/types/types';
	import ContentManager from './ContentManager.svelte';
	import { t } from '$lib/i18n/index.svelte';

	let { instance }: { instance: InstanceData } = $props();
</script>

<div class="plugins-panel">
	{#if instance.server_type === 'vanilla'}
		<p class="hint">{t('pluginsPanel.vanillaHint')}</p>
	{:else}
		<ContentManager kind="plugin" {instance} />
	{/if}
</div>

<style>
	.plugins-panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.82rem;
	}
</style>
