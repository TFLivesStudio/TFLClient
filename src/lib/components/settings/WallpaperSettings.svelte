<script lang="ts">
	import { t, type TranslationKey } from '$lib/i18n/index.svelte';
	import { WandSparkles, Check, Upload } from 'lucide-svelte';
	let {
		ambience,
		wallpaper,
		customWallpaperUrl,
		customWallpaperBusy,
		customWallpaperError,
		dialogsBlocked,
		applyAmbience,
		applyWallpaper,
		selectCustomWallpaper,
		pickCustomWallpaper
	}: {
		ambience: string;
		wallpaper: string;
		customWallpaperUrl: string | null;
		customWallpaperBusy: boolean;
		customWallpaperError: string | null;
		dialogsBlocked: boolean;
		applyAmbience: (id: string) => void;
		applyWallpaper: (id: string) => void;
		selectCustomWallpaper: () => void;
		pickCustomWallpaper: () => Promise<void>;
	} = $props();
	const WALLPAPERS: { id: string; key: TranslationKey; preview: string }[] = [
		{ id: 'none', key: 'settings.wallpaper.items.none', preview: 'transparent' },
		{
			id: 'void-night',
			key: 'settings.wallpaper.items.voidNight',
			preview: 'linear-gradient(135deg, #02040a, #050d1a, #030810)'
		},
		{
			id: 'nether',
			key: 'settings.wallpaper.items.nether',
			preview: 'linear-gradient(135deg, #140202, #250808, #0e0101)'
		},
		{
			id: 'end',
			key: 'settings.wallpaper.items.end',
			preview: 'linear-gradient(135deg, #070212, #0e0520, #040110)'
		},
		{
			id: 'deep-ocean',
			key: 'settings.wallpaper.items.deepOcean',
			preview: 'linear-gradient(135deg, #010f1a, #021a2e, #010c14)'
		},
		{
			id: 'aurora-gradient',
			key: 'settings.wallpaper.items.auroraGradient',
			preview: 'linear-gradient(135deg, #040d10, #041410 40%, #0a0420 80%, #04100d)'
		},
		{
			id: 'animated-aurora',
			key: 'settings.wallpaper.items.animatedAurora',
			preview: 'linear-gradient(135deg, #040d10, #041410 40%, #0a0420 80%, #04100d)'
		},
		{ id: 'obsidian-solid', key: 'settings.wallpaper.items.obsidianSolid', preview: '#08090c' },
		{ id: 'charcoal', key: 'settings.wallpaper.items.charcoal', preview: '#0f1115' },
		{
			id: 'savanna',
			key: 'settings.wallpaper.items.savanna',
			preview: 'url(/wallpapers/mc-wallpaper-1.jpg)'
		},
		{
			id: 'golden-sunset',
			key: 'settings.wallpaper.items.goldenSunset',
			preview: 'url(/wallpapers/mc-wallpaper-2.jpg)'
		},
		{
			id: 'lake-night',
			key: 'settings.wallpaper.items.lakeNight',
			preview: 'url(/wallpapers/mc-wallpaper-3.jpg)'
		},
		{
			id: 'neon-arcade',
			key: 'settings.wallpaper.items.neonArcade',
			preview: 'url(/wallpapers/mc-wallpaper-4.jpg)'
		},
		{
			id: 'red-canyon',
			key: 'settings.wallpaper.items.redCanyon',
			preview: 'url(/wallpapers/mc-wallpaper-5.jpg)'
		},
		{
			id: 'enchanted-valley',
			key: 'settings.wallpaper.items.enchantedValley',
			preview: 'url(/wallpapers/mc-wallpaper-6.jpg)'
		},
		{
			id: 'snowy-peak',
			key: 'settings.wallpaper.items.snowyPeak',
			preview: 'url(/wallpapers/mc-wallpaper-7.jpg)'
		},
		{
			id: 'stone-bridge',
			key: 'settings.wallpaper.items.stoneBridge',
			preview: 'url(/wallpapers/mc-wallpaper-8.jpg)'
		},
		{
			id: 'misty-fortress',
			key: 'settings.wallpaper.items.mistyFortress',
			preview: 'url(/wallpapers/mc-wallpaper-9.jpg)'
		},
		{
			id: 'village-tower',
			key: 'settings.wallpaper.items.villageTower',
			preview: 'url(/wallpapers/mc-wallpaper-10.jpg)'
		},
		{
			id: 'deep-cave',
			key: 'settings.wallpaper.items.deepCave',
			preview: 'url(/wallpapers/mc-wallpaper-11.jpg)'
		},
		{
			id: 'sunset-coast',
			key: 'settings.wallpaper.items.sunsetCoast',
			preview: 'url(/wallpapers/mc-wallpaper-12.jpg)'
		},
		{
			id: 'abstract-blocks',
			key: 'settings.wallpaper.items.abstractBlocks',
			preview: 'url(/wallpapers/mc-wallpaper-13.jpg)'
		}
	];

	const AMBIENCES: { id: string; key: TranslationKey }[] = [
		{ id: 'aurora', key: 'settings.ambience.aurora' },
		{ id: 'cosmic', key: 'settings.ambience.cosmic' },
		{ id: 'minimal', key: 'settings.ambience.minimal' },
		{ id: 'particles', key: 'settings.ambience.particles' }
	];
