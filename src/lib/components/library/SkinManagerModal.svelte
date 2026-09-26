<script lang="ts">
	import { onMount } from 'svelte';
	import {
		getMojangProfile,
		setSkinFromFile,
		resetSkin,
		setActiveCape,
		hideCape,
		pickImageFile
	} from '$lib/api/tflApi';
	import { appState } from '$lib/state/state.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import type { MojangProfile } from '$lib/types/types';
	import { X, Upload, RotateCcw, Loader2, Ban } from 'lucide-svelte';

	let { onClose }: { onClose: () => void } = $props();

	let profile = $state<MojangProfile | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let busy = $state<string | null>(null);
	let tab = $state<'skin' | 'cape'>('skin');
	let variant = $state<'classic' | 'slim'>('classic');

	// Subir skin usa el diálogo nativo de archivos, mismo riesgo conocido que
	// ícono/wallpaper propio (ver InstanceIconPicker.svelte) — mismo gate.
	const dialogsBlocked = $derived(appState.settings?.native_dialog_mode !== 'manual');
	const activeSkin = $derived(profile?.skins.find((s) => s.state === 'ACTIVE') ?? null);
	const activeCape = $derived(profile?.capes.find((c) => c.state === 'ACTIVE') ?? null);

	async function load() {
		loading = true;
		error = null;
		try {
			profile = await getMojangProfile();
			const current = profile.skins.find((s) => s.state === 'ACTIVE');
			if (current) variant = current.variant.toLowerCase() as 'classic' | 'slim';
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}
	onMount(load);

	async function uploadSkin() {
		if (dialogsBlocked) return;
		const path = await pickImageFile();
		if (!path) return;
		busy = 'upload';
		error = null;
		try {
			await setSkinFromFile(path, variant);
			await load();
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
	}

	async function doResetSkin() {
		busy = 'reset';
		error = null;
		try {
			await resetSkin();
			await load();
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
	}

	async function chooseCape(capeId: string) {
		busy = capeId;
		error = null;
		try {
			await setActiveCape(capeId);
			await load();
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
	}

	async function removeCape() {
		busy = 'none';
		error = null;
		try {
			await hideCape();
			await load();
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
		aria-modal="true"
		tabindex="-1"
	>
		<div class="panel-header">
			<h2>{t('skinManager.title')}</h2>
			<button type="button" class="close-btn" onclick={onClose} aria-label={t('settings.close')}>
				<X size={16} />
			</button>
		</div>

		<div class="tabs">
			<button type="button" class="tab-btn" class:active={tab === 'skin'} onclick={() => (tab = 'skin')}>
				{t('skinManager.tabSkin')}
			</button>
			<button type="button" class="tab-btn" class:active={tab === 'cape'} onclick={() => (tab = 'cape')}>
				{t('skinManager.tabCape')}
			</button>
		</div>

		{#if loading}
			<div class="loading-row"><Loader2 size={20} class="spin" /></div>
		{:else}
			{#if error}<p class="error">{error}</p>{/if}

			{#if tab === 'skin'}
				<div class="skin-section">
					<span class="skin-preview" style={activeSkin ? `background-image: url(${activeSkin.url})` : ''}></span>
					<div class="skin-controls">
						<div class="variant-tabs">
							<button
								type="button"
								class="variant-btn"
								class:active={variant === 'classic'}
								onclick={() => (variant = 'classic')}
							>
								{t('skinManager.variantClassic')}
							</button>
							<button
								type="button"
								class="variant-btn"
								class:active={variant === 'slim'}
								onclick={() => (variant = 'slim')}
							>
								{t('skinManager.variantSlim')}
							</button>
						</div>
						<button
							type="button"
							class="action-btn primary"
							disabled={busy === 'upload' || dialogsBlocked}
							title={dialogsBlocked ? t('contentManager.manualModeRequired') : undefined}
							onclick={uploadSkin}
						>
							{#if busy === 'upload'}<Loader2 size={14} class="spin" />{:else}<Upload size={14} />{/if}
							{t('skinManager.uploadSkin')}
						</button>
						<button type="button" class="action-btn" disabled={busy === 'reset'} onclick={doResetSkin}>
							{#if busy === 'reset'}<Loader2 size={14} class="spin" />{:else}<RotateCcw size={14} />{/if}
							{t('skinManager.resetSkin')}
						</button>
						{#if dialogsBlocked}
							<p class="hint">{t('instanceIconPicker.autoModeNotice')}</p>
						{/if}
					</div>
				</div>
			{:else if profile && profile.capes.length === 0}
				<p class="hint">{t('skinManager.noCapes')}</p>
			{:else if profile}
				<div class="cape-grid">
					<button
						type="button"
						class="cape-swatch none"
						class:active={!activeCape}
						disabled={busy === 'none'}
						onclick={removeCape}
					>
						{#if busy === 'none'}<Loader2 size={16} class="spin" />{:else}<Ban size={20} />{/if}
						<span>{t('skinManager.noCape')}</span>
					</button>
					{#each profile.capes as cape (cape.id)}
						<button
							type="button"
							class="cape-swatch"
							class:active={cape.state === 'ACTIVE'}
							disabled={busy === cape.id}
							onclick={() => chooseCape(cape.id)}
						>
							{#if busy === cape.id}
								<Loader2 size={16} class="spin" />
							{:else}
								<img src={cape.url} alt={cape.alias ?? ''} />
							{/if}
							<span>{cape.alias ?? '—'}</span>
						</button>
					{/each}
				</div>
			{/if}
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
		width: min(480px, calc(100vw - 32px));
		max-height: 78vh;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 20px;
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 14px;
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

	.tabs {
		display: flex;
		gap: 6px;
	}

	.tab-btn {
		flex: 1;
		padding: 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.8rem;
		font-weight: 700;
		cursor: pointer;
	}

	.tab-btn.active {
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		border-color: var(--accent);
		color: var(--text-primary);
	}

	.loading-row {
		display: flex;
		justify-content: center;
		padding: 32px;
		color: var(--text-muted);
	}

	.error {
		color: var(--color-error);
		font-size: 0.78rem;
	}

	.hint {
		font-size: 0.72rem;
		color: var(--text-muted);
	}

	.skin-section {
		display: flex;
		gap: 16px;
		align-items: flex-start;
	}

	.skin-preview {
		flex-shrink: 0;
		display: block;
		width: 96px;
		height: 96px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background-color: var(--bg-input);
		background-repeat: no-repeat;
		background-size: 768px 768px;
		background-position: -96px -96px;
		image-rendering: pixelated;
	}

	.skin-controls {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.variant-tabs {
		display: flex;
		gap: 6px;
	}

	.variant-btn {
		flex: 1;
		padding: 6px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.74rem;
		font-weight: 600;
		cursor: pointer;
	}

	.variant-btn.active {
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		border-color: var(--accent);
		color: var(--text-primary);
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.78rem;
		font-weight: 600;
		cursor: pointer;
	}

	.action-btn.primary {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		color: var(--accent);
	}

	.action-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.cape-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
		overflow-y: auto;
	}

	.cape-swatch {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 10px 6px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		cursor: pointer;
	}

	.cape-swatch.active {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		color: var(--text-primary);
	}

	.cape-swatch img {
		width: 100%;
		height: 44px;
		object-fit: contain;
		image-rendering: pixelated;
	}

	.cape-swatch.none {
		justify-content: center;
		min-height: 66px;
	}

	.cape-swatch span {
		font-size: 0.68rem;
		text-align: center;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 100%;
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
</style>
