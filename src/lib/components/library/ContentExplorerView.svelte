<script lang="ts">
	import { untrack } from 'svelte';
	import { modalFocus } from '$lib/visual/modalFocus';
	import type { InstanceData } from '$lib/types/types';
	import type { ContentExplorerKind } from '$lib/state/uiState.svelte';
	import { createContentExplorer, errorMessageKey } from '$lib/state/contentExplorer.svelte';
	import { favoriteMods, isFavoriteMod, toggleFavoriteMod } from '$lib/state/modFavorites.svelte';
	import { viewport, SIDEBAR_FORCE_COMPACT_WIDTH } from '$lib/state/viewport.svelte';
	import {
		getInstalledModsInfo,
		getInstalledShadersInfo,
		getInstalledResourcepacksInfo,
		getInstalledPluginsInfo
	} from '$lib/api/content';
	import { getInstanceModpacks } from '$lib/api/modpacks';
	import { t } from '$lib/i18n/index.svelte';
	import {
		X,
		Search,
		ChevronDown,
		Check,
		Filter,
		Loader2,
		ArrowLeft,
		Download
	} from 'lucide-svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import SortSelect from '$lib/components/ui/SortSelect.svelte';
	import ContentResultRow from './ContentResultRow.svelte';
	import ProjectDetailView from './ProjectDetailView.svelte';

	/** Vista amplia de exploración de contenido — pedido explícito del
	 * cliente: una pantalla dedicada DENTRO del launcher (sin ventana Tauri
	 * nueva) que conserva el contexto de la instancia de origen. Mismo
	 * composable que la vista compacta (ContentManager.svelte/
	 * ModpacksPanel.svelte) — nada de lógica de búsqueda duplicada. */
	let {
		instance,
		kind: initialKind,
		onClose
	}: {
		instance: InstanceData;
		kind: ContentExplorerKind;
		onClose: () => void;
	} = $props();

	const KIND_NOUN: Record<ContentExplorerKind, string> = {
		mod: 'mods',
		shader: 'shaders',
		resourcepack: 'resource packs',
		plugin: 'plugins',
		modpack: 'modpacks'
	};

	// Mismas reglas de compatibilidad que ya usan los wrappers chicos
	// (ModsPanel/PluginsPanel/ModpacksPanel.svelte) — mods/modpacks piden
	// loader de cliente, plugins piden software de servidor no-vanilla.
	const availableKinds = $derived.by((): ContentExplorerKind[] => {
		if (instance.server_type) {
			return instance.server_type === 'vanilla' ? [] : ['plugin'];
		}
		const kinds: ContentExplorerKind[] = ['shader', 'resourcepack'];
		if (instance.loader !== 'vanilla') {
			kinds.unshift('mod');
			kinds.push('modpack');
		}
		return kinds;
	});

	let activeKind = $state(initialKind);

	const CATEGORIES: Record<ContentExplorerKind, string[]> = {
		mod: [
			'adventure',
			'cursed',
			'decoration',
			'economy',
			'equipment',
			'food',
			'game-mechanics',
			'library',
			'magic',
			'management',
			'minigame',
			'mobs',
			'optimization',
			'social',
			'storage',
			'technology',
			'transportation',
			'utility',
			'worldgen'
		],
		resourcepack: [
			'audio',
			'blocks',
			'combat',
			'decoration',
			'cursed',
			'equipment',
			'entities',
			'environment',
			'fonts',
			'gui',
			'items',
			'locale',
			'models',
			'modded',
			'realistic',
			'simplistic',
			'themed',
			'tweaks',
			'utility',
			'vanilla-like'
		],
		shader: [
			'atmosphere',
			'bloom',
			'cartoon',
			'colored-lighting',
			'cursed',
			'fantasy',
			'foliage',
			'high',
			'low',
			'medium',
			'path-tracing',
			'pbr',
			'potato',
			'realistic',
			'reflections',
			'screenshot',
			'semi-realistic',
			'shadows',
			'vanilla-like'
		],
		plugin: [
			'adventure',
			'cursed',
			'decoration',
			'economy',
			'equipment',
			'food',
			'game-mechanics',
			'library',
			'magic',
			'management',
			'minigame',
			'mobs',
			'optimization',
			'social',
			'storage',
			'technology',
			'transportation',
			'utility',
			'worldgen'
		],
		modpack: [
			'adventure',
			'combat',
			'economy',
			'game-mechanics',
			'horror',
			'kitchen-sink',
			'lightweight',
			'magic',
			'multiplayer',
			'optimization',
			'quests',
			'technology'
		]
	};

	function categoryLabel(cat: string): string {
		const words = cat.split('-');
		return words.map((w) => w[0].toUpperCase() + w.slice(1)).join(' ');
	}

	// Un composable POR kind, creado recién cuando se visita esa pestaña —
	// así cada uno conserva su propia búsqueda/filtros/página mientras la
	// vista amplia sigue abierta, sin compartir estado EN VIVO entre
	// pestañas (si una instancia ya tiene explorer creado, cambiar de
	// pestaña y volver no reinicia su búsqueda).
	const explorers = new Map<ContentExplorerKind, ReturnType<typeof createContentExplorer>>();
	function explorerFor(k: ContentExplorerKind) {
		let e = explorers.get(k);
		if (!e) {
			e = createContentExplorer(() => ({ instance, kind: k }));
			explorers.set(k, e);
			untrack(() => e!.init());
		}
		return e;
	}
	const explorer = $derived(explorerFor(activeKind));

	$effect(() => {
		void instance.name;
		untrack(() => {
			for (const e of explorers.values()) e.resetForNewContext();
		});
	});

	// "Ya instalado" para la pestaña activa — mismo chequeo que
	// ContentManager.svelte/ModpacksPanel.svelte, acá aparte porque esta
	// vista no tiene una sección "Gestionar" que ya lo traiga.
	let installedProjectIds = $state<Set<string>>(new Set());
	async function refreshInstalledFor(k: ContentExplorerKind) {
		try {
			if (k === 'modpack') {
				const packs = await getInstanceModpacks(instance.name);
				installedProjectIds = new Set(packs.map((p) => p.project_id));
				return;
			}
			const infoFn =
				k === 'mod'
					? getInstalledModsInfo
					: k === 'shader'
						? getInstalledShadersInfo
						: k === 'plugin'
							? getInstalledPluginsInfo
							: getInstalledResourcepacksInfo;
			const info = await infoFn(instance.name);
			installedProjectIds = new Set(info.map((i) => i.project_id).filter((p): p is string => !!p));
		} catch {
			installedProjectIds = new Set();
		}
	}
	$effect(() => {
		void instance.name;
		void activeKind;
		refreshInstalledFor(activeKind);
	});

	async function handleInstall(projectId: string, versionId?: string) {
		if (!versionId && installedProjectIds.has(projectId)) return;
		await explorer.install(projectId, versionId, () => refreshInstalledFor(activeKind));
	}

	let categoriesOpen = $state(false);
	let categoryDropdownEl = $state<HTMLDivElement | undefined>(undefined);
	let filtersCollapsed = $state(viewport.width < SIDEBAR_FORCE_COMPACT_WIDTH);

	function handleWindowClick(e: MouseEvent) {
		if (categoriesOpen && categoryDropdownEl && !categoryDropdownEl.contains(e.target as Node)) {
			categoriesOpen = false;
		}
	}
