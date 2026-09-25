<script lang="ts">
	import { onMount } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { Image } from '@tauri-apps/api/image';
	import { writeImage } from '@tauri-apps/plugin-clipboard-manager';
	import type { InstanceData, ScreenshotInfo } from '$lib/types/types';
	import { getInstanceScreenshots, deleteScreenshot, openScreenshotsFolder } from '$lib/api/tflApi';
	import { FolderOpen, Trash2, X, ImageOff, Copy, Check } from 'lucide-svelte';
	import { t } from '$lib/i18n/index.svelte';

	let { instance }: { instance: InstanceData } = $props();

	let shots = $state<ScreenshotInfo[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let preview = $state<ScreenshotInfo | null>(null);
	let copiedFilename = $state<string | null>(null);
	let copyTimer: ReturnType<typeof setTimeout> | undefined;
	let zoom = $state(1);

	function openPreview(shot: ScreenshotInfo) {
		preview = shot;
		zoom = 1;
	}

	function handleWheel(e: WheelEvent) {
		e.preventDefault();
		zoom = Math.min(4, Math.max(1, zoom - e.deltaY * 0.0015));
	}

	async function refresh() {
		loading = true;
		error = null;
		try {
			shots = await getInstanceScreenshots(instance.name);
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	onMount(refresh);

	async function handleCopy(shot: ScreenshotInfo, e: MouseEvent) {
		e.stopPropagation();
		try {
			const img = await Image.fromPath(shot.path);
			await writeImage(img);
			copiedFilename = shot.filename;
			clearTimeout(copyTimer);
			copyTimer = setTimeout(() => (copiedFilename = null), 1500);
		} catch (err) {
			error = String(err);
		}
	}

	async function handleDelete(shot: ScreenshotInfo, e: MouseEvent) {
		e.stopPropagation();
		try {
			await deleteScreenshot(instance.name, shot.filename);
			shots = shots.filter((s) => s.filename !== shot.filename);
			if (preview?.filename === shot.filename) preview = null;
		} catch (err) {
			error = String(err);
		}
	}

	function fmtDate(ms: number): string {
		if (!ms) return '';
		return new Date(ms).toLocaleString();
	}
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && (preview = null)} />

<div class="screenshots-panel">
	<div class="toolbar">
		<p class="hint">
			{t('screenshotsPanel.hint')}
		</p>
		<button type="button" class="folder-btn" onclick={() => openScreenshotsFolder(instance.name)}>
			<FolderOpen size={13} /> {t('screenshotsPanel.openFolder')}
		</button>
	</div>

	{#if error}
		<p class="error">{error}</p>
	{/if}

	{#if loading}
		<p class="hint">{t('instanceDetail.loading')}</p>
	{:else if shots.length === 0}
		<div class="empty">
			<ImageOff size={22} />
			<p>{t('screenshotsPanel.empty')}</p>
		</div>
	{:else}
		<div class="grid">
			{#each shots as shot (shot.filename)}
				<div
					class="thumb anim-fade-in"
					onclick={() => openPreview(shot)}
					onkeydown={(e) => e.key === 'Enter' && openPreview(shot)}
					role="button"
					tabindex="0"
				>
					<img src={convertFileSrc(shot.path)} alt={shot.filename} loading="lazy" />
					<span class="thumb-date">{fmtDate(shot.modified_ms)}</span>
					<div class="thumb-actions">
						<button
							type="button"
							class="thumb-action-btn"
							onclick={(e) => handleCopy(shot, e)}
							aria-label={t('screenshotsPanel.copyImage')}
						>
							{#if copiedFilename === shot.filename}<Check size={12} />{:else}<Copy size={12} />{/if}
						</button>
						<button
							type="button"
							class="thumb-action-btn thumb-action-danger"
							onclick={(e) => handleDelete(shot, e)}
							aria-label={t('screenshotsPanel.delete')}
						>
							<Trash2 size={12} />
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if preview}
	<div
		class="lightbox"
		onclick={() => (preview = null)}
		onkeydown={(e) => e.key === 'Escape' && (preview = null)}
		role="button"
		tabindex="-1"
	>
		<div class="lightbox-actions">
			<button type="button" class="lightbox-btn" onclick={(e) => handleCopy(preview!, e)} aria-label={t('screenshotsPanel.copyImage')}>
				{#if copiedFilename === preview.filename}<Check size={16} />{:else}<Copy size={16} />{/if}
			</button>
			<button type="button" class="lightbox-btn" onclick={() => (preview = null)} aria-label={t('settings.close')}>
				<X size={18} />
			</button>
		</div>
		<div
			class="lightbox-frame"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			onwheel={handleWheel}
			role="dialog"
			aria-modal="true"
			tabindex="-1"
		>
			<img
				src={convertFileSrc(preview.path)}
				alt={preview.filename}
				style="transform: scale({zoom})"
			/>
		</div>
	</div>
{/if}

<style>
	.screenshots-panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.toolbar {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
		flex: 1;
	}

	.folder-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 7px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.76rem;
		font-weight: 600;
		cursor: pointer;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.folder-btn:hover {
		border-color: var(--accent);
		color: var(--accent);
	}

	.error {
		color: var(--color-error);
		font-size: 0.78rem;
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 32px 0;
		color: var(--text-muted);
		text-align: center;
	}

	.empty p {
		font-size: 0.78rem;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
	}

	.thumb {
		position: relative;
		aspect-ratio: 16 / 9;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		overflow: hidden;
		padding: 0;
		cursor: pointer;
		background: var(--bg-input);
	}

	.thumb img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.thumb-date {
		position: absolute;
		left: 0;
		right: 0;
		bottom: 0;
		padding: 4px 6px;
		font-size: 0.62rem;
		color: #ffffff;
		background: linear-gradient(transparent, rgba(0, 0, 0, 0.65));
		text-align: left;
		opacity: 0;
		transition: opacity 0.15s;
	}

	.thumb:hover .thumb-date {
		opacity: 1;
	}

	.thumb-actions {
		position: absolute;
		top: 4px;
		right: 4px;
		display: flex;
		gap: 4px;
		opacity: 0;
		transition: opacity 0.15s;
	}

	.thumb:hover .thumb-actions {
		opacity: 1;
	}

	.thumb-action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: rgba(0, 0, 0, 0.55);
		color: #ffffff;
		cursor: pointer;
	}

	.thumb-action-btn:hover {
		background: rgba(0, 0, 0, 0.8);
	}

	.thumb-action-danger:hover {
		background: var(--color-error);
	}

	.lightbox {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.85);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 400;
		padding: 80px;
		overflow: hidden;
	}

	.lightbox-frame {
		max-width: 78vw;
		max-height: 78vh;
		display: flex;
		overflow: visible;
		cursor: zoom-in;
	}

	.lightbox-frame img {
		max-width: 100%;
		max-height: 100%;
		border-radius: var(--border-radius);
		box-shadow: var(--shadow-lg);
		transition: transform 0.05s linear;
	}

	.lightbox-actions {
		position: absolute;
		top: 16px;
		right: 16px;
		display: flex;
		gap: 8px;
	}

	.lightbox-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: none;
		background: rgba(255, 255, 255, 0.1);
		color: #ffffff;
		cursor: pointer;
	}

	.lightbox-btn:hover {
		background: rgba(255, 255, 255, 0.2);
	}
</style>
