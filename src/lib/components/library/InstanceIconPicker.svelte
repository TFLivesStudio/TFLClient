<script lang="ts">
	import { modalFocus } from '$lib/visual/modalFocus';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { pickImageFile, setInstanceIcon } from '$lib/api';
	import { appState } from '$lib/state/state.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { tr } from '$lib/i18n/experience';
	import { normalized } from '$lib/settings/catalog';
	import { X, Upload, Loader2, Search, Check } from 'lucide-svelte';

	let {
		instanceName,
		onClose,
		onChanged
	}: {
		instanceName: string;
		onClose: () => void;
		onChanged: () => void;
	} = $props();

	interface IconPreset {
		id: string;
		data_url: string;
	}

	let presets = $state<IconPreset[]>([]);
	let loading = $state(true);
	let query = $state(''),
		category = $state('all'),
		selected = $state<IconPreset | null>(null);
	const categories = [
		['all', () => tr('Todos', 'All')],
		['nature', () => tr('Naturaleza', 'Nature')],
		['biome', () => tr('Paisajes', 'Landscapes')],
		['artifact', () => tr('Artefactos', 'Artifacts')],
		['creature', () => tr('Criaturas', 'Creatures')],
		['sigil', () => tr('Emblemas', 'Emblems')],
		['gear', () => tr('Equipo', 'Gear')],
		['legacy', () => tr('Clásicos', 'Legacy')]
	] as const;
	function group(id: string) {
		return /^(nature|biome|artifact|creature|sigil|gear)-/.exec(id)?.[1] ?? 'legacy';
	}
	const filtered = $derived(
		presets.filter(
			(p) =>
				(category === 'all' || group(p.id) === category) &&
				normalized(label(p.id)).includes(normalized(query))
		)
	);
	let busy = $state<string | null>(null);
	let error = $state<string | null>(null);

	// Subir imagen propia usa el diálogo nativo de archivos, que crashea el
	// launcher entero en macOS con la firma ad-hoc del build (ver
	// CHANGELOG_macos-dialog-crash-java26.txt) — por eso queda detrás del
	// modo global "Automático/Manual" (Ajustes), no de la plataforma.
	let dialogsBlocked = $derived(appState.settings?.native_dialog_mode !== 'manual');

	onMount(async () => {
		try {
			presets = await invoke<IconPreset[]>('list_instance_icon_presets');
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	});

	function label(id: string) {
		const name = id
			.replace(/^(mob|place|nature|biome|artifact|creature|sigil|gear)-/, '')
			.replace(/[_-]/g, ' ');
		return name.charAt(0).toUpperCase() + name.slice(1);
	}

	async function choosePreset(id: string) {
		busy = id;
		error = null;
		try {
			await invoke('set_instance_icon_from_preset', { name: instanceName, presetId: id });
			onChanged();
			onClose();
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
	}

	async function uploadOwn() {
		if (dialogsBlocked) return;
		busy = 'upload';
		error = null;
		try {
			const source = await pickImageFile();
			if (!source) return;
			await setInstanceIcon(instanceName, source);
			onChanged();
			onClose();
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
	}
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div
	class="overlay"
	onclick={onClose}
	onkeydown={(e) => e.key === 'Escape' && onClose()}
	role="button"
	tabindex="-1"
>
	<div
		class="panel"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		use:modalFocus={onClose}
		aria-modal="true"
		tabindex="-1"
	>
		<div class="panel-header">
			<h2>{t('instanceIconPicker.title')}</h2>
			<button type="button" class="close-btn" onclick={onClose} aria-label={t('settings.close')}>
				<X size={16} />
			</button>
		</div>

		<button
			type="button"
			class="upload-btn"
			disabled={busy === 'upload' || dialogsBlocked}
			title={dialogsBlocked ? t('contentManager.manualModeRequired') : undefined}
			onclick={uploadOwn}
		>
			{#if busy === 'upload'}<Loader2 size={14} class="spin" />{:else}<Upload size={14} />{/if}
			{t('instanceIconPicker.uploadOwn')}
		</button>
		{#if dialogsBlocked}
			<p class="mac-notice">{t('instanceIconPicker.autoModeNotice')}</p>
		{/if}

		{#if error}<p class="error">{error}</p>{/if}

		{#if loading}
			<div class="loading-row"><Loader2 size={18} class="spin" /></div>
		{:else}
			<label class="icon-search"
				><Search size={14} /><input
					type="search"
					bind:value={query}
					placeholder={tr('Buscar iconos…', 'Search icons…')}
					aria-label={tr('Buscar iconos', 'Search icons')}
				/></label
			>
			<div class="categories">
				{#each categories as [id, name] (id)}<button
						type="button"
						class="vr-control"
						aria-pressed={category === id}
						onclick={() => (category = id)}>{name()}</button
					>{/each}
			</div>
			{#if selected}<div class="icon-preview">
					<img src={selected.data_url} alt={label(selected.id)} /><span
						><strong>{label(selected.id)}</strong><small
							>{tr('Vista previa del icono', 'Icon preview')}</small
						></span
					><button
						type="button"
						class="vr-control"
						disabled={!!busy}
						onclick={() => {
							if (selected) void choosePreset(selected.id);
						}}><Check size={14} />{tr('Aplicar', 'Apply')}</button
					>
				</div>{/if}
			<div class="icon-grid">
				{#each filtered as p (p.id)}
					<button
						type="button"
						class="icon-swatch"
						disabled={busy === p.id}
						onclick={() => (selected = p)}
						aria-pressed={selected?.id === p.id}
						aria-label={label(p.id)}
						title={label(p.id)}
					>
						{#if busy === p.id}
							<Loader2 size={16} class="spin" />
						{:else}
							<img src={p.data_url} alt="" />
						{/if}
					</button>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.panel {
		width: min(520px, calc(100vw - 32px));
		max-height: 78vh;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 20px;
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 12px;
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.panel-header h2 {
		font-size: var(--text-lg);
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.upload-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 9px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		color: var(--accent);
		font-size: 0.82rem;
		font-weight: 700;
		cursor: pointer;
		flex-shrink: 0;
	}

	.upload-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.error {
		color: var(--color-error);
		font-size: 0.78rem;
	}

	.mac-notice {
		color: var(--text-secondary, #888);
		font-size: 0.72rem;
	}

	.loading-row {
		display: flex;
		justify-content: center;
		padding: 24px;
		color: var(--text-muted);
	}

	.icon-grid {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 6px;
		overflow-y: auto;
		padding-right: 2px;
	}

	.icon-swatch {
		aspect-ratio: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 6px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		cursor: pointer;
		transition:
			border-color 0.15s,
			transform 0.15s;
	}

	.icon-swatch:hover {
		border-color: var(--accent);
		transform: translateY(-1px);
	}

	.icon-swatch:disabled {
		cursor: not-allowed;
		opacity: 0.6;
	}

	.icon-swatch img {
		width: 100%;
		height: 100%;
		object-fit: contain;
		image-rendering: pixelated;
	}

	:global(.spin) {
		animation: tfl-spin 0.8s linear infinite;
	}

	@keyframes tfl-spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	.panel-header h2 {
		font-size: var(--text-heading);
		letter-spacing: -0.035em;
	}
	.panel-header {
		padding-bottom: 18px;
		margin-bottom: 8px;
	}
	.panel {
		padding: 26px;
		background: var(--surface-elevated);
	}
	.close-btn {
		width: 34px;
		height: 34px;
	}
	.icon-search {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 10px;
		background: var(--bg-input);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		color: var(--text-secondary);
	}
	.icon-search input {
		border: 0;
		background: none;
		color: var(--text-primary);
		font-size: 12px;
		min-width: 0;
		flex: 1;
		outline: 0;
	}
	.categories {
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
	}
	.categories button {
		font-size: 10px;
		padding: 5px 8px;
	}
	.categories button[aria-pressed='true'],
	.icon-swatch[aria-pressed='true'] {
		border-color: var(--accent);
	}
	.icon-preview {
		display: flex;
		gap: 12px;
		align-items: center;
		padding: 10px 0;
		border-block: 1px solid var(--border);
	}
	.icon-preview > img {
		width: 56px;
		height: 56px;
		object-fit: contain;
		image-rendering: pixelated;
	}
	.icon-preview > span {
		flex: 1;
		display: grid;
		gap: 5px;
		font-size: 13px;
	}
	.icon-preview small {
		font-size: 11px;
		color: var(--text-secondary);
	}
	.icon-preview button {
		display: flex;
		align-items: center;
		gap: 5px;
	}
	.icon-grid {
		min-height: 70px;
	}
	.icon-swatch {
		padding: 4px;
	}
	.icon-swatch img {
		border: 0;
		box-shadow: none;
		background: none;
	}
</style>
