<script lang="ts">
	import { modalFocus } from '$lib/visual/modalFocus';
	import { onMount } from 'svelte';
	import { searchModpacks, getModpackProjectDetail, installModpack } from '$lib/api/modpacks';
	import { createInstance } from '$lib/api';
	import { getModVersions } from '$lib/api/content';
	import type {
		ModSearchHit,
		ContentSearchPage,
		SortIndex,
		InstanceData,
		Loader,
		ContentProjectDetail
	} from '$lib/types/types';
	import { X, Search, Download, Loader2, Check, PackageOpen } from 'lucide-svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import SortSelect from '$lib/components/ui/SortSelect.svelte';
	import ProjectDetailView from './ProjectDetailView.svelte';
	import { t } from '$lib/i18n/index.svelte';

	let {
		onClose,
		onInstanceCreated
	}: {
		onClose: () => void;
		onInstanceCreated?: (instance: InstanceData) => void;
	} = $props();

	let query = $state('');
	let debouncedQuery = $state('');
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;
	let sortIndex = $state<SortIndex>('downloads');
	let offset = $state(0);
	const limit = 12;

	let loading = $state(true);
	let hits = $state<ModSearchHit[]>([]);
	let totalHits = $state(0);
	let error = $state<string | null>(null);

	// Detalle de proyecto
	let detailProjectId = $state<string | null>(null);
	let detailHit = $state<ModSearchHit | null>(null);
	let detailData = $state<ContentProjectDetail | null>(null);
	let detailLoading = $state(false);
	let detailError = $state<string | null>(null);

	// Estado de instalación
	let installingId = $state<string | null>(null);
	let installSuccessId = $state<string | null>(null);
	let installError = $state<string | null>(null);

	async function runSearch() {
		loading = true;
		error = null;
		try {
			const res: ContentSearchPage = await searchModpacks(
				debouncedQuery,
				'',
				'fabric' as Loader,
				[],
				offset,
				limit,
				sortIndex
			);
			hits = res.hits;
			totalHits = res.total_hits;
		} catch (e) {
			error = String(e);
			hits = [];
			totalHits = 0;
		} finally {
			loading = false;
		}
	}

	function handleQueryInput(e: Event) {
		const val = (e.target as HTMLInputElement).value;
		query = val;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			debouncedQuery = val.trim();
			offset = 0;
			runSearch();
		}, 300);
	}

	function handleSortChange(idx: SortIndex) {
		sortIndex = idx;
		offset = 0;
		runSearch();
	}

	function handlePrev() {
		if (offset >= limit) {
			offset -= limit;
			runSearch();
		}
	}

	function handleNext() {
		if (offset + limit < totalHits) {
			offset += limit;
			runSearch();
		}
	}

	async function openDetail(hit: ModSearchHit) {
		detailProjectId = hit.project_id;
		detailHit = hit;
		detailLoading = true;
		detailError = null;
		detailData = null;
		try {
			detailData = await getModpackProjectDetail(hit.project_id);
		} catch (e) {
			detailError = String(e);
		} finally {
			detailLoading = false;
		}
	}

	async function handleInstall(hit: ModSearchHit, versionId?: string) {
		void versionId;
		installingId = hit.project_id;
		installError = null;
		try {
			const detail = detailData ?? (await getModpackProjectDetail(hit.project_id));
			const mcVer = detail.game_versions[0] || '1.20.1';
			const rawLoader = detail.loaders[0]?.toLowerCase() || 'fabric';
			const loader: Loader = ['fabric', 'quilt', 'forge', 'neoforge'].includes(rawLoader)
				? (rawLoader as Loader)
				: 'fabric';

			const inst = await createInstance(hit.title.trim(), mcVer, loader);
			await installModpack(inst.name, hit.project_id, mcVer, loader);

			installSuccessId = hit.project_id;
			onInstanceCreated?.(inst);
			setTimeout(() => {
				onClose();
			}, 900);
		} catch (e) {
			installError = String(e);
		} finally {
			installingId = null;
		}
	}

	onMount(() => {
		runSearch();
	});
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
			<div class="title-row">
				<PackageOpen size={20} class="header-icon" />
				<div>
					<h2>Modpacks de Modrinth</h2>
					<p class="subtitle">
						Explora la comunidad de modpacks y crea una instancia directamente.
					</p>
				</div>
			</div>
			<button type="button" class="close-btn" onclick={onClose} aria-label={t('settings.close')}>
				<X size={16} />
			</button>
		</div>

		<div class="search-bar">
			<div class="input-wrap" data-search-control>
				<Search size={16} class="search-icon" />
				<input
					type="text"
					aria-label="Buscar modpacks en Modrinth"
					placeholder="Buscar modpacks en Modrinth..."
					value={query}
					oninput={handleQueryInput}
				/>
				{#if query}
					<button
						type="button"
						class="clear-btn"
						aria-label="Limpiar b?squeda"
						onclick={() => {
							query = '';
							debouncedQuery = '';
							offset = 0;
							runSearch();
						}}
					>
						<X size={14} />
					</button>
				{/if}
			</div>
			<SortSelect value={sortIndex} hasQuery={Boolean(query.trim())} onchange={handleSortChange} />
		</div>

		{#if installError}
			<div class="error-banner">
				<span>{installError}</span>
				<button type="button" onclick={() => (installError = null)}><X size={14} /></button>
			</div>
		{/if}

		<div class="results-area">
			{#if loading}
				<div class="loading-state">
					<Loader2 size={24} class="spin" />
					<p>Buscando modpacks...</p>
				</div>
			{:else if error}
				<div class="error-state">
					<p>{error}</p>
					<button data-pixel-control type="button" class="retry-btn" onclick={runSearch}
						>Reintentar</button
					>
				</div>
			{:else if hits.length === 0}
				<div class="empty-state">
					<p>No se encontraron modpacks coincidentes.</p>
				</div>
			{:else}
				<div class="hits-list">
					{#each hits as hit (hit.project_id)}
						<div class="hit-card">
							{#if hit.icon_url}
								<img src={hit.icon_url} alt="" class="hit-icon" />
							{:else}
								<div class="hit-icon-fallback">
									<PackageOpen size={22} />
								</div>
							{/if}
							<div class="hit-content">
								<div class="hit-title-row">
									<button type="button" class="hit-title-btn" onclick={() => openDetail(hit)}>
										{hit.title}
									</button>
									<span class="hit-author">de {hit.author}</span>
								</div>
								<p class="hit-desc">{hit.description}</p>
								<div class="hit-meta">
									<span class="meta-tag downloads">
										<Download size={12} />
										{hit.downloads.toLocaleString()}
									</span>
									<span class="meta-tag">Modrinth</span>
								</div>
							</div>
							<div class="hit-actions">
								{#if installSuccessId === hit.project_id}
									<button data-pixel-control type="button" class="action-btn success" disabled>
										<Check size={16} /> Instalado
									</button>
								{:else if installingId === hit.project_id}
									<button data-pixel-control type="button" class="action-btn installing" disabled>
										<Loader2 size={16} class="spin" /> Instalando...
									</button>
								{:else}
									<button
										data-pixel-control="primary"
										type="button"
										class="action-btn primary"
										onclick={() => handleInstall(hit)}
									>
										<Download size={14} /> Crear instancia
									</button>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<div class="panel-footer">
			<Pagination {offset} {limit} {totalHits} {loading} onPrev={handlePrev} onNext={handleNext} />
		</div>
	</div>
</div>

{#if detailProjectId}
	<ProjectDetailView
		kind="modpack"
		projectId={detailProjectId}
		hit={detailHit}
		detail={detailData}
		loading={detailLoading}
		error={detailError}
		installed={false}
		installing={installingId === detailProjectId}
		onBack={() => (detailProjectId = null)}
		onInstall={(versionId) => {
			if (detailHit) handleInstall(detailHit, versionId);
		}}
		loadVersions={async (pid) => {
			if (!detailData) return [];
			const mcVer = detailData.game_versions[0] || '1.20.1';
			const rawLoader = detailData.loaders[0]?.toLowerCase() || 'fabric';
			const loader = ['fabric', 'quilt', 'forge', 'neoforge'].includes(rawLoader)
				? (rawLoader as Loader)
				: 'fabric';
			return getModVersions(pid, mcVer, loader);
		}}
	/>
{/if}

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
		width: 780px;
		max-width: 95vw;
		height: 85vh;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		box-shadow: var(--shadow-lg);
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		flex-shrink: 0;
	}

	.title-row {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.title-row :global(.header-icon) {
		color: var(--accent);
	}

	.panel-header h2 {
		font-size: var(--text-lg);
		margin: 0;
	}

	.subtitle {
		font-size: 0.8rem;
		color: var(--text-secondary);
		margin: 2px 0 0;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		cursor: pointer;
	}

	.close-btn:hover {
		color: var(--text-primary);
		border-color: var(--text-muted);
	}

	.search-bar {
		display: flex;
		gap: 12px;
		align-items: center;
		flex-shrink: 0;
	}

	.input-wrap {
		position: relative;
		flex: 1;
		display: flex;
		align-items: center;
	}

	.input-wrap :global(.search-icon) {
		position: absolute;
		left: 12px;
		color: var(--text-muted);
		pointer-events: none;
	}

	.input-wrap input {
		width: 100%;
		padding: 9px 36px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.85rem;
		outline: none;
	}

	.input-wrap input:focus {
		border-color: var(--accent);
	}

	.clear-btn {
		position: absolute;
		right: 10px;
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 14px;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--danger, #ef4444) 15%, transparent);
		border: 1px solid var(--danger, #ef4444);
		color: var(--text-primary);
		font-size: 0.82rem;
	}

	.error-banner button {
		background: none;
		border: none;
		color: inherit;
		cursor: pointer;
	}

	.results-area {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
	}

	.hits-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.hit-card {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 12px 14px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-input);
		transition: border-color 0.15s;
	}

	.hit-card:hover {
		border-color: var(--accent);
	}

	.hit-icon {
		width: 44px;
		height: 44px;
		border-radius: var(--border-radius-sm);
		object-fit: cover;
		flex-shrink: 0;
	}

	.hit-icon-fallback {
		width: 44px;
		height: 44px;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--accent) 15%, transparent);
		color: var(--accent);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.hit-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.hit-title-row {
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.hit-title-btn {
		background: none;
		border: none;
		padding: 0;
		font-size: 0.95rem;
		font-weight: 700;
		color: var(--text-primary);
		cursor: pointer;
		text-align: left;
	}

	.hit-title-btn:hover {
		color: var(--accent);
		text-decoration: underline;
	}

	.hit-author {
		font-size: 0.75rem;
		color: var(--text-muted);
	}

	.hit-desc {
		font-size: 0.78rem;
		color: var(--text-secondary);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
		line-height: 1.4;
		margin: 0;
	}

	.hit-meta {
		display: flex;
		gap: 8px;
		align-items: center;
		margin-top: 2px;
	}

	.meta-tag {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 0.7rem;
		padding: 2px 6px;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--text-muted) 15%, transparent);
		color: var(--text-secondary);
	}

	.meta-tag.downloads {
		color: var(--text-muted);
	}

	.hit-actions {
		flex-shrink: 0;
	}

	.action-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		border-radius: var(--border-radius-sm);
		font-size: 0.82rem;
		font-weight: 600;
		cursor: pointer;
		border: 1px solid transparent;
		transition: filter 0.15s;
	}

	.action-btn.primary {
		background: var(--accent);
		color: var(--bg-app);
	}

	.action-btn.primary:hover {
		filter: brightness(1.1);
	}

	.action-btn.installing {
		background: color-mix(in srgb, var(--accent) 30%, transparent);
		color: var(--text-primary);
		cursor: default;
	}

	.action-btn.success {
		background: color-mix(in srgb, var(--success, #22c55e) 20%, transparent);
		color: var(--success, #22c55e);
		border-color: var(--success, #22c55e);
		cursor: default;
	}

	.loading-state,
	.empty-state,
	.error-state {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 10px;
		color: var(--text-muted);
		font-size: 0.88rem;
		padding: 40px;
	}

	.retry-btn {
		padding: 6px 14px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		cursor: pointer;
	}

	.panel-footer {
		display: flex;
		justify-content: center;
		padding-top: 8px;
		border-top: 1px solid var(--border);
		flex-shrink: 0;
	}

	:global(.spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
