<script lang="ts">
	import type { Snippet } from 'svelte';
	import { setting, type SettingId } from '$lib/settings/catalog';
	import { i18nState } from '$lib/i18n/index.svelte';
	let { id, hint = '', children }: { id: SettingId; hint?: string; children: Snippet } = $props();
	const item = $derived(setting(id));
</script>

<section class="setting-group" data-setting={id} tabindex="-1" aria-labelledby={`setting-${id}`}>
	<header>
		<h4 id={`setting-${id}`}>{item.label[i18nState.locale === 'en' ? 1 : 0]}</h4>
		{#if hint}<p>{hint}</p>{/if}
	</header>
	<div class="setting-content">{@render children()}</div>
</section>

<style>
	.setting-group {
		min-width: 0;
		scroll-margin-top: 20px;
		padding: 0 0 26px;
		border-bottom: 1px solid var(--border);
	}
	header {
		margin-bottom: 14px;
	}
	h4 {
		margin: 0;
		font-size: 15px;
		font-weight: 650;
		color: var(--text-primary);
	}
	p {
		font-size: 12px;
		line-height: 1.65;
		color: var(--text-secondary);
		margin: 6px 0 0;
		max-width: 72ch;
	}
	.setting-content {
		min-width: 0;
	}
	.setting-group:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 8px;
	}
	.setting-group:global([data-highlighted]) {
		outline: 2px solid var(--accent);
		outline-offset: 8px;
		background: color-mix(in srgb, var(--accent) 5%, transparent);
	}
	.setting-content :global(> section > .section-label) {
		display: none;
	}
</style>
