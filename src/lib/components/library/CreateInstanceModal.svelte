<script lang="ts">
	import { onMount } from 'svelte';
	import { getAvailableVersions, createInstance } from '$lib/api/tflApi';
	import type { InstanceData, MinecraftVersion, Loader } from '$lib/types/types';
	import { Loader2 } from 'lucide-svelte';

	let {
		onClose,
		onCreated
	}: {
		onClose: () => void;
		onCreated: (instance: InstanceData) => void;
	} = $props();

	const LOADERS: { id: Loader; label: string; color: string }[] = [
		{ id: 'vanilla', label: 'Vanilla', color: 'var(--loader-vanilla)' },
		{ id: 'fabric', label: 'Fabric', color: 'var(--loader-fabric)' },
		{ id: 'forge', label: 'Forge', color: 'var(--loader-forge)' },
		{ id: 'neoforge', label: 'NeoForge', color: 'var(--loader-neoforge)' },
		{ id: 'quilt', label: 'Quilt', color: 'var(--loader-quilt)' }
	];

	let name = $state('');
	let versions = $state<MinecraftVersion[]>([]);
	let selectedVersion = $state('');
	let selectedLoader = $state<Loader>('vanilla');
	let loadingVersions = $state(true);
	let creating = $state(false);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			const all = await getAvailableVersions();
			versions = all.filter((v) => v.type === 'release');
			selectedVersion = versions[0]?.id ?? '';
		} catch (e) {
			error = String(e);
		} finally {
			loadingVersions = false;
		}
	});

	async function handleCreate() {
		if (!name.trim() || !selectedVersion) return;
		creating = true;
		error = null;
		try {
			const instance = await createInstance(name.trim(), selectedVersion, selectedLoader);
			onCreated(instance);
		} catch (e) {
			error = String(e);
		} finally {
			creating = false;
		}
	}
</script>

<div
	class="overlay"
	onclick={onClose}
	onkeydown={(e) => e.key === 'Escape' && onClose()}
	role="button"
	tabindex="-1"
>
	<div
		class="modal"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<h2>Nueva instancia</h2>

		<label for="instance-name">Nombre</label>
		<input
			id="instance-name"
			type="text"
			bind:value={name}
			placeholder="Mi mundo"
			autocomplete="off"
		/>

		<label for="instance-version">Versión de Minecraft</label>
		{#if loadingVersions}
			<div class="loading">
				<Loader2 size={16} class="spin" /> Cargando versiones…
			</div>
		{:else}
			<select id="instance-version" bind:value={selectedVersion}>
				{#each versions as v (v.id)}
					<option value={v.id}>{v.id}</option>
				{/each}
			</select>
		{/if}

		<label for="instance-loader">Loader</label>
		<div class="loader-grid" id="instance-loader">
			{#each LOADERS as l (l.id)}
				<button
					type="button"
					class="loader-btn"
					class:active={selectedLoader === l.id}
					style="--loader-color: {l.color}"
					onclick={() => (selectedLoader = l.id)}
				>
					<span class="loader-dot"></span>
					{l.label}
				</button>
			{/each}
		</div>

		<p class="hint">
			La versión del loader (Fabric/Forge/NeoForge/Quilt) se resuelve sola — la más
			reciente/recomendada para esta versión de Minecraft.
		</p>

		{#if error}
			<p class="error">{error}</p>
		{/if}

		<div class="actions">
			<button type="button" class="btn" onclick={onClose}>Cancelar</button>
			<button
				type="button"
				class="btn primary"
				disabled={creating || !name.trim() || !selectedVersion}
				onclick={handleCreate}
			>
				{#if creating}<Loader2 size={16} class="spin" />{/if}
				Crear
			</button>
		</div>
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

	.modal {
		width: 420px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		box-shadow: var(--shadow-lg);
	}

	.modal h2 {
		font-size: var(--text-lg);
		margin-bottom: 8px;
	}

	label {
		font-size: 0.75rem;
		color: var(--text-secondary);
		margin-top: 8px;
	}

	input,
	select {
		padding: 10px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.88rem;
	}

	.loader-grid {
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		gap: 6px;
	}

	.loader-btn {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 5px;
		padding: 10px 4px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.72rem;
		font-weight: 600;
		cursor: pointer;
		transition:
			background 0.15s,
			border-color 0.15s,
			color 0.15s;
	}

	.loader-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--loader-color);
		opacity: 0.55;
		transition: opacity 0.15s;
	}

	.loader-btn:hover {
		border-color: var(--loader-color);
	}

	.loader-btn.active {
		background: color-mix(in srgb, var(--loader-color) 16%, var(--bg-input));
		border-color: var(--loader-color);
		color: var(--text-primary);
	}

	.loader-btn.active .loader-dot {
		opacity: 1;
	}

	.hint {
		font-size: 0.7rem;
		color: var(--text-muted);
		margin-top: 4px;
	}

	.error {
		font-size: 0.75rem;
		color: var(--color-error);
	}

	.loading {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.8rem;
		color: var(--text-secondary);
	}

	.actions {
		display: flex;
		gap: 8px;
		margin-top: 16px;
	}

	.btn {
		flex: 1;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-weight: 600;
		font-size: 0.82rem;
		cursor: pointer;
	}

	.btn.primary {
		background: var(--accent);
		color: var(--accent-text);
		border-color: var(--accent);
	}

	.btn:disabled {
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
