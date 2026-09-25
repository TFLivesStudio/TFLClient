<script lang="ts">
	import type { InstanceData } from '$lib/types/types';
	import ContentManager from './ContentManager.svelte';
	import { t } from '$lib/i18n/index.svelte';

	let { instance }: { instance: InstanceData } = $props();
</script>

<div class="resourcepacks-panel">
	<p class="hint">
		{t('resourcePacksPanel.hint')}
	</p>
	<ContentManager kind="resourcepack" {instance} />
</div>

<style>
	.resourcepacks-panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
	}
</style>
