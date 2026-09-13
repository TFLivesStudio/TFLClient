<script lang="ts">
	import { onMount } from 'svelte';
	import type { InstanceData } from '$lib/types/types';
	import {
		launchInstance,
		deleteInstance,
		renameInstance,
		updateInstanceMemory,
		getRecommendedRam
	} from '$lib/api/tflApi';
	import ModsPanel from './ModsPanel.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { Play, Loader2, Pencil, Trash2, Check, X } from 'lucide-svelte';

	let {
		instance,
		onChanged
	}: {
		instance: InstanceData;
		onChanged: (instance: InstanceData | null) => void;
	} = $props();

	let launching = $state(false);
	let error = $state<string | null>(null);
	let tab = $state<'details' | 'mods'>('details');
	let showDeleteConfirm = $state(false);
	let editingName = $state(false);
	let nameDraft = $state(instance.name);
	let ramRecommended = $state<number | null>(null);

	$effect(() => {
		nameDraft = instance.name;
	});

	onMount(loadRamHint);

	const LOADER_META: Record<string, { label: string; color: string }> = {
		vanilla: { label: 'Vanilla', color: 'var(--loader-vanilla)' },
		fabric: { label: 'Fabric', color: 'var(--loader-fabric)' },
		forge: { label: 'Forge', color: 'var(--loader-forge)' },
		neoforge: { label: 'NeoForge', color: 'var(--loader-neoforge)' },
		quilt: { label: 'Quilt', color: 'var(--loader-quilt)' }
	};
	const loaderMeta = $derived(LOADER_META[instance.loader]);

	const lastPlayedLabel = $derived.by(() => {
		if (!instance.last_played) return 'Nunca jugada';
		const diffMs = Date.now() - instance.last_played * 1000;
		const days = Math.floor(diffMs / 86_400_000);
		if (days <= 0) return 'Jugada hoy';
		if (days === 1) return 'Jugada ayer';
		return `Jugada hace ${days} días`;
	});

	async function handlePlay() {
		launching = true;
		error = null;
		try {
			await launchInstance(instance.name);
		} catch (e) {
			error = String(e);
		} finally {
			launching = false;
		}
	}

	async function confirmRename() {
		const trimmed = nameDraft.trim();
		if (!trimmed || trimmed === instance.name) {
			editingName = false;
			nameDraft = instance.name;
			return;
		}
		try {
			const updated = await renameInstance(instance.name, trimmed);
			editingName = false;
			onChanged(updated);
		} catch (e) {
			error = String(e);
		}
	}

	async function handleDelete() {
		showDeleteConfirm = false;
		try {
			await deleteInstance(instance.name);
			onChanged(null);
		} catch (e) {
			error = String(e);
		}
	}

	async function loadRamHint() {
		if (ramRecommended !== null) return;
		try {
			const r = await getRecommendedRam();
			ramRecommended = r.recommended_max_mb;
		} catch {
			// silencioso — solo es una sugerencia
		}
	}

	async function setMemory(min: number | null, max: number | null) {
		try {
			const updated = await updateInstanceMemory(instance.name, min, max);
			onChanged(updated);
		} catch (e) {
			error = String(e);
		}
	}
</script>

