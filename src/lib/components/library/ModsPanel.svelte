<script lang="ts">
	import { onMount } from 'svelte';
	import type { InstanceData, ModSearchHit, InstalledModInfo, ModUpdateAvailable } from '$lib/types/types';
	import {
		searchMods,
		installMod,
		getInstalledModsInfo,
		checkModUpdates,
		updateAllMods,
		getModVersionChangelog,
		findDuplicateMods,
		removeMod
	} from '$lib/api/tflApi';
	import { favoriteMods, isFavoriteMod, toggleFavoriteMod } from '$lib/state/modFavorites.svelte';
	import { Search, Download, Trash2, Loader2, RefreshCw, AlertTriangle, Star, FileText } from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();

	let query = $state('');
	let results = $state<ModSearchHit[]>([]);
	let installed = $state<InstalledModInfo[]>([]);
	let updates = $state<ModUpdateAvailable[]>([]);
	let duplicateGroups = $state<string[][]>([]);
	let searching = $state(false);
	let installingId = $state<string | null>(null);
	let updatingAll = $state(false);
	let error = $state<string | null>(null);

	let changelogFor = $state<string | null>(null);
	let changelogText = $state<string | null>(null);
	let loadingChangelog = $state(false);

	async function refreshInstalled() {
		installed = await getInstalledModsInfo(instance.name);
		try {
			updates = await checkModUpdates(instance.name);
		} catch {
			updates = [];
		}
		try {
			duplicateGroups = await findDuplicateMods(instance.name);
		} catch {
			duplicateGroups = [];
		}
	}

	onMount(refreshInstalled);

	async function handleUpdateAll() {
		updatingAll = true;
		error = null;
		try {
			await updateAllMods(instance.name);
			await refreshInstalled();
		} catch (e) {
			error = String(e);
		} finally {
			updatingAll = false;
		}
	}

	async function toggleChangelog(update: ModUpdateAvailable) {
		if (changelogFor === update.new_version_id) {
			changelogFor = null;
			changelogText = null;
			return;
		}
		changelogFor = update.new_version_id;
		changelogText = null;
		loadingChangelog = true;
		try {
			changelogText = (await getModVersionChangelog(update.new_version_id)) || 'Sin notas de cambios.';
		} catch (e) {
			changelogText = `No se pudo cargar: ${e}`;
		} finally {
			loadingChangelog = false;
		}
	}

	let searchToken = 0;
	async function runSearch(q: string) {
		const token = ++searchToken;
		searching = true;
		error = null;
		try {
			const hits = await searchMods(q, instance.mc_version, instance.loader);
			if (token === searchToken) results = hits;
		} catch (e) {
			if (token === searchToken) error = String(e);
		} finally {
			if (token === searchToken) searching = false;
		}
	}

	// Búsqueda en vivo mientras se tipea — sin botón "Buscar" — con debounce
	// corto para no golpear la API en cada tecla.
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		const q = query;
		clearTimeout(debounceTimer);
		if (!q.trim()) {
			results = [];
			searching = false;
			return;
		}
		debounceTimer = setTimeout(() => runSearch(q), 300);
		return () => clearTimeout(debounceTimer);
	});

	async function handleInstall(projectId: string) {
		installingId = projectId;
		error = null;
		try {
			await installMod(instance.name, projectId, instance.mc_version, instance.loader);
			await refreshInstalled();
		} catch (e) {
			error = String(e);
		} finally {
			installingId = null;
		}
	}

	async function handleRemove(filename: string) {
		try {
			await removeMod(instance.name, filename);
			await refreshInstalled();
		} catch (e) {
			error = String(e);
		}
	}
</script>