</script>

<svelte:window onclick={handleWindowClick} />

<div class="overlay">
	<div use:modalFocus={onClose} class="panel" role="dialog" aria-modal="true" tabindex="-1">
		<header class="panel-header">
			<div class="header-info">
				<h2>
					{t('contentExplorer.title', { noun: KIND_NOUN[activeKind], instance: instance.name })}
				</h2>
				<span class="subtitle"
					>{t('contentExplorer.subtitle', {
						mcVersion: instance.mc_version,
						loader: instance.server_type ?? instance.loader
					})}</span
				>
			</div>
			<button type="button" class="back-btn" onclick={onClose}>
				<ArrowLeft size={14} />
				{t('contentExplorer.back')}
			</button>
			<button type="button" class="close-btn" onclick={onClose} aria-label={t('ui.close')}>
				<X size={16} />
			</button>
		</header>

		{#if availableKinds.length > 1}
			<div class="kind-tabs">
				{#each availableKinds as k (k)}
					<button
						type="button"
						class="kind-tab"
						class:active={activeKind === k}
						onclick={() => (activeKind = k)}
					>
						{KIND_NOUN[k]}
					</button>
				{/each}
			</div>
		{/if}

		{#if explorer.detailProjectId}
			<ProjectDetailView
				kind={activeKind}
				projectId={explorer.detailProjectId}
				hit={explorer.detailHit}
				detail={explorer.detail}
				loading={explorer.detailLoading}
				error={explorer.detailError}
				onBack={() => explorer.closeDetail()}
				onInstall={(versionId) => handleInstall(explorer.detailProjectId!, versionId)}
				installing={!!explorer.detailProjectId &&
					explorer.pendingIds.includes(explorer.detailProjectId)}
				installed={installedProjectIds.has(explorer.detailProjectId)}
				loadVersions={(id) => explorer.getVersionsFor(id)}
			/>
		{:else}
			<div class="toolbar">
				<div class="search-row">
					{#if explorer.loading}
						<Loader2 size={14} class="spin search-icon" />
					{:else}
						<Search size={14} class="search-icon" />
					{/if}
					<input
						type="text"
						value={explorer.query}
						oninput={(e) => explorer.setQuery((e.target as HTMLInputElement).value)}
						placeholder={t('contentManager.searchPlaceholder', { noun: KIND_NOUN[activeKind] })}
					/>
				</div>
				<button
					type="button"
					class="filters-toggle"
					class:active={!filtersCollapsed}
					onclick={() => (filtersCollapsed = !filtersCollapsed)}
				>
					<Filter size={13} />
					{t('contentExplorer.filters')}
				</button>
			</div>

			{#if !filtersCollapsed}
				<div class="filters-row">
					<div class="category-dropdown" bind:this={categoryDropdownEl}>
						<button
							type="button"
							class="category-trigger"
							class:active={explorer.selectedCategories.size > 0}
							onclick={() => (categoriesOpen = !categoriesOpen)}
						>
							{t('contentManager.categories')}{explorer.selectedCategories.size > 0
								? ` (${explorer.selectedCategories.size})`
								: ''}
							<ChevronDown size={13} />
						</button>
						{#if categoriesOpen}
							<div class="category-panel">
								{#each CATEGORIES[activeKind] as cat (cat)}
									<button
										type="button"
										class="category-option"
										class:active={explorer.selectedCategories.has(cat)}
										onclick={() => explorer.toggleCategory(cat)}
									>
										{#if explorer.selectedCategories.has(cat)}<Check size={12} />{:else}<span
												class="option-spacer"
											></span>{/if}
										{categoryLabel(cat)}
									</button>
								{/each}
								{#if explorer.selectedCategories.size > 0}
									<button
										type="button"
										class="category-option clear"
										onclick={() => explorer.clearCategories()}
									>
										{t('contentManager.clearCategories')}
									</button>
								{/if}
							</div>
						{/if}
					</div>
					<SortSelect
						value={explorer.sortIndex}
						hasQuery={explorer.hasQuery}
						disabled={explorer.loading}
						onchange={(v) => explorer.setSort(v)}
					/>
				</div>
			{/if}

			<div class="results-area">
				{#if explorer.errorKind}
					<p class="error">
						{t(errorMessageKey(explorer.errorKind))}
						<button
							type="button"
							class="retry-btn"
							onclick={() => explorer.setQuery(explorer.query)}
						>
							{t('contentManager.retry')}
						</button>
					</p>
				{/if}

				{#if activeKind === 'mod' && !explorer.hasQuery && favoriteMods.length > 0}
					<div class="results">
						<span class="section-label">{t('contentManager.favorites')}</span>
						{#each favoriteMods as mod (mod.projectId)}
							<div class="favorite-card">
								{#if mod.iconUrl}<img src={mod.iconUrl} alt={mod.title} />{:else}<div
										class="favorite-icon-fallback"
									></div>{/if}
								<span class="favorite-title">{mod.title}</span>
								<button
									type="button"
									class="install-btn"
									disabled={explorer.installingId === mod.projectId}
									onclick={() => handleInstall(mod.projectId)}
								>
									{#if explorer.installingId === mod.projectId}<Loader2
											size={14}
											class="spin"
										/>{:else}<Download size={14} />{/if}
								</button>
							</div>
						{/each}
					</div>
				{/if}

				{#if !explorer.loading && !explorer.errorKind && explorer.results.length === 0}
					<p class="hint">{t('contentManager.noResults')}</p>
				{:else}
					<div class="results">
						{#each explorer.results as item (item.project_id)}
							<ContentResultRow
								{item}
								installed={installedProjectIds.has(item.project_id)}
								installing={explorer.pendingIds.includes(item.project_id)}
								showFavorite={activeKind === 'mod'}
								isFavorite={isFavoriteMod(item.project_id)}
								hasVersions={activeKind !== 'modpack'}
								quickInstall={activeKind !== 'modpack'}
								versionsOpen={explorer.openVersionsFor === item.project_id}
								versionOptions={explorer.versionOptions}
								loadingVersions={explorer.loadingVersions}
								onInstall={(versionId) => handleInstall(item.project_id, versionId)}
								onToggleFavorite={() =>
									toggleFavoriteMod({
										projectId: item.project_id,
										title: item.title,
										iconUrl: item.icon_url
									})}
								onToggleVersions={() => explorer.toggleVersions(item.project_id)}
								onOpenDetail={() => explorer.openDetail(item.project_id, item)}
							/>
						{/each}
					</div>
					{#if explorer.totalHits > explorer.limit}
						<Pagination
							offset={explorer.offset}
							limit={explorer.limit}
							totalHits={explorer.totalHits}
							loading={explorer.loading}
							onPrev={() => explorer.goToPrevPage()}
							onNext={() => explorer.goToNextPage()}
						/>
					{/if}
				{/if}
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
		width: calc(100vw - 32px);
		max-width: 1100px;
		height: calc(100vh - 32px);
		max-height: 820px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 18px 22px;
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 12px;
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.header-info {
		flex: 1;
		min-width: 0;
	}

	.header-info h2 {
		margin: 0;
		font-size: 1rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.subtitle {
		font-size: 0.72rem;
		color: var(--text-muted);
	}

	.back-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.72rem;
		font-weight: 600;
		cursor: pointer;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.close-btn:hover {
		color: var(--text-primary);
	}

	.kind-tabs {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
		border-bottom: 1px solid var(--border);
		padding-bottom: 8px;
	}

	.kind-tab {
		padding: 5px 12px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		font-size: 0.74rem;
		font-weight: 700;
		text-transform: capitalize;
		cursor: pointer;
	}

	.kind-tab.active {
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		color: var(--accent);
	}

	.toolbar {
		display: flex;
		gap: 8px;
	}

	.search-row {
		position: relative;
		flex: 1;
		display: flex;
		align-items: center;
	}

	.search-row :global(.search-icon) {
		position: absolute;
		left: 12px;
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-row input {
		width: 100%;
		padding: 9px 12px 9px 34px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.85rem;
	}

	.filters-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
		padding: 0 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.72rem;
		font-weight: 600;
		cursor: pointer;
	}

	.filters-toggle.active {
		border-color: var(--accent);
		color: var(--accent);
	}

	.filters-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}

	.filters-row :global(.tfl-field) {
		width: auto;
		min-width: 160px;
	}

	.category-dropdown {
		position: relative;
	}

	.category-trigger {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-muted);
		font-size: 0.72rem;
		font-weight: 600;
		cursor: pointer;
	}

	.category-trigger.active {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, var(--bg-input));
		color: var(--accent);
	}

	.category-panel {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		z-index: 20;
		display: grid;
		grid-template-columns: repeat(2, minmax(140px, 1fr));
		gap: 2px;
		max-height: 220px;
		overflow-y: auto;
		padding: 6px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-card);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
	}

	.category-option {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 8px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 0.72rem;
		text-align: left;
		cursor: pointer;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.category-option:hover {
		background: var(--bg-input);
	}

	.category-option.active {
		color: var(--accent);
		font-weight: 600;
	}

	.category-option :global(svg) {
		flex-shrink: 0;
		color: var(--accent);
	}

	.category-option.clear {
		justify-content: center;
		grid-column: 1 / -1;
		margin-top: 2px;
		border-top: 1px solid var(--border);
		border-radius: 0;
		color: var(--text-muted);
		font-weight: 600;
	}

	.option-spacer {
		width: 12px;
		flex-shrink: 0;
	}

	.results-area {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.results {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.section-label {
		display: block;
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
		margin-bottom: 6px;
	}

	.favorite-card {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-card);
	}

	.favorite-card img,
	.favorite-icon-fallback {
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		flex-shrink: 0;
		background: var(--bg-input);
	}

	.favorite-title {
		flex: 1;
		font-size: 0.8rem;
		font-weight: 600;
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

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
	}

	.error {
		color: var(--color-error);
		font-size: 0.78rem;
	}

	.retry-btn {
		margin-left: 8px;
		padding: 2px 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--color-error);
		background: transparent;
		color: var(--color-error);
		font-size: 0.68rem;
		font-weight: 600;
		cursor: pointer;
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

	@media (max-width: 860px) {
		.panel {
			padding: 14px;
		}
	}
</style>
