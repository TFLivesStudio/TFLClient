<script lang="ts">
	import { Monitor, Moon, Sun, Sparkles, Check } from 'lucide-svelte';
	import { appState } from '$lib/state/state.svelte';
	import { updateSettings } from '$lib/api';
	import { applyQualityVisuals } from '$lib/state/appearanceState.svelte';
	import { visual, setVisual, preferenceStatus } from '$lib/visual/preferences.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { tr } from '$lib/i18n/experience';
	import type { QualityProfile } from '$lib/types/types';
	import SettingGroup from './SettingGroup.svelte';
	import AccentPalette from './AccentPalette.svelte';
	let {
		onQuality,
		onTheme,
		onSurface,
		onAccent,
		onFont,
		onDensity,
		onCards,
		font,
		surface,
		density,
		cards
	}: {
		onQuality: (q: QualityProfile) => Promise<void>;
		onTheme: (t: 'dark' | 'light') => Promise<void>;
		onSurface: (id: string) => void;
		onAccent: (id: string) => void;
		onFont: (id: string) => void;
		onDensity: (id: string) => void;
		onCards: (id: string) => void;
		font: string;
		surface: string;
		density: string;
		cards: string;
	} = $props();
	let error = $state('');
	async function effects(
		key: 'reduce_animations' | 'disable_blur_effects' | 'disable_infinite_animations',
		value: boolean
	) {
		if (!appState.settings) return;
		try {
			const next = { ...appState.settings, [key]: value };
			await updateSettings(next);
			appState.settings = next;
			applyQualityVisuals(next);
			error = '';
		} catch (e) {
			error = String(e);
		}
	}
	async function theme(value: 'dark' | 'light' | 'oled') {
		if (value === 'oled') {
			await onTheme('dark');
			onSurface('oled');
		} else {
			if (surface === 'oled') onSurface('obsidian');
			await onTheme(value);
		}
	}
</script>