<div class="instance-detail">
	<div class="hero">
		<div class="hero-icon" style="--loader-color: {loaderMeta.color}">
			{instance.name.charAt(0).toUpperCase()}
		</div>

		<div class="hero-info">
			{#if editingName}
				<div class="name-edit">
					<input
						type="text"
						bind:value={nameDraft}
						onkeydown={(e) => {
							if (e.key === 'Enter') confirmRename();
							if (e.key === 'Escape') {
								editingName = false;
								nameDraft = instance.name;
							}
						}}
						{@attach (el) => el.focus()}
					/>
					<button type="button" class="icon-btn" onclick={confirmRename} aria-label="Guardar">
						<Check size={15} />
					</button>
					<button
						type="button"
						class="icon-btn"
						onclick={() => {
							editingName = false;
							nameDraft = instance.name;
						}}
						aria-label="Cancelar"
					>
						<X size={15} />
					</button>
				</div>
			{:else}
				<div class="name-row">
					<h2>{instance.name}</h2>
					<button
						type="button"
						class="icon-btn"
						onclick={() => (editingName = true)}
						aria-label="Renombrar"
					>
						<Pencil size={13} />
					</button>
					<button
						type="button"
						class="icon-btn danger"
						onclick={() => (showDeleteConfirm = true)}
						aria-label="Eliminar"
					>
						<Trash2 size={13} />
					</button>
				</div>
			{/if}

			<div class="meta-row">
				<span class="loader-badge" style="--loader-color: {loaderMeta.color}"
					>{loaderMeta.label}</span
				>
				<span class="meta-text">{instance.mc_version}</span>
				<span class="meta-dot">·</span>
				<span class="meta-text">{lastPlayedLabel}</span>
			</div>
		</div>

		<button type="button" class="play-btn" disabled={launching} onclick={handlePlay}>
			{#if launching}
				<Loader2 size={16} class="spin" />
				Preparando…
			{:else}
				<Play size={16} fill="currentColor" />
				Jugar
			{/if}
		</button>
	</div>

	{#if error}
		<p class="error">{error}</p>
	{/if}

	<div class="tabs">
		<button
			type="button"
			class="tab-btn"
			class:active={tab === 'details'}
			onclick={() => (tab = 'details')}
		>
			Detalles
		</button>
		<button
			type="button"
			class="tab-btn"
			class:active={tab === 'mods'}
			onclick={() => (tab = 'mods')}
		>
			Mods
		</button>
	</div>

	{#if tab === 'details'}
		<section class="ram-section">
			<span class="section-label">Memoria (esta instancia)</span>
			<p class="hint">
				Vacío usa el valor global de Ajustes{ramRecommended
					? ` (recomendado: ${ramRecommended} MB)`
					: ''}.
			</p>
			<div class="ram-row">
				<label>
					Mínima (MB)
					<input
						type="number"
						value={instance.min_memory ?? ''}
						placeholder="Global"
						min="512"
						step="256"
						onchange={(e) => {
							const v = e.currentTarget.valueAsNumber;
							setMemory(Number.isNaN(v) ? null : v, instance.max_memory);
						}}
					/>
				</label>
				<label>
					Máxima (MB)
					<input
						type="number"
						value={instance.max_memory ?? ''}
						placeholder="Global"
						min="512"
						step="256"
						onchange={(e) => {
							const v = e.currentTarget.valueAsNumber;
							setMemory(instance.min_memory, Number.isNaN(v) ? null : v);
						}}
					/>
				</label>
			</div>
		</section>
	{:else}
		<ModsPanel {instance} />
	{/if}
</div>

{#if showDeleteConfirm}
	<ConfirmDialog
		title="Eliminar instancia"
		message={`"${instance.name}" y todos sus archivos (mundos, mods, configs) se van a borrar. No se puede deshacer.`}
		confirmLabel="Eliminar"
		danger
		onConfirm={handleDelete}
		onCancel={() => (showDeleteConfirm = false)}
	/>
{/if}

<style>
	.instance-detail {
		padding: 32px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		height: 100%;
		overflow-y: auto;
	}

	.hero {
		display: flex;
		align-items: center;
		gap: 18px;
	}

	.hero-icon {
		flex-shrink: 0;
		width: 64px;
		height: 64px;
		border-radius: var(--border-radius-lg);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.6rem;
		font-weight: 800;
		color: var(--loader-color);
		background: color-mix(in srgb, var(--loader-color) 16%, transparent);
		border: 1px solid color-mix(in srgb, var(--loader-color) 35%, transparent);
	}

	.hero-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.name-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.name-row h2 {
		font-size: 1.3rem;
	}

	.name-edit {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.name-edit input {
		font-size: 1.1rem;
		font-weight: 700;
		padding: 4px 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: var(--bg-input);
		color: var(--text-primary);
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.icon-btn:hover {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.06);
	}

	.icon-btn.danger:hover {
		color: var(--color-error);
	}

	.meta-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.loader-badge {
		font-size: 0.65rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.4px;
		padding: 3px 8px;
		border-radius: 999px;
		color: var(--loader-color);
		background: color-mix(in srgb, var(--loader-color) 16%, transparent);
	}

	.meta-text {
		font-size: 0.78rem;
		color: var(--text-secondary);
	}

	.meta-dot {
		color: var(--text-muted);
	}

	.play-btn {
		flex-shrink: 0;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		background: var(--accent);
		color: var(--accent-text);
		border: none;
		padding: 12px 28px;
		border-radius: var(--border-radius-sm);
		font-weight: 800;
		font-size: 0.85rem;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		cursor: pointer;
		box-shadow: var(--shadow-md);
		transition: transform 0.12s ease;
	}

	.play-btn:hover:not(:disabled) {
		transform: translateY(-1px);
	}

	.play-btn:disabled {
		opacity: 0.7;
		cursor: not-allowed;
	}

	.error {
		color: var(--color-error);
		font-size: 0.82rem;
	}

	.tabs {
		display: flex;
		gap: 4px;
		border-bottom: 1px solid var(--border);
	}

	.tab-btn {
		padding: 8px 4px;
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--text-secondary);
		font-size: 0.8rem;
		font-weight: 600;
		cursor: pointer;
		margin-right: 16px;
	}

	.tab-btn.active {
		color: var(--text-primary);
		border-bottom-color: var(--accent);
	}

	.ram-section {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		padding: 16px;
	}

	.section-label {
		display: block;
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.hint {
		font-size: 0.72rem;
		color: var(--text-muted);
		margin-top: 4px;
		margin-bottom: 12px;
	}

	.ram-row {
		display: flex;
		gap: 12px;
	}

	.ram-row label {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 0.72rem;
		color: var(--text-secondary);
	}

	.ram-row input {
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.85rem;
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
