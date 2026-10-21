<script lang="ts">
	import { Check, Sparkles, Grid2X2, Layers, Monitor } from 'lucide-svelte';
	import { visual, setVisual, preferenceStatus } from '$lib/visual/preferences.svelte';
	import { vr } from '$lib/i18n/refresh';
	import type { QualityProfile } from '$lib/types/types';
	let { onQuality }: { onQuality: (profile: QualityProfile) => Promise<void> } = $props();
	async function preset(name: 'calm' | 'cinematic' | 'arcade') {
		setVisual('style', name === 'arcade' ? 'pixel' : 'modern');
		setVisual('material', name === 'calm' ? 'matte' : 'soft-glass');
		await onQuality(name === 'calm' ? 'Lite' : 'Experience');
	}
</script>

<section class="visual-settings">
	<div class="setting-title">
		<span class="vr-eyebrow"><Sparkles size={16} />{vr('visual')}</span><span
			class="save-state"
			role="status"
			>{#if !preferenceStatus.error}<Check size={13} />{/if}{vr(
				preferenceStatus.error ? 'saveError' : 'saved'
			)}</span
		>
	</div>
	<div class="style-options">
		{#each ['modern', 'pixel'] as style (style)}<button
				class="style-option"
				class:pixel-demo={style === 'pixel'}
				aria-pressed={visual.style === style}
				onclick={() => setVisual('style', style as 'modern' | 'pixel')}
				><span class="style-art"><span></span><i></i><i></i><b></b></span><strong
					>{vr(style as 'modern' | 'pixel')}</strong
				><small>{vr(style === 'modern' ? 'modernHint' : 'pixelHint')}</small
				>{#if visual.style === style}<Check class="chosen" size={16} />{/if}</button
			>{/each}
	</div>
	{#if visual.style === 'pixel'}<div class="setting-row">
			<span>{vr('pixelTypography')}</span>
			<div class="segmented">
				{#each ['readable', 'classic'] as type (type)}<button
						class="vr-control"
						aria-pressed={visual.pixelType === type}
						onclick={() => setVisual('pixelType', type as 'readable' | 'classic')}
						>{vr(type as 'readable' | 'classic')}</button
					>{/each}
			</div>
		</div>{/if}
	<div class="setting-row">
		<span><Layers size={16} />{vr('material')}</span>
		<div class="segmented">
			{#each ['matte', 'soft-glass', 'glass'] as material (material)}<button
					class="vr-control"
					aria-pressed={visual.material === material}
					onclick={() => setVisual('material', material as 'matte' | 'soft-glass' | 'glass')}
					>{vr(material === 'soft-glass' ? 'soft' : (material as 'matte' | 'glass'))}</button
				>{/each}
		</div>
	</div>
	<label class="setting-row"
		><span>{vr('adaptive')}</span><input
			type="checkbox"
			checked={visual.adaptive}
			onchange={(e) => setVisual('adaptive', e.currentTarget.checked)}
		/></label
	>
	<div class="setting-row">
		<label for="custom-accent">{vr('customAccent')}</label><input
			id="custom-accent"
			type="color"
			value={visual.customAccent || '#ff7a2e'}
			oninput={(e) => setVisual('customAccent', e.currentTarget.value)}
		/><button class="vr-control" onclick={() => setVisual('customAccent', '')}
			>{vr('resetAccent')}</button
		>
	</div>
	<div class="presets">
		<span class="vr-eyebrow"><Monitor size={15} />{vr('preset')}</span>
		<div class="segmented">
			{#each ['calm', 'cinematic', 'arcade'] as scene (scene)}<button
					class="vr-control"
					onclick={() => preset(scene as 'calm' | 'cinematic' | 'arcade')}
					>{vr(scene as 'calm' | 'cinematic' | 'arcade')}</button
				>{/each}
		</div>
		<p class="vr-muted">{vr('profileHint')}</p>
	</div>
	<div class="vr-preview">
		<span class="vr-eyebrow">{vr('preview')}</span>
		<h3>{vr('yourWorlds')}</h3>
		<div>
			<span class="preview-block"><Grid2X2 size={20} /></span><span
				><strong>Minecraft</strong><small>1.21.1 · Fabric</small></span
			><span class="preview-cta" aria-hidden="true">▶ {vr('play')}</span>
		</div>
	</div>
	<div class="home-options">
		<span class="vr-eyebrow">{vr('homePanels')}</span
		>{#each ['recent', 'servers', 'activity', 'system'] as panel (panel)}<label class="setting-row"
				><span
					>{vr(
						`${panel}Panel` as 'recentPanel' | 'serversPanel' | 'activityPanel' | 'systemPanel'
					)}</span
				><input
					type="checkbox"
					checked={visual.home[panel as keyof typeof visual.home]}
					onchange={(e) => setVisual('home', { ...visual.home, [panel]: e.currentTarget.checked })}
				/></label
			>{/each}
	</div>
</section>

<style>
	.visual-settings {
		display: flex;
		flex-direction: column;
		gap: 22px;
		padding-bottom: 26px;
		border-bottom: 1px solid var(--border);
	}
	.setting-title {
		display: flex;
		gap: 12px;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: center;
	}
	.save-state {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--text-caption);
		color: var(--text-muted);
	}
	.style-options {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 14px;
	}
	.style-option {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 9px;
		padding: 16px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		background: var(--surface-card);
		text-align: left;
		color: var(--text-primary);
	}
	.style-option[aria-pressed='true'] {
		border-color: var(--accent);
		box-shadow: 0 0 0 1px var(--accent);
	}
	.style-option small {
		color: var(--text-secondary);
		line-height: 1.5;
	}
	.style-option strong {
		font-size: 1.15rem;
	}
	.style-art {
		position: relative;
		display: block;
		background: var(--surface-canvas);
		height: 80px;
		border: 1px solid var(--border);
		border-radius: 9px;
		width: 100%;
		overflow: hidden;
	}
	.style-art span {
		position: absolute;
		inset: 10px auto 10px 10px;
		width: 20px;
		border-radius: 4px;
		background: var(--surface-interactive);
	}
	.style-art i {
		position: absolute;
		left: 40px;
		top: 12px;
		width: 40%;
		height: 10px;
		background: var(--text-muted);
		opacity: 0.3;
		border-radius: 4px;
	}
	.style-art i + i {
		top: 30px;
		width: 60%;
		height: 22px;
		background: linear-gradient(
			100deg,
			color-mix(in srgb, var(--accent) 24%, transparent),
			transparent
		);
	}
	.style-art b {
		position: absolute;
		bottom: 10px;
		left: 40px;
		width: 45px;
		height: 12px;
		border-radius: 4px;
		background: var(--accent);
	}
	.pixel-demo .style-art,
	.pixel-demo .style-art * {
		border-radius: 0 !important;
	}
	.pixel-demo .style-art {
		box-shadow: 3px 3px 0 #0005;
		background-image: linear-gradient(var(--border) 1px, transparent 1px);
		background-size: 8px 8px;
	}
	:global(.chosen) {
		position: absolute;
		right: 12px;
		bottom: 40px;
		color: var(--accent);
	}
	.setting-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 20px;
		font-size: var(--text-small);
		padding: 4px 0;
	}
	.setting-row > span {
		display: flex;
		align-items: center;
		gap: 9px;
	}
	.setting-row input {
		width: 18px;
		height: 18px;
		accent-color: var(--accent);
	}
	.segmented {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}
	.presets {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.vr-preview {
		padding: 22px;
		border: 1px solid var(--border);
		background: var(--surface-canvas);
		border-radius: var(--border-radius);
	}
	.vr-preview h3 {
		font-size: 1.4rem;
		margin: 16px 0;
	}
	.vr-preview > div {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.preview-block {
		padding: 12px;
		background: var(--surface-interactive);
		border-radius: var(--border-radius-sm);
		color: var(--accent);
	}
	.vr-preview strong,
	.vr-preview small {
		display: block;
	}
	.vr-preview small {
		color: var(--text-muted);
		margin-top: 6px;
	}
	.preview-cta {
		margin-left: auto;
		padding: 12px 20px;
		color: var(--accent-text);
		background: var(--accent);
		border-radius: var(--border-radius-sm);
		font-weight: 700;
	}
	.home-options {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	@media (max-width: 850px) {
		.setting-row {
			align-items: flex-start;
			flex-wrap: wrap;
		}
		.style-option {
			padding: 12px;
		}
		.style-option small {
			font-size: 12px;
		}
	}
</style>
