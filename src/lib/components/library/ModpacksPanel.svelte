<script lang="ts">
	import { onMount } from 'svelte';
	import type { InstanceData, ModSearchHit, InstalledModpack, ModpackUpdateInfo } from '$lib/types/types';
	import {
		searchModpacks,
		installModpack,
		installModpackFromUrl,
		updateCommunityModpack,
		getInstanceModpacks,
		checkModpackUpdates,
		removeModpack
	} from '$lib/api/tflApi';
	import { Search, Download, Trash2, Loader2, Package, RefreshCw } from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();

	let query = $state('');
	let results = $state<ModSearchHit[]>([]);
	let installed = $state<InstalledModpack[]>([]);
	let updates = $state<Record<string, string>>({}); // version_id -> new_mrpack_url
	let searching = $state(false);
	let installingId = $state<string | null>(null);
	let removingId = $state<string | null>(null);
	let updatingId = $state<string | null>(null);
	let error = $state<string | null>(null);

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

	let searchToken = 0;
	async function runSearch(q: string) {
		const token = ++searchToken;
		searching = true;
		error = null;
		try {
			const hits = await searchModpacks(q, instance.mc_version, instance.loader);
			if (token === searchToken) results = hits;
		} catch (e) {
			if (token === searchToken) error = String(e);
		} finally {
			if (token === searchToken) searching = false;
		}
	}

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
			await installModpack(instance.name, projectId, instance.mc_version, instance.loader);
			await refreshInstalled();
		} catch (e) {
			error = String(e);
		} finally {
			installingId = null;
		}
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
			Vanilla no soporta modpacks — elegí Fabric, Forge, NeoForge o Quilt al crear la instancia.
		</p>
	{:else}
		<p class="hint">
			Instalar un modpack agrega sus mods y configuración a esta instancia — no reemplaza lo que
			ya tenías. Quitarlo borra exactamente lo que trajo, nada más.
		</p>

		<div class="search-row">
			{#if searching}
				<Loader2 size={14} class="spin search-icon" />
			{:else}
				<Search size={14} class="search-icon" />
			{/if}
			<input type="text" bind:value={query} placeholder="Buscar modpacks en Modrinth…" />
		</div>

		{#if error}
			<p class="error">{error}</p>
		{/if}

		{#if installed.length > 0}
			<div class="installed">
				<span class="section-label">Instalados ({installed.length})</span>
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
								Actualizar
							</button>
						{/if}
						<button
							type="button"
							class="icon-btn"
							disabled={removingId === pack.version_id}
							onclick={() => handleRemove(pack.version_id)}
							aria-label="Quitar"
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

		{#if results.length > 0}
			<div class="results">
				<span class="section-label">Resultados</span>
				{#each results as pack (pack.project_id)}
					<div class="mod-card anim-fade-in">
						{#if pack.icon_url}
							<img src={pack.icon_url} alt={pack.title} />
						{:else}
							<div class="mod-icon-fallback"></div>
						{/if}
						<div class="mod-info">
							<span class="mod-title">{pack.title}</span>
							<p class="mod-desc">{pack.description}</p>
						</div>
						<button
							type="button"
							class="install-btn"
							disabled={installingId === pack.project_id}
							onclick={() => handleInstall(pack.project_id)}
						>
							{#if installingId === pack.project_id}
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
	.modpacks-panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
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
