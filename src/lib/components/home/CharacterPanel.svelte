<script lang="ts">
	import { downloadTasks } from '$lib/state/downloadState.svelte';
	import { SkinViewer3D } from '$lib/components/ui';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { tr } from '$lib/i18n/experience';
	import { Shirt } from 'lucide-svelte';
	import type { MojangProfile } from '$lib/types/types';

	let {
		isPremium,
		accountUuid,
		profile,
		onOpenSkinManager,
		launching = false
	}: {
		isPremium: boolean;
		accountUuid: string | null;
		profile: MojangProfile | null;
		onOpenSkinManager: () => void;
		launching?: boolean;
	} = $props();

	const downloading = $derived(Object.values(downloadTasks).some((task) => !task.failed));
	const activeSkin = $derived(profile?.skins.find((skin) => skin.state === 'ACTIVE') ?? null);
	const activeCape = $derived(profile?.capes.find((cape) => cape.state === 'ACTIVE') ?? null);
	let skinError = $state(false);
	$effect(() => {
		void activeSkin?.url;
		void accountUuid;
		skinError = false;
	});
</script>

<div class="character-figure" class:account-offline={!!accountUuid && !isPremium}>
	{#if accountUuid}
		{#key accountUuid}
			<SkinViewer3D
				priority="hero"
				{launching}
				working={downloading}
				width={220}
				height={320}
				zoom={0.9}
				skinUrl={skinError ? '/skins/steve.png' : (activeSkin?.url ?? '/skins/steve.png')}
				model={!skinError && activeSkin?.variant === 'SLIM' ? 'slim' : 'default'}
				capeUrl={activeCape?.url ?? null}
				onSkinError={() => (skinError = true)}
			/>
		{/key}
	{:else}
		<div class="no-account-orb" aria-hidden="true"><Tfl width="36" height="36" /></div>
	{/if}
</div>

{#if accountUuid && !isPremium}
	<p class="account-caption">{tr('Cuenta offline', 'Offline account')}</p>
{:else if !accountUuid}
	<p class="account-caption">{t('home.noAccountCaption')}</p>
{/if}
{#if skinError}
	<p class="skin-error" role="status">
		{tr(
			'No se pudo cargar la skin. Se muestra Steve.',
			'The skin could not be loaded. Showing Steve.'
		)}
	</p>
{/if}

{#if isPremium}
	<div class="skins-row">
		{#each (profile?.skins ?? []).slice(0, 3) as skin (skin.id)}
			<button
				type="button"
				class="skin-slot"
				title={t('home.openSkinManager')}
				aria-label={t('home.openSkinManager')}
				onclick={onOpenSkinManager}
			>
				<span class="skin-layer base" style:background-image={`url("${skin.url}")`}></span>
				<span class="skin-layer overlay" style:background-image={`url("${skin.url}")`}></span>
			</button>
		{/each}
		<button
			type="button"
			class="skin-slot skin-slot-add"
			title={t('home.openSkinManager')}
			aria-label={t('home.openSkinManager')}
			onclick={onOpenSkinManager}><Shirt size={14} /></button
		>
	</div>
{/if}

<style>
	.character-figure {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 150px;
	}
	.character-figure :global(canvas),
	.character-figure :global(.pixel-person) {
		filter: drop-shadow(0 14px 28px rgba(0, 0, 0, 0.45));
	}
	.account-offline :global(canvas),
	.account-offline :global(.pixel-person) {
		filter: grayscale(1) drop-shadow(0 14px 28px rgba(0, 0, 0, 0.45));
	}
	.no-account-orb {
		display: grid;
		place-items: center;
		width: 160px;
		height: 160px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		color: var(--accent);
		background: var(--surface-card);
	}
	.account-caption,
	.skin-error {
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		color: #d8e1eb;
		text-align: center;
	}
	.skin-error {
		color: #ffb4b4;
	}
	.skins-row {
		display: flex;
		gap: 6px;
	}
	.skin-slot {
		position: relative;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--surface-interactive);
		overflow: hidden;
		cursor: pointer;
		padding: 0;
	}
	.skin-slot:hover {
		border-color: var(--accent);
	}
	.skin-slot:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 3px;
	}
	.skin-layer {
		position: absolute;
		inset: 0;
		background-repeat: no-repeat;
		background-size: 256px 256px;
		image-rendering: pixelated;
	}
	.skin-layer.base {
		background-position: -32px -32px;
	}
	.skin-layer.overlay {
		background-position: -160px -32px;
	}
	.skin-slot-add {
		display: grid;
		place-items: center;
		color: var(--text-muted);
	}
</style>
