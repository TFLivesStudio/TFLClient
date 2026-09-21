<script lang="ts">
	import { onMount } from 'svelte';
	import type { InstanceData, ModSearchHit } from '$lib/types/types';
	import { searchShaders, installShader, getInstanceShaders, removeShader } from '$lib/api/tflApi';
	import { Search, Download, Trash2, Loader2 } from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();

	// 3 opciones fijas en vez de tener que buscar y elegir un shader pack a
	// mano — project_id reales de Modrinth, verificados en vivo, no
	// inventados. Cada uno queda instalado como cualquier otro shader (se
	// puede sacar desde "Instalados" como siempre).
	const GRAPHICS_PRESETS = [
		{ label: 'Bajo', projectId: 'izsIPI7a' }, // MakeUp - Ultra Fast
		{ label: 'Medio', projectId: 'HVnmMxH1' }, // Complementary Shaders - Reimagined
		{ label: 'Alto', projectId: 'R6NEzAwj' } // Complementary Shaders - Unbound
	];

	let query = $state('');
	let results = $state<ModSearchHit[]>([]);
	let installed = $state<string[]>([]);
	let searching = $state(false);
	let installingId = $state<string | null>(null);
	let error = $state<string | null>(null);

	async function refreshInstalled() {
		installed = await getInstanceShaders(instance.name);
	}

	onMount(refreshInstalled);

	let searchToken = 0;
	async function runSearch(q: string) {
		const token = ++searchToken;
		searching = true;
		error = null;
		try {
			const hits = await searchShaders(q, instance.mc_version);
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
			await installShader(instance.name, projectId, instance.mc_version);
			await refreshInstalled();
		} catch (e) {
			error = String(e);
		} finally {
			installingId = null;
		}
	}

	async function handleRemove(filename: string) {
		try {
			await removeShader(instance.name, filename);
			await refreshInstalled();
		} catch (e) {
			error = String(e);
		}
	}
</script>

<div class="shaders-panel">
	{#if instance.loader === 'vanilla'}
		<p class="hint">
			Vanilla no soporta shaders — necesitás un mod tipo Iris u OptiFine, que a su vez requiere
			Fabric, Forge, NeoForge o Quilt.
		</p>
	{:else}
		<p class="hint">
			Instalar un shader acá solo lo copia a la instancia — todavía necesitás un mod compatible
			(Iris, Oculus, OptiFine…) instalado en la pestaña Mods para que Minecraft lo use.
		</p>

		<div class="presets">
			<span class="section-label">Perfil gráfico rápido</span>
			<div class="preset-row">
				{#each GRAPHICS_PRESETS as preset (preset.projectId)}
					<button
						type="button"
						class="preset-btn"
						disabled={installingId === preset.projectId}
						onclick={() => handleInstall(preset.projectId)}
					>
						{#if installingId === preset.projectId}
							<Loader2 size={13} class="spin" />
						{/if}
						{preset.label}
					</button>
				{/each}
			</div>
		</div>

		<div class="search-row">
			{#if searching}
				<Loader2 size={14} class="spin search-icon" />
			{:else}
				<Search size={14} class="search-icon" />
			{/if}
			<input type="text" bind:value={query} placeholder="Buscar shaders en Modrinth…" />
		</div>

		{#if error}
			<p class="error">{error}</p>
		{/if}

		{#if installed.length > 0}
			<div class="installed">
				<span class="section-label">Instalados ({installed.length})</span>
				{#each installed as filename (filename)}
					<div class="installed-row">
						<span class="filename">{filename}</span>
						<button
							type="button"
							class="icon-btn"
							onclick={() => handleRemove(filename)}
							aria-label="Quitar"
						>
							<Trash2 size={13} />
						</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if results.length > 0}
			<div class="results">
				<span class="section-label">Resultados</span>
				{#each results as shader (shader.project_id)}
					<div class="mod-card anim-fade-in">
						{#if shader.icon_url}
							<img src={shader.icon_url} alt={shader.title} />
						{:else}
							<div class="mod-icon-fallback"></div>
						{/if}
						<div class="mod-info">
							<span class="mod-title">{shader.title}</span>
							<p class="mod-desc">{shader.description}</p>
						</div>
						<button
							type="button"
							class="install-btn"
							disabled={installingId === shader.project_id}
							onclick={() => handleInstall(shader.project_id)}
						>
							{#if installingId === shader.project_id}
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
	.shaders-panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
	}

	.presets {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.preset-row {
		display: flex;
		gap: 8px;
	}

	.preset-btn {
		flex: 1;
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
		font-weight: 700;
		cursor: pointer;
	}

	.preset-btn:hover:not(:disabled) {
		border-color: var(--accent);
		color: var(--accent);
	}

	.preset-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
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
		justify-content: space-between;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		margin-bottom: 4px;
	}

	.filename {
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