<div class="appearance-settings">
	<SettingGroup
		id="style"
		hint={tr(
			'Dos identidades completas. Tus mundos y ajustes se conservan.',
			'Two complete identities. Your worlds and settings stay with you.'
		)}
		><div class="style-options">
			{#each ['modern', 'pixel'] as style (style)}<button
					type="button"
					class="style-option"
					class:pixel-demo={style === 'pixel'}
					aria-pressed={visual.style === style}
					onclick={() => setVisual('style', style as 'modern' | 'pixel')}
					><span class="style-art"><span></span><i></i><b></b></span><strong
						>{vr(style as 'modern' | 'pixel')}</strong
					><small>{vr(style === 'modern' ? 'modernHint' : 'pixelHint')}</small
					>{#if visual.style === style}<Check size={16} class="chosen" />{/if}</button
				>{/each}
		</div></SettingGroup
	>
	<SettingGroup id="theme"
		><div class="theme-options">
			{#each ['dark', 'light', 'oled'] as value (value)}{@const active =
					value === 'oled'
						? surface === 'oled'
						: surface !== 'oled' && appState.settings?.theme === value}<button
					type="button"
					class="theme-card"
					aria-pressed={active}
					onclick={() => theme(value as 'dark' | 'light' | 'oled')}
					><span class={`theme-art ${value}`}><i></i><b></b></span><span
						>{#if value === 'light'}<Sun size={15} />{tr(
								'Claro',
								'Light'
							)}{:else if value === 'oled'}<Monitor size={15} />OLED{:else}<Moon size={15} />{tr(
								'Oscuro',
								'Dark'
							)}{/if}</span
					></button
				>{/each}
		</div>
		<div class="choice-row surface-row">
			{#each ['obsidian', 'midnight', 'slate'] as item (item)}<button
					type="button"
					class="vr-control"
					aria-pressed={surface === item}
					onclick={() => onSurface(item)}>{item[0].toUpperCase() + item.slice(1)}</button
				>{/each}
		</div></SettingGroup
	>
	<SettingGroup id="accent"><AccentPalette onReset={() => onAccent('orange')} /></SettingGroup>
	<SettingGroup
		id="material"
		hint={tr(
			'Lite y «Desactivar desenfoque» mantienen superficies sólidas para ahorrar recursos.',
			'Lite and “Disable blur” keep solid surfaces to save resources.'
		)}
		><div class="choice-row">
			{#each ['matte', 'soft-glass', 'glass'] as material (material)}<button
					type="button"
					class="vr-control"
					aria-pressed={visual.material === material}
					onclick={() => setVisual('material', material as 'matte' | 'soft-glass' | 'glass')}
					>{vr(material === 'soft-glass' ? 'soft' : (material as 'matte' | 'glass'))}</button
				>{/each}
		</div></SettingGroup
	>
	<SettingGroup id="font"
		><label class="field"
			>{tr('Fuente de la interfaz', 'Interface font')}<select
				aria-label={tr('Fuente de la interfaz', 'Interface font')}
				value={font}
				onchange={(e) => onFont(e.currentTarget.value)}
				>{#each [['system', tr('Sistema', 'System')], ['nunito', 'Nunito'], ['inter', 'Inter'], ['poppins', 'Poppins'], ['jetbrains-mono', 'JetBrains Mono']] as [id, label] (id)}<option
						value={id}>{label}</option
					>{/each}</select
			></label
		>{#if visual.style === 'pixel'}<div class="choice-row" style="margin-top:12px">
				{#each ['readable', 'classic'] as type (type)}<button
						type="button"
						class="vr-control"
						aria-pressed={visual.pixelType === type}
						onclick={() => setVisual('pixelType', type as 'readable' | 'classic')}
						>{vr(type as 'readable' | 'classic')}</button
					>{/each}
			</div>{/if}</SettingGroup
	>
	<SettingGroup id="density"
		><div class="choice-row">
			{#each ['comfortable', 'compact'] as d (d)}<button
					type="button"
					class="vr-control"
					aria-pressed={density === d}
					onclick={() => onDensity(d)}
					>{tr(
						d === 'compact' ? 'Compacta' : 'Cómoda',
						d === 'compact' ? 'Compact' : 'Comfortable'
					)}</button
				>{/each}
		</div>
		<div class="choice-row" style="margin-top:12px">
			{#each ['rich', 'minimal'] as c (c)}<button
					type="button"
					class="vr-control"
					aria-pressed={cards === c}
					onclick={() => onCards(c)}
					>{tr(
						c === 'rich' ? 'Tarjetas con información' : 'Tarjetas mínimas',
						c === 'rich' ? 'Detailed cards' : 'Minimal cards'
					)}</button
				>{/each}
		</div></SettingGroup
	>
	<SettingGroup
		id="quality"
		hint={tr(
			'Afecta a los efectos del launcher, no a los FPS de Minecraft. Elegir un perfil restablece sus valores de movimiento y transparencia.',
			'Controls launcher effects, not Minecraft FPS. Choosing a profile restores its motion and transparency defaults.'
		)}
		><div class="quality-options">
			{#each ['Lite', 'Balanced', 'Experience'] as q (q)}<button
					type="button"
					class="quality-card"
					aria-pressed={appState.settings?.quality_profile === q}
					onclick={() => onQuality(q as QualityProfile)}
					><Sparkles size={18} /><strong>{q}</strong><small
						>{tr(
							q === 'Lite'
								? 'Estático y ligero'
								: q === 'Balanced'
									? 'Movimiento suave'
									: 'Escenas cinematográficas',
							q === 'Lite'
								? 'Static & lightweight'
								: q === 'Balanced'
									? 'Gentle motion'
									: 'Cinematic scenes'
						)}</small
					></button
				>{/each}
		</div></SettingGroup
	>
	<SettingGroup
		id="motion"
		hint={tr(
			'Las preferencias de movimiento reducido del sistema siempre tienen prioridad.',
			'The system’s reduced motion preference always takes priority.'
		)}
	>
		{#each [['reduce_animations', tr('Reducir movimiento', 'Reduce motion')], ['disable_blur_effects', tr('Desactivar desenfoque', 'Disable blur')], ['disable_infinite_animations', tr('Detener efectos continuos', 'Stop continuous effects')]] as [key, label] (key)}<label
				class="toggle"
				><span>{label}</span><input
					type="checkbox"
					checked={Boolean(
						appState.settings?.[
							key as 'reduce_animations' | 'disable_blur_effects' | 'disable_infinite_animations'
						]
					)}
					onchange={(e) =>
						effects(
							key as 'reduce_animations' | 'disable_blur_effects' | 'disable_infinite_animations',
							e.currentTarget.checked
						)}
				/></label
			>{/each}
	</SettingGroup>
	<SettingGroup id="home"
		>{#each ['recent', 'servers', 'activity', 'system'] as panel (panel)}<label class="toggle"
				><span
					>{vr(
						`${panel}Panel` as 'recentPanel' | 'serversPanel' | 'activityPanel' | 'systemPanel'
					)}</span
				><input
					type="checkbox"
					checked={visual.home[panel as keyof typeof visual.home]}
					onchange={(e) => setVisual('home', { ...visual.home, [panel]: e.currentTarget.checked })}
				/></label
			>{/each}</SettingGroup
	>
	{#if error || preferenceStatus.error}<p role="alert">{error || vr('saveError')}</p>{/if}
</div>

<style>
	.appearance-settings {
		display: grid;
		gap: 28px;
	}
	.style-options,
	.theme-options,
	.quality-options {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 12px;
	}
	.theme-options,
	.quality-options {
		grid-template-columns: repeat(3, minmax(0, 1fr));
	}
	button {
		cursor: pointer;
		color: var(--text-primary);
	}
	.style-option,
	.theme-card,
	.quality-card {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: start;
		text-align: left;
		gap: 10px;
		padding: 16px;
		background: var(--surface-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
	}
	button[aria-pressed='true'] {
		border-color: var(--accent);
		box-shadow: 0 0 0 1px var(--accent);
	}
	small {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.6;
	}
	.chosen {
		position: absolute;
		top: 16px;
		right: 16px;
		color: var(--accent);
	}
	.style-art {
		display: block;
		width: 100%;
		height: 70px;
		position: relative;
		border: 1px solid var(--border);
		border-radius: 10px;
		background: var(--bg-main);
		overflow: hidden;
	}
	.style-art span {
		position: absolute;
		top: 10px;
		left: 10px;
		width: 20px;
		height: 50px;
		background: var(--surface-interactive);
		border-radius: 4px;
	}
	.style-art i {
		position: absolute;
		left: 40px;
		right: 14px;
		top: 15px;
		height: 20px;
		background: var(--surface-interactive);
		border-radius: 4px;
	}
	.style-art b {
		position: absolute;
		bottom: 12px;
		left: 40px;
		background: var(--accent);
		height: 9px;
		width: 40px;
		border-radius: 4px;
	}
	.pixel-demo .style-art,
	.pixel-demo .style-art * {
		border-radius: 0;
		box-shadow:
			inset 2px 2px #fff2,
			inset -2px -2px #0008;
	}
	.pixel-demo .style-art {
		background-image: repeating-linear-gradient(0deg, #ffffff08 0 2px, transparent 2px 8px);
	}
	.theme-card {
		padding: 10px;
	}
	.theme-card > span:last-child {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
	}
	.theme-art {
		display: block;
		width: 100%;
		height: 55px;
		background: #171a20;
		border: 1px solid #ffffff18;
		position: relative;
		border-radius: 4px;
	}
	.theme-art i {
		position: absolute;
		inset: 8px auto 8px 8px;
		width: 15%;
		background: #323640;
	}
	.theme-art b {
		position: absolute;
		inset: 8px 8px 8px 29%;
		background: #252932;
		border-radius: 3px;
	}
	.theme-art.light {
		background: #eef1f5;
	}
	.theme-art.light i {
		background: #d1d7e0;
	}
	.theme-art.light b {
		background: white;
	}
	.theme-art.oled {
		background: #000;
	}
	.theme-art.oled i,
	.theme-art.oled b {
		background: #131313;
	}
	.choice-row {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
	.surface-row {
		margin-top: 12px;
	}
	.field {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 14px;
		font-size: 13px;
	}
	select {
		padding: 10px;
		min-width: 150px;
		background: var(--bg-input);
		color: var(--text-primary);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
	}
	.toggle {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 14px;
		min-height: 38px;
		font-size: 13px;
	}
	input[type='checkbox'] {
		accent-color: var(--accent);
		width: 17px;
		height: 17px;
	}
	.quality-card {
		align-items: start;
	}
	.quality-card :global(svg) {
		color: var(--accent);
	}
	@media (max-width: 900px) {
		.quality-options {
			grid-template-columns: 1fr;
		}
		.quality-card {
			flex-direction: row;
			flex-wrap: wrap;
			align-items: center;
		}
		.quality-card small {
			flex-basis: 100%;
		}
		.style-option {
			padding: 12px;
		}
	}
</style>
