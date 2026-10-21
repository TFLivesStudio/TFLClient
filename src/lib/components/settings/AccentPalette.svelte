<script lang="ts">
	import { Check, Pipette, RotateCcw } from 'lucide-svelte';
	import { visual, setVisual } from '$lib/visual/preferences.svelte';
	import { tr } from '$lib/i18n/experience';
	import { accentText } from '$lib/visual/policy';
	let { onReset }: { onReset: () => void } = $props();
	const colors = [
		['TFL', '#ff7a2e'],
		['Apricot', '#fb923c'],
		['Ruby', '#ef4444'],
		['Coral', '#fb7185'],
		['Rose', '#f43f5e'],
		['Pink', '#ec4899'],
		['Orchid', '#d946ef'],
		['Plum', '#ab4aba'],
		['Violet', '#8b5cf6'],
		['Iris', '#5b5bd6'],
		['Indigo', '#6366f1'],
		['Cobalt', '#3b82f6'],
		['Sky', '#38bdf8'],
		['Cyan', '#00a2c7'],
		['Teal', '#14b8a6'],
		['Jade', '#29a383'],
		['Emerald', '#10b981'],
		['Grass', '#46a758'],
		['Lime', '#84cc16'],
		['Lemon', '#eab308'],
		['Gold', '#d4a044'],
		['Copper', '#b77952'],
		['Silver', '#94a3b8'],
		['Graphite', '#6b7280']
	];
	let custom = $state(false),
		hex = $state(visual.customAccent || '#ff7a2e');

	const active = $derived(visual.customAccent || getLegacy());
	function getLegacy() {
		const key = typeof localStorage !== 'undefined' ? localStorage.getItem('tfl-accent') : '';
		const map: Record<string, string> = {
			orange: '#ff7a2e',
			violet: '#8b5cf6',
			teal: '#14b8a6',
			blue: '#3b82f6',
			rose: '#f43f5e',
			lime: '#84cc16',
			iris: '#5b5bd6',
			jade: '#29a383',
			crimson: '#e93d82',
			cyan: '#00a2c7',
			grass: '#46a758',
			plum: '#ab4aba'
		};
		return map[key ?? 'orange'] ?? '#ff7a2e';
	}
	function apply(value: string) {
		hex = value;
		if (/^#[\da-f]{6}$/i.test(value)) setVisual('customAccent', value);
	}
</script>

<div class="palette" aria-label={tr('Colores de acento', 'Accent colors')}>
	{#each colors as [name, color] (color)}<button
			type="button"
			class="accent-swatch"
			aria-label={name}
			title={name}
			aria-pressed={active.toLowerCase() === color}
			style:background={color}
			style:color={accentText(color)}
			onclick={() => {
				custom = false;
				apply(color);
			}}
			>{#if active.toLowerCase() === color}<Check size={18} />{/if}</button
		>{/each}
	<button
		type="button"
		class="accent-swatch custom"
		aria-label={tr('Acento personalizado', 'Custom accent')}
		aria-expanded={custom}
		onclick={() => {
			custom = !custom;
			hex = active;
		}}><Pipette size={19} /></button
	>
</div>
{#if custom}<div class="custom-editor">
		<input
			type="color"
			aria-label={tr('Elegir color', 'Pick a color')}
			value={/^#[\da-f]{6}$/i.test(hex) ? hex : active}
			oninput={(e) => apply(e.currentTarget.value)}
		/><label
			>HEX<input
				type="text"
				aria-label={tr('Color hexadecimal', 'Hex color')}
				value={hex}
				maxlength="7"
				spellcheck="false"
				aria-invalid={!/^#[\da-f]{6}$/i.test(hex)}
				oninput={(e) => apply(e.currentTarget.value)}
			/></label
		><span class="sample" style:background={active} style:color={accentText(active)}
			>{tr('Vista previa', 'Preview')}</span
		>
	</div>{/if}
<button
	type="button"
	class="reset"
	onclick={() => {
		onReset();
		setVisual('customAccent', '');
		hex = '#ff7a2e';
		custom = false;
	}}><RotateCcw size={13} />{tr('Restaurar naranja TFL', 'Restore TFL orange')}</button
>

<style>
	.palette {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(40px, 1fr));
		gap: 10px;
		max-width: 700px;
	}
	.accent-swatch {
		height: 40px;
		min-width: 40px;
		display: grid;
		place-items: center;
		border: 2px solid transparent;
		border-radius: var(--border-radius-sm);
		padding: 0;
		cursor: pointer;
		box-shadow: inset 0 0 0 1px #0002;
	}
	.accent-swatch[aria-pressed='true'] {
		outline: 2px solid var(--text-primary);
		outline-offset: 3px;
	}
	.custom {
		background: conic-gradient(#ff7a2e, #e93d82, #8b5cf6, #3b82f6, #29a383, #eab308, #ff7a2e);
		color: #fff;
		text-shadow: 0 1px 2px #000;
	}
	.custom :global(svg) {
		filter: drop-shadow(0 1px 1px #000);
	}
	.custom-editor {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-top: 20px;
		flex-wrap: wrap;
	}
	.custom-editor > input {
		width: 52px;
		height: 44px;
		min-height: 0;
		border: 0;
		padding: 0;
		background: none;
		cursor: pointer;
	}
	label {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 12px;
		color: var(--text-secondary);
	}
	label input {
		width: 115px;
		padding: 10px;
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-family: monospace;
	}
	.sample {
		padding: 12px;
		border-radius: var(--border-radius-sm);
		font-size: 12px;
	}
	.reset {
		display: flex;
		gap: 6px;
		align-items: center;
		border: 0;
		background: none;
		color: var(--text-secondary);
		font-size: 11px;
		cursor: pointer;
		margin-top: 16px;
		padding: 0;
	}
</style>