<div class="mods-panel">
	{#if instance.loader === 'vanilla'}
		<p class="hint">
			Vanilla no soporta mods — elegí Fabric, Forge, NeoForge o Quilt al crear la instancia.
		</p>
	{:else}
		<div class="search-row">
			{#if searching}
				<Loader2 size={14} class="spin search-icon" />
			{:else}
				<Search size={14} class="search-icon" />
			{/if}
			<input type="text" bind:value={query} placeholder="Buscar mods en Modrinth…" />
		</div>

		{#if error}
			<p class="error">{error}</p>
		{/if}

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

		{#if installed.length > 0}
			<div class="installed">
				<div class="installed-header">
					<span class="section-label">Instalados ({installed.length})</span>
					{#if updates.length > 0}
						<button type="button" class="update-all-btn" disabled={updatingAll} onclick={handleUpdateAll}>
							{#if updatingAll}<Loader2 size={12} class="spin" />{:else}<RefreshCw size={12} />{/if}
							Actualizar todos ({updates.length})
						</button>
					{/if}
				</div>
				{#each installed as mod (mod.filename)}
					{@const update = updates.find((u) => u.filename === mod.filename)}
					<div class="installed-row">
						<span class="filename">{mod.title ?? mod.filename}</span>
						{#if update}
							<button type="button" class="changelog-toggle" onclick={() => toggleChangelog(update)}>
								<FileText size={11} /> Novedades
							</button>
						{/if}
						<button
							type="button"
							class="icon-btn"
							onclick={() => handleRemove(mod.filename)}
							aria-label="Quitar"
						>
							<Trash2 size={13} />
						</button>
					</div>
					{#if update && changelogFor === update.new_version_id}
						<div class="changelog-box">
							{#if loadingChangelog}
								<Loader2 size={13} class="spin" />
							{:else}
								<p>{changelogText}</p>
							{/if}
						</div>
					{/if}
				{/each}
			</div>
		{/if}

		{#if !query.trim() && favoriteMods.length > 0}
			<div class="results">
				<span class="section-label">Favoritos</span>
				{#each favoriteMods as mod (mod.projectId)}
					<div class="mod-card anim-fade-in">
						{#if mod.iconUrl}
							<img src={mod.iconUrl} alt={mod.title} />
						{:else}
							<div class="mod-icon-fallback"></div>
						{/if}
						<div class="mod-info">
							<span class="mod-title">{mod.title}</span>
						</div>
						<button
							type="button"
							class="install-btn"
							disabled={installingId === mod.projectId}
							onclick={() => handleInstall(mod.projectId)}
						>
							{#if installingId === mod.projectId}
								<Loader2 size={14} class="spin" />
							{:else}
								<Download size={14} />
							{/if}
						</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if results.length > 0}
			<div class="results">
				<span class="section-label">Resultados</span>
				{#each results as mod (mod.project_id)}
					<div class="mod-card anim-fade-in">
						{#if mod.icon_url}
							<img src={mod.icon_url} alt={mod.title} />
						{:else}
							<div class="mod-icon-fallback"></div>
						{/if}
						<div class="mod-info">
							<span class="mod-title">{mod.title}</span>
							<p class="mod-desc">{mod.description}</p>
						</div>
						<button
							type="button"
							class="favorite-btn"
							class:active={isFavoriteMod(mod.project_id)}
							onclick={() =>
								toggleFavoriteMod({
									projectId: mod.project_id,
									title: mod.title,
									iconUrl: mod.icon_url
								})}
							aria-label="Favorito"
						>
							<Star size={14} fill={isFavoriteMod(mod.project_id) ? 'currentColor' : 'none'} />
						</button>
						<button
							type="button"
							class="install-btn"
							disabled={installingId === mod.project_id}
							onclick={() => handleInstall(mod.project_id)}
						>
							{#if installingId === mod.project_id}
								<Loader2 size={14} class="spin" />
							{:else}
								<Download size={14} />
							{/if}
						</button>
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>

<style>
	.mods-panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.82rem;
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

	.installed-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 6px;
	}

	.installed-header .section-label {
		margin-bottom: 0;
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
	}

	.update-all-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
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

	.favorite-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
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

	.installed-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		margin-bottom: 4px;
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
