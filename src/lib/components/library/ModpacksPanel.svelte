<script lang="ts">
	import { untrack } from 'svelte';
	import { onMount } from 'svelte';
	import type { InstanceData, InstalledModpack } from '$lib/types/types';
	import {
		updateCommunityModpack,
		getInstanceModpacks,
		checkModpackUpdates,
		removeModpack
	} from '$lib/api';
	import {
		Search,
		Trash2,
		Loader2,
		Package,
		RefreshCw,
		Maximize2,
		ChevronDown,
		Check
	} from 'lucide-svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { createContentExplorer, errorMessageKey } from '$lib/state/contentExplorer.svelte';
	import { openContentExplorer } from '$lib/state/uiState.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import SortSelect from '$lib/components/ui/SortSelect.svelte';
	import ContentResultRow from './ContentResultRow.svelte';
	import ProjectDetailView from './ProjectDetailView.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';

	let { instance }: { instance: InstanceData } = $props();

	// Categorías reales de Modrinth para modpacks (mismo taxonomy que mods,
	// confirmado contra la API real — ver ContentManager.svelte).
	const CATEGORIES = [
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
	];

	function categoryLabel(cat: string): string {
		const words = cat.split('-');
		return words.map((w) => w[0].toUpperCase() + w.slice(1)).join(' ');
	}

	let installed = $state<InstalledModpack[]>([]);
	let updates = $state<Record<string, string>>({}); // version_id -> new_mrpack_url
	let removingId = $state<string | null>(null);
	let updatingId = $state<string | null>(null);
	let error = $state<string | null>(null);
	let categoriesOpen = $state(false);
	let categoryDropdownEl = $state<HTMLDivElement | undefined>(undefined);
	let pendingInstallProjectId = $state<string | null>(null);
	let pendingInstallVersionId = $state<string | undefined>(undefined);

	async function refreshInstalled() {
		installed = await getInstanceModpacks(instance.name);
		try {
			const found = await checkModpackUpdates(instance.name);
			updates = Object.fromEntries(found.map((u) => [u.version_id, u.new_mrpack_url]));
		} catch {
			// silencioso — solo es una notificación, no bloquea la lista
			updates = {};
		}
	}

	onMount(refreshInstalled);

	// Búsqueda/paginación/orden/detalle compartidos con mods/shaders/etc. —
	// antes esto tenía su propia implementación aparte (incluso en Rust,
	// ver search_modpacks), que no buscaba nada con texto vacío. Unificado
	// sobre el mismo composable, "explorar sin buscar" funciona gratis acá
	// también.
	const explorer = createContentExplorer(() => ({ instance, kind: 'modpack' }));
	$effect(() => {
		void instance.name;
		untrack(() => explorer.resetForNewContext());
	});

	async function handleUpdate(pack: InstalledModpack) {
		const newUrl = updates[pack.version_id];
		if (!newUrl) return;
		updatingId = pack.version_id;
		error = null;
		try {
			// Instala la versión nueva primero y solo si eso funciona borra la
			// vieja (preservando lo que ambas compartan) — antes se borraba
			// primero, y si la descarga de la nueva fallaba a mitad de camino
			// (red cortada), la instancia quedaba sin el pack viejo ni el
			// nuevo, sin ningún reintento automático.
			await updateCommunityModpack(instance.name, pack.version_id, pack.project_id, newUrl);
			await refreshInstalled();
		} catch (e) {
			error = String(e);
		} finally {
			updatingId = null;
		}
	}

	// Instalar un modpack sobre una instancia que YA tiene uno puede
	// sobrescribir archivos de configuración compartidos — pedido
	// explícito del cliente: avisar antes, no una descarga aparentemente
	// inocua. Actualizar (handleUpdate, arriba) ya es cuidadoso por diseño
	// y no pasa por esta advertencia.
	function requestInstall(projectId: string, versionId?: string) {
		if (installed.length > 0) {
			pendingInstallProjectId = projectId;
			pendingInstallVersionId = versionId;
			return;
		}
		void doInstall(projectId, versionId);
	}

	async function doInstall(projectId: string, versionId?: string) {
		// `versionId` nunca llega poblado para modpacks (sin dropdown de
		// versiones, ver `hasVersions={false}` abajo) — el composable
		// instala siempre la última build compatible.
		await explorer.install(projectId, versionId, refreshInstalled);
	}

	function confirmPendingInstall() {
		const projectId = pendingInstallProjectId;
		const versionId = pendingInstallVersionId;
		pendingInstallProjectId = null;
		pendingInstallVersionId = undefined;
		if (projectId) void doInstall(projectId, versionId);
	}

	async function handleRemove(versionId: string) {
		removingId = versionId;
		try {
			await removeModpack(instance.name, versionId);
			await refreshInstalled();
		} catch (e) {
			error = String(e);
		} finally {
			removingId = null;
		}
	}
</script>

