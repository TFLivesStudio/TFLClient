<script lang="ts">
	import { onDestroy } from 'svelte';
	import { tr } from '$lib/i18n/experience';
	import type { ModSearchHit, ModVersionSummary } from '$lib/types/types';
	import { Download, Star, ChevronDown, Check, Info, Loader2 } from 'lucide-svelte';
	import { t } from '$lib/i18n/index.svelte';

	/** Fila de resultado — extraída de ContentManager.svelte para que la
	 * vista compacta Y la vista amplia (ContentExplorerView.svelte) la
	 * compartan sin duplicar markup/CSS. Filas modernas, no cards gigantes
	 * (pedido explícito): ícono + nombre + autor + descripción (puede
	 * ocupar 2 líneas) + descargas + estado + instalar + ficha completa. */
	let {
		item,
		installed,
		installing,
		showFavorite = false,
		isFavorite = false,
		hasVersions = true,
		quickInstall = true,
		versionsOpen = false,
		versionOptions = [],
		loadingVersions = false,
		onInstall,
		onToggleFavorite,
		onToggleVersions,
		onOpenDetail
	}: {
		item: ModSearchHit;
		installed: boolean;
		installing: boolean;
		showFavorite?: boolean;
		isFavorite?: boolean;
		hasVersions?: boolean;
		quickInstall?: boolean;
		versionsOpen?: boolean;
		versionOptions?: ModVersionSummary[];
		loadingVersions?: boolean;
		onInstall: (versionId?: string) => void;
		onToggleFavorite?: () => void;
		onToggleVersions?: () => void;
		onOpenDetail: () => void;
	} = $props();

	let detailTimer: ReturnType<typeof setTimeout> | undefined;
	onDestroy(() => clearTimeout(detailTimer));
	function detailClick(e: MouseEvent) {
		if (!quickInstall || e.detail === 0) {
			onOpenDetail();
			return;
		}
		clearTimeout(detailTimer);
		if (e.detail < 2) detailTimer = setTimeout(onOpenDetail, 550);
	}
	function quick(e: MouseEvent) {
		if (!quickInstall) return;
		clearTimeout(detailTimer);
		e.preventDefault();
		e.stopPropagation();
		if (!installed && !installing) onInstall();
	}

	function formatDownloads(n: number): string {
		if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
		if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
		return String(n);
	}
</script>

<div class="result-row anim-fade-in">
	<button
		type="button"
		class="result-icon-btn"
		onclick={detailClick}
		ondblclick={quick}
		aria-label={item.title}
	>
		{#if item.icon_url}
			<img src={item.icon_url} alt="" />
		{:else}
			<div class="result-icon-fallback"></div>
		{/if}
	</button>
	<div class="result-info">
		<button
			type="button"
			class="result-title-btn"
			onclick={detailClick}
			ondblclick={quick}
			title={quickInstall
				? tr(
						'Doble clic para instalar; un clic abre la ficha',
						'Double-click to install; single click opens details'
					)
				: undefined}
		>
			<span class="result-title">{item.title}</span>
		</button>
		<span class="result-meta">
			{item.author} · {formatDownloads(item.downloads)}
		</span>
		<p class="result-desc">{item.description}</p>
	</div>
	<div class="result-actions">
		{#if showFavorite}
			<button
				type="button"
				class="favorite-btn"
				class:active={isFavorite}
				onclick={onToggleFavorite}
				aria-label={t('contentManager.favorite')}
			>
				<Star size={14} fill={isFavorite ? 'currentColor' : 'none'} />
			</button>
		{/if}
		<button
			type="button"
			class="detail-btn"
			onclick={onOpenDetail}
			aria-label={t('contentManager.viewDetail')}
			title={t('contentManager.viewDetail')}
		>
			<Info size={14} />
		</button>
		{#if hasVersions}
			<button
				type="button"
				class="version-toggle"
				class:open={versionsOpen}
				onclick={onToggleVersions}
				aria-label={t('contentManager.viewVersions')}
				title={t('contentManager.viewVersions')}
			>
				<ChevronDown size={13} />
			</button>
		{/if}
		{#if installed}
			<span class="installed-badge" title={t('contentManager.alreadyInstalled')}
				><Check size={13} /></span
			>
		{:else}
			<button
				type="button"
				class="install-btn"
				aria-label={`${tr('Instalar', 'Install')} ${item.title}`}
				disabled={installing}
				onclick={() => onInstall()}
			>
				{#if installing}<Loader2 size={14} class="spin" />{:else}<Download size={14} />{/if}
			</button>
		{/if}
	</div>
</div>
{#if hasVersions && versionsOpen}
	<div class="version-row">
		{#if loadingVersions}
			<Loader2 size={13} class="spin" />
		{:else if versionOptions.length === 0}
			<span class="hint">{t('contentManager.noCompatibleVersions')}</span>
		{:else}
			{#each versionOptions as v (v.id)}
				<button
					type="button"
					class="version-option"
					disabled={installing}
					onclick={() => onInstall(v.id)}
				>
					{v.version_number || v.name}
				</button>
			{/each}
		{/if}
	</div>
{/if}

<style>
	.result-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-card);
	}

	.result-icon-btn {
		padding: 0;
		border: none;
		background: transparent;
		cursor: pointer;
		flex-shrink: 0;
	}

	.result-icon-btn img,
	.result-icon-fallback {
		display: block;
		width: 36px;
		height: 36px;
		border-radius: var(--border-radius-sm);
		flex-shrink: 0;
		background: var(--bg-input);
	}

	.result-info {
		flex: 1;
		min-width: 0;
	}

	.result-title-btn {
		display: block;
		padding: 0;
		border: none;
		background: transparent;
		cursor: pointer;
		text-align: left;
	}

	.result-title {
		font-size: 0.82rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.result-title-btn:hover .result-title {
		color: var(--accent);
	}

	.result-meta {
		display: block;
		font-size: 0.66rem;
		color: var(--text-muted);
		margin: 1px 0 2px;
	}

	.result-desc {
		font-size: 0.7rem;
		color: var(--text-muted);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.result-actions {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
	}

	.favorite-btn,
	.detail-btn,
	.version-toggle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-muted);
		cursor: pointer;
	}

	.favorite-btn.active {
		color: #f5c518;
		border-color: color-mix(in srgb, #f5c518 45%, var(--border));
	}

	.version-toggle.open {
		color: var(--accent);
		border-color: var(--accent);
	}

	.installed-badge {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		border: 1px solid color-mix(in srgb, #22c55e 45%, var(--border));
		background: color-mix(in srgb, #22c55e 16%, var(--bg-input));
		color: #22c55e;
	}

	.install-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		cursor: pointer;
		flex-shrink: 0;
	}

	.install-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.version-row {
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
		padding: 6px 10px 10px;
		margin-top: -4px;
	}

	.version-option {
		padding: 4px 9px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.68rem;
		cursor: pointer;
	}

	.version-option:hover:not(:disabled) {
		border-color: var(--accent);
		color: var(--accent);
	}

	.version-option:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
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