</script>

<div class="wallpaper-settings">
	<section>
		<span class="section-label"><WandSparkles size={13} /> {t('settings.ambience.label')}</span>
		<div class="row">
			{#each AMBIENCES as item (item.id)}<button
					type="button"
					class="choice"
					class:active={ambience === item.id}
					onclick={() => applyAmbience(item.id)}>{t(item.key)}</button
				>{/each}
		</div>
	</section>
	<section>
		<span class="section-label"><WandSparkles size={13} /> {t('settings.wallpaper.label')}</span>
		<div class="wallpaper-grid">
			{#each WALLPAPERS as item (item.id)}
				<button
					type="button"
					class="wallpaper-swatch"
					class:active={wallpaper === item.id}
					style="background: {item.preview}; background-size: cover; background-position: center;"
					onclick={() => applyWallpaper(item.id)}
					aria-label={t(item.key)}
					title={t(item.key)}
				>
					{#if wallpaper === item.id}<Check size={12} />{/if}
				</button>
			{/each}
			{#if customWallpaperUrl}
				<button
					type="button"
					class="wallpaper-swatch"
					class:active={wallpaper === 'custom'}
					style="background-image: url('{customWallpaperUrl}'); background-size: cover; background-position: center;"
					onclick={selectCustomWallpaper}
					aria-label={t('settings.wallpaper.yourImage')}
					title={t('settings.wallpaper.yourImage')}
				>
					{#if wallpaper === 'custom'}<Check size={12} />{/if}
				</button>
			{/if}
			<button
				type="button"
				class="wallpaper-swatch wallpaper-upload"
				onclick={pickCustomWallpaper}
				disabled={customWallpaperBusy || dialogsBlocked}
				aria-label={customWallpaperUrl
					? t('settings.wallpaper.change')
					: t('settings.wallpaper.upload')}
				title={dialogsBlocked
					? t('settings.wallpaper.uploadDisabledTitle')
					: customWallpaperUrl
						? t('settings.wallpaper.change')
						: t('settings.wallpaper.upload')}
			>
				<Upload size={13} />
			</button>
		</div>
		{#if dialogsBlocked}<p class="mac-notice">
				{t('settings.wallpaper.autoModeDisabled')}
			</p>{/if}
		{#if customWallpaperError}<p class="error-text">{customWallpaperError}</p>{/if}
	</section>
</div>

<style>
	.wallpaper-settings {
		display: grid;
		gap: 20px;
	}
	.section-label {
		display: block;
		font-size: 12px;
		color: var(--text-secondary);
		margin-bottom: 12px;
	}
	.row {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
	.choice {
		padding: 8px;
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		border-radius: var(--border-radius-sm);
		cursor: pointer;
	}
	.choice.active {
		border-color: var(--accent);
		color: var(--accent);
	}
	.wallpaper-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
		gap: 10px;
	}
	.wallpaper-swatch {
		aspect-ratio: 16/10;
		border: 2px solid transparent;
		border-radius: var(--border-radius-sm);
		padding: 0;
		cursor: pointer;
		color: white;
		display: grid;
		place-items: center;
	}
	.wallpaper-swatch.active {
		border-color: var(--accent);
	}
	.wallpaper-upload {
		background: var(--surface-card);
		border: 1px dashed var(--border);
		color: var(--text-secondary);
	}
	.wallpaper-upload:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.error-text {
		color: var(--color-error);
		font-size: 12px;
	}
	.mac-notice {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.6;
	}
</style>