<div class="modpacks-panel">
	{#if instance.loader === 'vanilla'}
		<p class="hint">
			{t('modpacksPanel.vanillaNotSupported')}
		</p>
	{:else if explorer.detailProjectId}
		<ProjectDetailView
			kind="modpack"
			projectId={explorer.detailProjectId}
			hit={explorer.detailHit}
			detail={explorer.detail}
			loading={explorer.detailLoading}
			error={explorer.detailError}
			onBack={() => explorer.closeDetail()}
			onInstall={(versionId) => requestInstall(explorer.detailProjectId!, versionId)}
			installing={explorer.installingId === explorer.detailProjectId}
			installed={installed.some((p) => p.project_id === explorer.detailProjectId)}
			loadVersions={() => Promise.resolve([])}
		/>
	{:else}
		<p class="hint">
			{t('modpacksPanel.hint')}
		</p>

		{#if error}
			<p class="error">{error}</p>
		{/if}

		{#if installed.length > 0}
			<div class="installed">
				<span class="section-label"
					>{t('modpacksPanel.installedCount', { count: installed.length })}</span
				>
				{#each installed as pack (pack.version_id)}
					<div class="installed-row">
						<Package size={13} class="pack-icon" />
						<span class="filename">{pack.title} · {pack.file_count} archivos</span>
						{#if updates[pack.version_id]}
							<button
								type="button"
								class="update-btn"
								disabled={updatingId === pack.version_id}
								onclick={() => handleUpdate(pack)}
							>
								{#if updatingId === pack.version_id}
									<Loader2 size={12} class="spin" />
								{:else}
									<RefreshCw size={12} />
								{/if}
								{t('modpacksPanel.update')}
							</button>
						{/if}
						<button
							type="button"
							class="icon-btn"
							disabled={removingId === pack.version_id}
							onclick={() => handleRemove(pack.version_id)}
							aria-label={t('contentManager.remove')}
						>
							{#if removingId === pack.version_id}
								<Loader2 size={13} class="spin" />
							{:else}
								<Trash2 size={13} />
							{/if}
						</button>
					</div>
				{/each}
			</div>
		{/if}

		<div class="download-toolbar">
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
						{#each CATEGORIES as cat (cat)}
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
			<button
				type="button"
				class="explore-wide-btn"
				onclick={() => openContentExplorer({ instance, kind: 'modpack' })}
			>
				<Maximize2 size={13} />
				{t('contentManager.exploreWide')}
			</button>
		</div>

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
				placeholder={t('contentManager.searchPlaceholder', { noun: 'modpacks' })}
			/>
		</div>

		{#if explorer.errorKind}
			<p class="error">
				{t(errorMessageKey(explorer.errorKind))}
				<button type="button" class="retry-btn" onclick={() => explorer.setQuery(explorer.query)}>
					{t('contentManager.retry')}
				</button>
			</p>
		{/if}

		{#if !explorer.loading && !explorer.errorKind && explorer.results.length === 0}
			<p class="hint">{t('contentManager.noResults')}</p>
		{:else}
			<div class="results">
				{#if explorer.results.length > 0}<span class="section-label"
						>{t('contentManager.results')}</span
					>{/if}
				{#each explorer.results as item (item.project_id)}
					<ContentResultRow
						{item}
						installed={installed.some((p) => p.project_id === item.project_id)}
						installing={explorer.installingId === item.project_id}
						hasVersions={false}
						onInstall={() => requestInstall(item.project_id)}
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
	{/if}
</div>

{#if pendingInstallProjectId}
	<ConfirmDialog
		title={t('modpacksPanel.overwriteWarningTitle')}
		message={t('modpacksPanel.overwriteWarningMessage')}
		confirmLabel={t('modpacksPanel.overwriteWarningConfirm')}
		danger
		onConfirm={confirmPendingInstall}
		onCancel={() => {
			pendingInstallProjectId = null;
			pendingInstallVersionId = undefined;
		}}
	/>
{/if}

<style>
	.modpacks-panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
	}

	.download-toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}

	.download-toolbar :global(.tfl-field) {
		width: auto;
		min-width: 160px;
	}

	.explore-wide-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-left: auto;
		padding: 5px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.72rem;
		font-weight: 600;
		cursor: pointer;
	}

	.explore-wide-btn:hover {
		border-color: var(--accent);
		color: var(--accent);
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

	.search-row {
		position: relative;
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

	.section-label {
		display: block;
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
		margin-bottom: 6px;
	}

	.installed-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		margin-bottom: 4px;
	}

	.installed-row :global(.pack-icon) {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.filename {
		flex: 1;
		min-width: 0;
		font-size: 0.76rem;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.update-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		flex-shrink: 0;
		padding: 4px 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		color: var(--accent);
		font-size: 0.68rem;
		font-weight: 700;
		cursor: pointer;
	}

	.update-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		flex-shrink: 0;
	}

	.icon-btn:hover {
		color: var(--color-error);
	}

	.results {
		display: flex;
		flex-direction: column;
		gap: 6px;
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
