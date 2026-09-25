<script lang="ts">
	import { onMount } from 'svelte';
	import type {
		InstanceData,
		ModSearchHit,
		InstalledModInfo,
		ModUpdateAvailable,
		ModVersionSummary
	} from '$lib/types/types';
	import { appState } from '$lib/state/state.svelte';
	import {
		searchMods,
		installMod,
		getInstalledModsInfo,
		checkModUpdates,
		updateAllMods,
		getModVersionChangelog,
		findDuplicateMods,
		removeMod,
		getModVersions,
		searchShaders,
		installShader,
		getInstalledShadersInfo,
		removeShader,
		getShaderVersions,
		searchResourcepacks,
		installResourcepack,
		getInstalledResourcepacksInfo,
		removeResourcepack,
		getResourcepackVersions,
		pickContentFiles,
		addLocalModFiles,
		addLocalShaderFiles,
		addLocalResourcepackFiles
	} from '$lib/api/tflApi';
	import { favoriteMods, isFavoriteMod, toggleFavoriteMod } from '$lib/state/modFavorites.svelte';
	import {
		Search,
		Download,
		Trash2,
		Loader2,
		RefreshCw,
		AlertTriangle,
		Star,
		FileText,
		Upload,
		Check,
		ChevronDown
	} from 'lucide-svelte';

	type Kind = 'mod' | 'shader' | 'resourcepack';

	let {
		instance,
		kind,
		refreshSignal = 0
	}: { instance: InstanceData; kind: Kind; refreshSignal?: number } = $props();

	// Categorías reales de Modrinth por tipo de proyecto (GET /v2/tag/category)
	// — se usan como filtro en la pestaña Descargar.
	const CATEGORIES: Record<Kind, string[]> = {
		mod: [
			'adventure', 'cursed', 'decoration', 'economy', 'equipment', 'food',
			'game-mechanics', 'library', 'magic', 'management', 'minigame', 'mobs',
			'optimization', 'social', 'storage', 'technology', 'transportation',
			'utility', 'worldgen'
		],
		resourcepack: [
			'audio', 'blocks', 'combat', 'decoration', 'cursed', 'equipment',
			'entities', 'environment', 'fonts', 'gui', 'items', 'locale', 'models',
			'modded', 'realistic', 'simplistic', 'themed', 'tweaks', 'utility',
			'vanilla-like'
		],
		shader: [
			'atmosphere', 'bloom', 'cartoon', 'colored-lighting', 'cursed', 'fantasy',
			'foliage', 'high', 'low', 'medium', 'path-tracing', 'pbr', 'potato',
			'realistic', 'reflections', 'screenshot', 'semi-realistic', 'shadows',
			'vanilla-like'
		]
	};

	const EXTENSION: Record<Kind, string> = { mod: 'jar', shader: 'zip', resourcepack: 'zip' };
	const FILTER_LABEL: Record<Kind, string> = {
		mod: 'Mod (.jar)',
		shader: 'Shader pack (.zip)',
		resourcepack: 'Resource pack (.zip)'
	};
	const NOUN: Record<Kind, string> = { mod: 'mods', shader: 'shaders', resourcepack: 'resource packs' };

	function apiSearch(q: string, categories: string[]) {
		if (kind === 'mod') return searchMods(q, instance.mc_version, instance.loader, categories);
		if (kind === 'shader') return searchShaders(q, instance.mc_version, categories);
		return searchResourcepacks(q, instance.mc_version, categories);
	}
	function apiInstall(projectId: string, versionId?: string) {
		if (kind === 'mod')
			return installMod(instance.name, projectId, instance.mc_version, instance.loader, versionId);
		if (kind === 'shader') return installShader(instance.name, projectId, instance.mc_version, versionId);
		return installResourcepack(instance.name, projectId, instance.mc_version, versionId);
	}
	function apiInstalledInfo() {
		if (kind === 'mod') return getInstalledModsInfo(instance.name);
		if (kind === 'shader') return getInstalledShadersInfo(instance.name);
		return getInstalledResourcepacksInfo(instance.name);
	}
	function apiRemove(filename: string) {
		if (kind === 'mod') return removeMod(instance.name, filename);
		if (kind === 'shader') return removeShader(instance.name, filename);
		return removeResourcepack(instance.name, filename);
	}
	function apiVersions(projectId: string) {
		if (kind === 'mod') return getModVersions(projectId, instance.mc_version, instance.loader);
		if (kind === 'shader') return getShaderVersions(projectId, instance.mc_version);
		return getResourcepackVersions(projectId, instance.mc_version);
	}
	function apiAddLocal(paths: string[]) {
		if (kind === 'mod') return addLocalModFiles(instance.name, paths);
		if (kind === 'shader') return addLocalShaderFiles(instance.name, paths);
		return addLocalResourcepackFiles(instance.name, paths);
	}

	let subtab = $state<'manage' | 'download'>('manage');

	// ── Gestionar ──────────────────────────────────────────────────────
	let installed = $state<InstalledModInfo[]>([]);
	let manageSelectedCategories = $state<Set<string>>(new Set());
	let manageCategoriesOpen = $state(false);
	let manageCategoryDropdownEl = $state<HTMLDivElement | undefined>(undefined);

	// Solo las categorías que de verdad aparecen entre lo instalado — a
	// diferencia de Descargar, acá no tiene sentido ofrecer las ~19
	// categorías completas de Modrinth cuando el mod instalado puede ser
	// de solo 2 o 3 distintas.
	const manageAvailableCategories = $derived(
		[...new Set(installed.flatMap((i) => i.categories))].sort()
	);

	const installedFiltered = $derived(
		manageSelectedCategories.size === 0
			? installed
			: installed.filter((i) => i.categories.some((c) => manageSelectedCategories.has(c)))
	);

	function toggleManageCategory(cat: string) {
		const next = new Set(manageSelectedCategories);
		if (next.has(cat)) next.delete(cat);
		else next.add(cat);
		manageSelectedCategories = next;
	}

	let selected = $state<Set<string>>(new Set());
	let updates = $state<ModUpdateAvailable[]>([]);
	let duplicateGroups = $state<string[][]>([]);
	let updateMsg = $state<string | null>(null);
	let checkingUpdates = $state(false);
	let updatingAll = $state(false);
	let addingLocal = $state(false);
	let removingSelected = $state(false);
	let manageError = $state<string | null>(null);
	let changelogFor = $state<string | null>(null);
	let changelogText = $state<string | null>(null);
	let loadingChangelog = $state(false);

	let refreshToken = 0;
	async function refreshInstalled() {
		const token = ++refreshToken;
		const freshInstalled = await apiInstalledInfo();
		if (token !== refreshToken) return; // una llamada más nueva ya ganó
		installed = freshInstalled;
		const stillThere = new Set(installed.map((i) => i.filename));
		selected = new Set([...selected].filter((f) => stillThere.has(f)));
		if (kind === 'mod') {
			try {
				const freshUpdates = await checkModUpdates(instance.name);
				if (token === refreshToken) updates = freshUpdates;
			} catch {
				if (token === refreshToken) updates = [];
			}
			try {
				const freshDupes = await findDuplicateMods(instance.name);
				if (token === refreshToken) duplicateGroups = freshDupes;
			} catch {
				if (token === refreshToken) duplicateGroups = [];
			}
		}
	}

	$effect(() => {
		instance.name;
		kind;
		refreshSignal;
		manageSelectedCategories = new Set();
		refreshInstalled();
	});

	function toggleSelect(filename: string) {
		const next = new Set(selected);
		if (next.has(filename)) next.delete(filename);
		else next.add(filename);
		selected = next;
	}

	function toggleSelectAll() {
		selected =
			selected.size === installedFiltered.length
				? new Set()
				: new Set(installedFiltered.map((i) => i.filename));
	}

	async function handleRemoveSelected() {
		if (selected.size === 0) return;
		removingSelected = true;
		manageError = null;
		// Antes esto era un `for...of` que cortaba entero en el primer error
		// — si el tercero de cinco fallaba, ni se refrescaba la lista ni se
		// soltaba la selección, dejando la UI mostrando como "instalados y
		// seleccionados" un par de archivos que ya se habían borrado de
		// verdad. Ahora se intenta con todos, se refresca siempre, y solo
		// quedan seleccionados los que de verdad fallaron (para reintentar).
		const targets = [...selected];
		const results = await Promise.allSettled(targets.map((filename) => apiRemove(filename)));
		const failed = targets.filter((_, i) => results[i].status === 'rejected');
		selected = new Set(failed);
		if (failed.length > 0) {
			manageError = `No se pudieron quitar ${failed.length} de ${targets.length}: ${failed.join(', ')}`;
		}
		await refreshInstalled();
		removingSelected = false;
	}

	async function handleCheckUpdatesSelected() {
		checkingUpdates = true;
		updateMsg = null;
		manageError = null;
		try {
			const fresh = await checkModUpdates(instance.name);
			updates = fresh;
			const relevant = selected.size > 0 ? fresh.filter((u) => selected.has(u.filename)) : fresh;
			updateMsg = relevant.length === 0 ? 'Todo al día — no hay actualizaciones.' : null;
		} catch (e) {
			manageError = String(e);
		} finally {
			checkingUpdates = false;
		}
	}

	async function handleUpdateAll() {
		updatingAll = true;
		manageError = null;
		try {
			await updateAllMods(instance.name);
			await refreshInstalled();
		} catch (e) {
			manageError = String(e);
		} finally {
			updatingAll = false;
		}
	}

	async function handleRemoveOne(filename: string) {
		manageError = null;
		try {
			await apiRemove(filename);
			await refreshInstalled();
		} catch (e) {
			manageError = String(e);
		}
	}

	let changelogToken = 0;
	async function toggleChangelog(update: ModUpdateAvailable) {
		if (changelogFor === update.new_version_id) {
			changelogFor = null;
			changelogText = null;
			return;
		}
		changelogFor = update.new_version_id;
		changelogText = null;
		loadingChangelog = true;
		const token = ++changelogToken;
		try {
			const text = (await getModVersionChangelog(update.new_version_id)) || 'Sin notas de cambios.';
			if (token === changelogToken) changelogText = text;
		} catch (e) {
			if (token === changelogToken) changelogText = `No se pudo cargar: ${e}`;
		} finally {
			if (token === changelogToken) loadingChangelog = false;
		}
	}

	// El diálogo nativo de archivos crashea el launcher en macOS con la
	// firma ad-hoc del build (ver CHANGELOG_macos-dialog-crash-java26.txt) —
	// por eso esta función queda detrás del modo global Automático/Manual
	// (Ajustes), no de la plataforma.
	let dialogsBlocked = $derived(appState.settings?.native_dialog_mode !== 'manual');

	async function handleAddLocal() {
		if (dialogsBlocked) return;
		const paths = await pickContentFiles(EXTENSION[kind], FILTER_LABEL[kind]);
		if (paths.length === 0) return;
		addingLocal = true;
		manageError = null;
		try {
			const added = await apiAddLocal(paths);
			await refreshInstalled();
			// El backend sigue de largo con lo que sí puede copiar — si algo
			// no matcheaba la extensión esperada o falló al copiar, antes no
			// había ningún aviso de que faltó algo de lo elegido.
			if (added < paths.length) {
				manageError = `Se agregaron ${added} de ${paths.length} archivos — el resto no tenía la extensión esperada o no se pudo copiar.`;
			}
		} catch (e) {
			manageError = String(e);
		} finally {
			addingLocal = false;
		}
	}

	// ── Descargar ──────────────────────────────────────────────────────
	let query = $state('');
	let selectedCategories = $state<Set<string>>(new Set());
	let results = $state<ModSearchHit[]>([]);
	let searching = $state(false);
	let installingId = $state<string | null>(null);
	let downloadError = $state<string | null>(null);
	let openVersionsFor = $state<string | null>(null);
	let versionOptions = $state<ModVersionSummary[]>([]);
	let loadingVersions = $state(false);
	let categoriesOpen = $state(false);
	let categoryDropdownEl = $state<HTMLDivElement | undefined>(undefined);

	function handleWindowClick(e: MouseEvent) {
		if (categoriesOpen && categoryDropdownEl && !categoryDropdownEl.contains(e.target as Node)) {
			categoriesOpen = false;
		}
		if (
			manageCategoriesOpen &&
			manageCategoryDropdownEl &&
			!manageCategoryDropdownEl.contains(e.target as Node)
		) {
			manageCategoriesOpen = false;
		}
	}

	// Modrinth usa slugs kebab-case como valor real del filtro (ej.
	// "game-mechanics") — esto solo arma la etiqueta legible para mostrar.
	function categoryLabel(cat: string): string {
		const words = cat.split('-');
		return words.map((w) => w[0].toUpperCase() + w.slice(1)).join(' ');
	}

	let searchToken = 0;
	async function runSearch() {
		const token = ++searchToken;
		searching = true;
		downloadError = null;
		try {
			const hits = await apiSearch(query, [...selectedCategories]);
			if (token === searchToken) results = hits;
		} catch (e) {
			if (token === searchToken) downloadError = String(e);
		} finally {
			if (token === searchToken) searching = false;
		}
	}

	// A diferencia de una búsqueda con texto, acá se dispara igual con
	// query vacío — es lo que trae el listado por defecto (populares) al
	// entrar a la pestaña, sin que el usuario tenga que escribir nada.
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		if (subtab !== 'download') return;
		query;
		selectedCategories;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(runSearch, 300);
		return () => clearTimeout(debounceTimer);
	});

	function toggleCategory(cat: string) {
		const next = new Set(selectedCategories);
		if (next.has(cat)) next.delete(cat);
		else next.add(cat);
		selectedCategories = next;
	}

	const installedProjectIds = $derived(
		new Set(installed.map((i) => i.project_id).filter((p): p is string => !!p))
	);

	async function handleInstall(projectId: string, versionId?: string) {
		installingId = projectId;
		downloadError = null;
		try {
			await apiInstall(projectId, versionId);
			await refreshInstalled();
			openVersionsFor = null;
		} catch (e) {
			downloadError = String(e);
		} finally {
			installingId = null;
		}
	}

	let versionsToken = 0;
	async function toggleVersions(projectId: string) {
		if (openVersionsFor === projectId) {
			openVersionsFor = null;
			return;
		}
		openVersionsFor = projectId;
		versionOptions = [];
		loadingVersions = true;
		const token = ++versionsToken;
		try {
			const options = await apiVersions(projectId);
			if (token === versionsToken) versionOptions = options;
		} catch {
			if (token === versionsToken) versionOptions = [];
		} finally {
			if (token === versionsToken) loadingVersions = false;
		}
	}
