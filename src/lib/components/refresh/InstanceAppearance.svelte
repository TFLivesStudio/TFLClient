<script lang="ts">
	import { Image, Star } from 'lucide-svelte';
	import {
		instanceVisual,
		setInstanceVisual,
		visual,
		preferenceStatus
	} from '$lib/visual/preferences.svelte';
	import { ARTWORKS, artworkUrl } from '$lib/visual/policy';
	import { vr } from '$lib/i18n/refresh';
	let { id }: { id: string } = $props();
	const meta = $derived(instanceVisual(id));
</script>

<section class="instance-appearance">
	<div class="appearance-heading">
		<h3><Image size={17} />{vr('artwork')}</h3>
		<button
			class="vr-control"
			aria-pressed={meta.favorite}
			onclick={() => setInstanceVisual(id, { favorite: !meta.favorite })}
			><Star size={15} />{vr('favorite')}</button
		>
	</div>
	<p class="vr-muted">{vr('artworkHint')}</p>
	<div class="artwork-options">
		{#each ARTWORKS as artwork}<button
				aria-pressed={meta.artwork === artwork}
				aria-label={artwork === 'auto' || artwork === 'none'
					? vr(artwork)
					: `${vr('artwork')} ${artwork.slice(3)}`}
				onclick={() => setInstanceVisual(id, { artwork })}
				>{#if artworkUrl(artwork)}<img
						src={artworkUrl(artwork) ?? ''}
						alt=""
						loading="lazy"
					/>{:else}<span>{vr(artwork as 'auto' | 'none')}</span>{/if}</button
			>{/each}
	</div>
	<label class="group-label"
		>{vr('group')}<select
			value={meta.group}
			onchange={(e) => setInstanceVisual(id, { group: e.currentTarget.value })}
			><option value="">{vr('ungrouped')}</option>{#each visual.groups as group}<option
					value={group}>{group}</option
				>{/each}</select
		></label
	>{#if preferenceStatus.error}<p role="alert">{vr('saveError')}</p>{/if}
</section>

<style>
	.instance-appearance {
		padding: 22px;
		margin-top: 20px;
		border: 1px solid var(--border);
		background: var(--surface-card);
		border-radius: var(--border-radius);
	}
	.appearance-heading {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 14px;
	}
	.appearance-heading h3 {
		display: flex;
		gap: 10px;
		align-items: center;
		font-size: 1.1rem;
	}
	.artwork-options {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(70px, 1fr));
		gap: 8px;
		margin: 16px 0;
	}
	.artwork-options button {
		height: 58px;
		background: var(--surface-interactive);
		border: 2px solid transparent;
		border-radius: var(--border-radius-sm);
		overflow: hidden;
		color: var(--text-secondary);
		padding: 0;
		font-size: 12px;
	}
	.artwork-options button[aria-pressed='true'] {
		border-color: var(--accent);
	}
	.artwork-options img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.group-label {
		display: flex;
		gap: 16px;
		align-items: center;
		font-size: var(--text-small);
	}
	select {
		padding: 10px;
		background: var(--surface-interactive);
		color: var(--text-primary);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
	}
</style>