</script>

<svelte:window onclick={handleWindowClick} />

<div class="content-manager">
	<div class="subtabs">
		<button type="button" class="subtab" class:active={subtab === 'manage'} onclick={() => (subtab = 'manage')}>
			Gestionar {#if installed.length > 0}<span class="count">{installed.length}</span>{/if}
		</button>
		<button type="button" class="subtab" class:active={subtab === 'download'} onclick={() => (subtab = 'download')}>
			Descargar
		</button>
	</div>

	{#if subtab === 'manage'}
		{#if manageAvailableCategories.length > 1}
			<div class="category-dropdown" bind:this={manageCategoryDropdownEl}>
				<button
					type="button"
					class="category-trigger"
					class:active={manageSelectedCategories.size > 0}
					onclick={() => (manageCategoriesOpen = !manageCategoriesOpen)}
				>
					Categorías{manageSelectedCategories.size > 0 ? ` (${manageSelectedCategories.size})` : ''}
					<ChevronDown size={13} />
				</button>
				{#if manageCategoriesOpen}
					<div class="category-panel">
						{#each manageAvailableCategories as cat (cat)}
							<button
								type="button"
								class="category-option"
								class:active={manageSelectedCategories.has(cat)}
								onclick={() => toggleManageCategory(cat)}
							>
								{#if manageSelectedCategories.has(cat)}<Check size={12} />{:else}<span class="option-spacer"></span>{/if}
								{categoryLabel(cat)}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
		<div class="manage-toolbar">
			<button type="button" class="tool-btn" onclick={toggleSelectAll} disabled={installedFiltered.length === 0}>
				{selected.size === installedFiltered.length && installedFiltered.length > 0 ? 'Ninguno' : 'Seleccionar todos'}
			</button>
			<button
				type="button"
				class="tool-btn"
				disabled={addingLocal || dialogsBlocked}
				title={dialogsBlocked ? 'Activá el modo Manual en Ajustes para usar esto' : undefined}
				onclick={handleAddLocal}
			>
				{#if addingLocal}<Loader2 size={13} class="spin" />{:else}<Upload size={13} />{/if}
				Añadir por archivo
			</button>
			{#if kind === 'mod'}
				<button type="button" class="tool-btn" disabled={checkingUpdates} onclick={handleCheckUpdatesSelected}>
					{#if checkingUpdates}<Loader2 size={13} class="spin" />{:else}<RefreshCw size={13} />{/if}
					Buscar actualizaciones{selected.size > 0 ? ` (${selected.size})` : ''}
				</button>
			{/if}
			<button
				type="button"
				class="tool-btn danger"
				disabled={selected.size === 0 || removingSelected}
				onclick={handleRemoveSelected}
			>
				{#if removingSelected}<Loader2 size={13} class="spin" />{:else}<Trash2 size={13} />{/if}
				Quitar{selected.size > 0 ? ` (${selected.size})` : ''}
			</button>
		</div>
		{#if dialogsBlocked}
			<p class="hint">Añadir por archivo está desactivado en modo Automático — activá "Manual" en Ajustes.</p>
		{/if}
		{#if updateMsg}<p class="hint">{updateMsg}</p>{/if}
		{#if manageError}<p class="error">{manageError}</p>{/if}

		{#if duplicateGroups.length > 0}
			<div class="duplicate-warning">
				<AlertTriangle size={13} />
				<span>
					{duplicateGroups.length === 1 ? 'Hay un mod' : `Hay ${duplicateGroups.length} mods`}
					instalado dos veces (versiones distintas del mismo mod a la vez) — puede causar crashes.
					Revisá: {duplicateGroups.map((g) => g.join(' + ')).join(' · ')}
				</span>
			</div>
		{/if}

		{#if updates.length > 0}
			<button type="button" class="update-all-btn" disabled={updatingAll} onclick={handleUpdateAll}>
				{#if updatingAll}<Loader2 size={12} class="spin" />{:else}<RefreshCw size={12} />{/if}
				Actualizar todos ({updates.length})
			</button>
		{/if}

		{#if installed.length === 0}
			<p class="hint">Todavía no instalaste ningún {kind === 'mod' ? 'mod' : kind === 'shader' ? 'shader' : 'resource pack'} acá.</p>
		{:else if installedFiltered.length === 0}
			<p class="hint">Ningún {kind === 'mod' ? 'mod' : kind === 'shader' ? 'shader' : 'resource pack'} instalado coincide con esas categorías.</p>
		{:else}
			<div class="installed">
				{#each installedFiltered as item (item.filename)}
					{@const update = updates.find((u) => u.filename === item.filename)}
					<div class="installed-row">
						<button
							type="button"
							class="checkbox"
							class:checked={selected.has(item.filename)}
							onclick={() => toggleSelect(item.filename)}
							aria-label="Seleccionar"
						>
							{#if selected.has(item.filename)}<Check size={11} />{/if}
						</button>
						{#if item.icon_url}
							<img class="installed-icon" src={item.icon_url} alt="" />
						{:else}
							<div class="installed-icon installed-icon-fallback"></div>
						{/if}
						<span class="filename">{item.title ?? item.filename}</span>
						{#if update}
							<button type="button" class="changelog-toggle" onclick={() => toggleChangelog(update)}>
								<FileText size={11} /> Novedades
							</button>
						{/if}
						<button type="button" class="icon-btn" onclick={() => handleRemoveOne(item.filename)} aria-label="Quitar">
							<Trash2 size={13} />
						</button>
					</div>
					{#if update && changelogFor === update.new_version_id}
						<div class="changelog-box">
							{#if loadingChangelog}<Loader2 size={13} class="spin" />{:else}<p>{changelogText}</p>{/if}
						</div>
					{/if}
				{/each}
			</div>
		{/if}
	{:else}
		<div class="category-dropdown" bind:this={categoryDropdownEl}>
			<button
				type="button"
				class="category-trigger"
				class:active={selectedCategories.size > 0}
				onclick={() => (categoriesOpen = !categoriesOpen)}
			>
				Categorías{selectedCategories.size > 0 ? ` (${selectedCategories.size})` : ''}
				<ChevronDown size={13} />
			</button>
			{#if categoriesOpen}
				<div class="category-panel">
					{#each CATEGORIES[kind] as cat (cat)}
						<button
							type="button"
							class="category-option"
							class:active={selectedCategories.has(cat)}
							onclick={() => toggleCategory(cat)}
						>
							{#if selectedCategories.has(cat)}<Check size={12} />{:else}<span class="option-spacer"></span>{/if}
							{categoryLabel(cat)}
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<div class="search-row">
			{#if searching}
				<Loader2 size={14} class="spin search-icon" />
			{:else}
				<Search size={14} class="search-icon" />
			{/if}
			<input type="text" bind:value={query} placeholder={`Buscar ${NOUN[kind]} en Modrinth…`} />
		</div>

		{#if downloadError}<p class="error">{downloadError}</p>{/if}

		{#if kind === 'mod' && !query.trim() && favoriteMods.length > 0}
			<div class="results">
				<span class="section-label">Favoritos</span>
				{#each favoriteMods as mod (mod.projectId)}
					<div class="mod-card anim-fade-in">
						{#if mod.iconUrl}<img src={mod.iconUrl} alt={mod.title} />{:else}<div class="mod-icon-fallback"></div>{/if}
						<div class="mod-info">
							<span class="mod-title">{mod.title}</span>
						</div>
						<button type="button" class="install-btn" disabled={installingId === mod.projectId} onclick={() => handleInstall(mod.projectId)}>
							{#if installingId === mod.projectId}<Loader2 size={14} class="spin" />{:else}<Download size={14} />{/if}
						</button>
					</div>
				{/each}
			</div>
		{/if}

		<div class="results">
			{#if results.length > 0}<span class="section-label">Resultados</span>{/if}
			{#each results as item (item.project_id)}
				{@const alreadyInstalled = installedProjectIds.has(item.project_id)}
				<div class="mod-card anim-fade-in">
					{#if item.icon_url}<img src={item.icon_url} alt={item.title} />{:else}<div class="mod-icon-fallback"></div>{/if}
					<div class="mod-info">
						<span class="mod-title">{item.title}</span>
						<p class="mod-desc">{item.description}</p>
					</div>
					{#if kind === 'mod'}
						<button
							type="button"
							class="favorite-btn"
							class:active={isFavoriteMod(item.project_id)}
							onclick={() => toggleFavoriteMod({ projectId: item.project_id, title: item.title, iconUrl: item.icon_url })}
							aria-label="Favorito"
						>
							<Star size={14} fill={isFavoriteMod(item.project_id) ? 'currentColor' : 'none'} />
						</button>
					{/if}
					<button
						type="button"
						class="version-toggle"
						onclick={() => toggleVersions(item.project_id)}
						aria-label="Ver versiones"
						title="Ver versiones"
					>
						<ChevronDown size={13} />
					</button>
					{#if alreadyInstalled}
						<span class="installed-badge" title="Ya instalado"><Check size={13} /></span>
					{:else}
						<button type="button" class="install-btn" disabled={installingId === item.project_id} onclick={() => handleInstall(item.project_id)}>
							{#if installingId === item.project_id}<Loader2 size={14} class="spin" />{:else}<Download size={14} />{/if}
						</button>
					{/if}
				</div>
				{#if openVersionsFor === item.project_id}
					<div class="version-row">
						{#if loadingVersions}
							<Loader2 size={13} class="spin" />
						{:else if versionOptions.length === 0}
							<span class="hint">No hay versiones compatibles con esta instancia.</span>
						{:else}
							{#each versionOptions as v (v.id)}
								<button
									type="button"
									class="version-option"
									disabled={installingId === item.project_id}
									onclick={() => handleInstall(item.project_id, v.id)}
								>
									{v.version_number || v.name}
								</button>
							{/each}
						{/if}
					</div>
				{/if}
			{/each}
		</div>
	{/if}
</div>

<style>
	.content-manager {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.subtabs {
		display: flex;
		gap: 6px;
		border-bottom: 1px solid var(--border);
		padding-bottom: 8px;
	}

	.subtab {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		font-size: 0.8rem;
		font-weight: 700;
		cursor: pointer;
	}

	.subtab.active {
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		color: var(--accent);
	}

	.subtab .count {
		font-size: 0.65rem;
		opacity: 0.8;
	}

	.manage-toolbar {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.tool-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.72rem;
		font-weight: 600;
		cursor: pointer;
	}

	.tool-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.tool-btn.danger:not(:disabled):hover {
		border-color: var(--color-error);
		color: var(--color-error);
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
	}

	.error {
		color: var(--color-error);
		font-size: 0.78rem;
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

	.duplicate-warning {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 10px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid color-mix(in srgb, var(--color-error) 40%, var(--border));
		background: color-mix(in srgb, var(--color-error) 12%, var(--bg-input));
		color: var(--color-error);
		font-size: 0.74rem;
		line-height: 1.4;
	}

	.duplicate-warning :global(svg) {
		flex-shrink: 0;
		margin-top: 1px;
	}

	.update-all-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 4px 9px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		color: var(--accent);
		font-size: 0.68rem;
		font-weight: 700;
		cursor: pointer;
		align-self: flex-start;
	}

	.update-all-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.checkbox {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		flex-shrink: 0;
		border-radius: 4px;
		border: 1px solid var(--border);
		background: var(--bg-card);
		color: var(--accent);
		cursor: pointer;
		padding: 0;
	}

	.checkbox.checked {
		background: color-mix(in srgb, var(--accent) 20%, var(--bg-card));
		border-color: var(--accent);
	}

	.changelog-toggle {
		display: flex;
		align-items: center;
		gap: 4px;
		flex-shrink: 0;
		padding: 3px 7px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: transparent;
		color: var(--text-muted);
		font-size: 0.64rem;
		cursor: pointer;
	}

	.changelog-box {
		padding: 8px 10px;
		margin: -2px 0 6px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-card);
		border: 1px solid var(--border);
		font-size: 0.72rem;
		color: var(--text-secondary);
		white-space: pre-wrap;
		max-height: 140px;
		overflow-y: auto;
	}

	.favorite-btn,
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

	.installed-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		margin-bottom: 4px;
	}

	.installed-icon {
		width: 20px;
		height: 20px;
		border-radius: 4px;
		flex-shrink: 0;
		background: var(--bg-card);
		object-fit: cover;
	}

	.installed-icon-fallback {
		background: var(--bg-card);
		border: 1px solid var(--border);
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
	}

	.icon-btn:hover {
		color: var(--color-error);
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

	.results {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.mod-card {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-card);
	}

	.mod-card img,
	.mod-icon-fallback {
		width: 36px;
		height: 36px;
		border-radius: var(--border-radius-sm);
		flex-shrink: 0;
		background: var(--bg-input);
	}

	.mod-info {
		flex: 1;
		min-width: 0;
	}

	.mod-title {
		font-size: 0.82rem;
		font-weight: 600;
	}

	.mod-desc {
		font-size: 0.7rem;
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
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
