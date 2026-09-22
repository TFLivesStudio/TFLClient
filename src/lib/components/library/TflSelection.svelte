<script lang="ts">
	import { onMount } from 'svelte';
	import type { InstanceData, Loader, TflSelectionEntry } from '$lib/types/types';
	import {
		getTflSelection,
		installModpack,
		installModpackFromUrl,
		createInstance
	} from '$lib/api/tflApi';
	import { X, Download, Loader2, Check } from 'lucide-svelte';
	import Tfl from '$lib/icons/Tfl.svelte';

	let {
		instances,
		onClose,
		onInstanceCreated
	}: {
		instances: InstanceData[];
		onClose: () => void;
		onInstanceCreated?: (instance: InstanceData) => void;
	} = $props();

	// Sentinel para las opciones "crear instancia nueva" mezcladas en el
	// mismo <select> que las instancias existentes — solo tiene sentido
	// para modpacks comunitarios, que ya traen mc_version/loader exactos
	// en su manifest (ver tfl_selection.rs). Los de Modrinth no declaran
	// eso acá (se resuelve contra la instancia elegida en el momento de
	// instalar), así que para esos se sigue pidiendo una instancia ya
	// creada — automatizarlo requeriría consultar Modrinth por separado,
	// queda para otra vuelta si hace falta.
	const NEW_PREFIX = 'new:';

	let entries = $state<TflSelectionEntry[]>([]);
	let loading = $state(true);
	let selectedInstance = $state<Record<string, string>>({});
	let installingFor = $state<string | null>(null);
	let doneFor = $state<Set<string>>(new Set());
	let errorFor = $state<Record<string, string>>({});

	// Modrinth: cualquier instancia con loader (la mejor versión se
	// resuelve en el momento contra la API de Modrinth). Comunitarios: solo
	// instancias cuya versión de Minecraft + loader coincida EXACTO con
	// alguno de los builds que ofrece el pack (puede tener varios, uno por
	// versión soportada).
	function compatibleInstancesFor(entry: TflSelectionEntry): InstanceData[] {
		if (entry.source === 'modrinth') {
			return instances.filter((i) => i.loader !== 'vanilla');
		}
		return instances.filter((i) =>
			entry.versions.some((v) => v.loader === i.loader && v.mc_version === i.mc_version)
		);
	}

	// Una opción "crear nueva" por cada build que ofrece el pack — solo
	// para comunitarios, ver comentario de NEW_PREFIX arriba.
	function createOptionsFor(entry: TflSelectionEntry): { value: string; label: string }[] {
		if (entry.source !== 'community') return [];
		return entry.versions.map((v) => ({
			value: `${NEW_PREFIX}${v.mc_version}:${v.loader}`,
			label: `＋ Crear instancia nueva — ${v.mc_version} (${v.loader})`
		}));
	}

	onMount(async () => {
		try {
			entries = await getTflSelection();
			for (const e of entries) {
				const compat = compatibleInstancesFor(e);
				if (compat[0]) {
					selectedInstance[e.id] = compat[0].uuid;
				} else {
					const createOpts = createOptionsFor(e);
					if (createOpts[0]) selectedInstance[e.id] = createOpts[0].value;
				}
			}
		} finally {
			loading = false;
		}
	});

	async function handleInstall(entry: TflSelectionEntry) {
		const selection = selectedInstance[entry.id];
		if (!selection) return;

		installingFor = entry.id;
		errorFor = { ...errorFor, [entry.id]: '' };
		try {
			let instance: InstanceData;

			if (selection.startsWith(NEW_PREFIX)) {
				const [mcVersion, loader] = selection.slice(NEW_PREFIX.length).split(':');
				const variant = entry.versions.find(
					(v) => v.mc_version === mcVersion && v.loader === loader
				);
				if (!variant) throw new Error('No hay un build de este pack para esa versión');
				const baseName =
					entry.versions.length > 1 ? `${entry.title} (${mcVersion})` : entry.title;
				instance = await createInstance(uniqueInstanceName(baseName), mcVersion, loader as Loader);
				onInstanceCreated?.(instance);
			} else {
				const found = instances.find((i) => i.uuid === selection);
				if (!found) return;
				instance = found;
			}

			if (entry.source === 'modrinth') {
				await installModpack(instance.name, entry.project_id!, instance.mc_version, instance.loader);
			} else {
				const variant = entry.versions.find(
					(v) => v.loader === instance.loader && v.mc_version === instance.mc_version
				);
				if (!variant) throw new Error('No hay un build de este pack para esa instancia');
				await installModpackFromUrl(instance.name, entry.id, variant.mrpack_url);
			}
			doneFor = new Set(doneFor).add(entry.id);
		} catch (e) {
			errorFor = { ...errorFor, [entry.id]: String(e) };
		} finally {
			installingFor = null;
		}
	}

	// create_instance rechaza nombres duplicados (InstanceError::AlreadyExists)
	// — si ya existe una instancia con ese nombre (por ejemplo, creaste este
	// mismo pack antes), se le suma un sufijo hasta que quede libre.
	function uniqueInstanceName(base: string): string {
		const taken = new Set(instances.map((i) => i.name));
		if (!taken.has(base)) return base;
		let n = 2;
		while (taken.has(`${base} (${n})`)) n++;
		return `${base} (${n})`;
	}
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
		aria-modal="true"
		tabindex="-1"
	>
		<div class="panel-header">
			<div class="title-row">
				<Tfl width="18" height="18" />
				<h2>TFL Selection</h2>
			</div>
			<button type="button" class="close-btn" onclick={onClose} aria-label="Cerrar">
				<X size={16} />
			</button>
		</div>
		<div class="selection-hero">
			<div class="hero-mark"><Tfl width="28" height="28" /></div>
			<div><span class="eyebrow">Contenido destacado</span><p class="subtitle">Descubrí packs curados y añadilos a una instancia compatible, o creá una nueva al toque.</p></div>
		</div>

		{#if loading}
			<div class="loading-row"><Loader2 size={18} class="spin" /></div>
		{:else if entries.length === 0}
			<p class="empty">Todavía no hay modpacks en la selección.</p>
		{:else}
			<div class="entries">
				{#each entries as entry (entry.id)}
					{@const compat = compatibleInstancesFor(entry)}
					{@const createOpts = createOptionsFor(entry)}
					{@const noOptions = compat.length === 0 && createOpts.length === 0}
					<div class="entry-card anim-fade-in">
						<div class="entry-glow"></div>
						{#if entry.icon_url}
							<img src={entry.icon_url} alt={entry.title} />
						{:else}
							<div class="entry-icon-fallback"></div>
						{/if}
						<div class="entry-info">
							<span class="entry-title">{entry.title}</span>
							<p class="entry-desc">{entry.description}</p>
							{#if entry.source === 'community'}
								<p class="entry-versions">
									{entry.versions.map((v) => `${v.mc_version} (${v.loader})`).join(' · ')}
								</p>
							{/if}
							{#if noOptions}
								<p class="entry-error">
									{entry.source === 'modrinth'
										? 'Necesitás una instancia con Fabric, Forge, NeoForge o Quilt para instalar este pack.'
										: 'Ninguna instancia tuya coincide con las versiones de este pack.'}
								</p>
							{:else if errorFor[entry.id]}
								<p class="entry-error">{errorFor[entry.id]}</p>
							{/if}
						</div>
						<div class="entry-actions">
							<select bind:value={selectedInstance[entry.id]} disabled={noOptions}>
								{#each compat as inst (inst.uuid)}
									<option value={inst.uuid}>{inst.name}</option>
								{/each}
								{#each createOpts as opt (opt.value)}
									<option value={opt.value}>{opt.label}</option>
								{/each}
							</select>
							<button
								type="button"
								class="install-btn"
								disabled={installingFor === entry.id || noOptions}
								onclick={() => handleInstall(entry)}
							>
								<span class="install-label">{doneFor.has(entry.id) ? 'Añadido' : 'Añadir'}</span>
								{#if installingFor === entry.id}
									<Loader2 size={14} class="spin" />
								{:else if doneFor.has(entry.id)}
									<Check size={14} />
								{:else}
									<Download size={14} />
								{/if}
							</button>
						</div>
					</div>
				{/each}
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
		width: min(620px, calc(100vw - 32px));
		max-height: 80vh;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 22px;
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 12px;
		overflow-y: auto;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.title-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.title-row :global(svg) {
		color: var(--accent);
	}

	.panel-header h2 {
		font-size: 1rem;
		font-family: var(--font-brand);
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.subtitle {
		font-size: 0.8rem;
		color: var(--text-muted);
		line-height: 1.45;
	}

	.selection-hero { display: flex; align-items: center; gap: 13px; padding: 13px; border: 1px solid color-mix(in srgb, var(--accent) 27%, var(--border)); border-radius: var(--border-radius); background: linear-gradient(120deg, color-mix(in srgb, var(--accent) 15%, transparent), transparent 62%), var(--bg-input); }
	.hero-mark { display: grid; place-items: center; width: 48px; height: 48px; border-radius: 15px; flex: 0 0 auto; color: var(--accent); background: color-mix(in srgb, var(--accent) 16%, transparent); border: 1px solid color-mix(in srgb, var(--accent) 32%, transparent); }
	.eyebrow { display: block; color: var(--accent); font-size: .64rem; letter-spacing: .13em; font-weight: 800; text-transform: uppercase; margin-bottom: 3px; }

	.loading-row {
		display: flex;
		justify-content: center;
		padding: 24px;
		color: var(--text-muted);
	}

	.empty {
		font-size: 0.8rem;
		color: var(--text-muted);
		padding: 16px 0;
	}

	.entries {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.entry-card {
		position: relative;
		overflow: hidden;
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 14px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: color-mix(in srgb, var(--bg-input) 90%, transparent);
		transition: border-color .16s, transform .16s;
	}
	.entry-card:hover { transform: translateY(-1px); border-color: color-mix(in srgb, var(--accent) 42%, var(--border)); }
	.entry-glow { position: absolute; width: 150px; height: 150px; right: -80px; top: -80px; border-radius: 999px; pointer-events: none; background: radial-gradient(circle, rgba(var(--accent-rgb), .12), transparent 67%); }

	.entry-card img,
	.entry-icon-fallback {
		width: 44px;
		height: 44px;
		border-radius: var(--border-radius-sm);
		flex-shrink: 0;
		background: var(--bg-card);
	}

	.entry-info {
		flex: 1;
		min-width: 0;
	}

	.entry-title {
		font-size: 0.9rem;
		font-weight: 700;
	}

	.entry-desc {
		font-size: 0.74rem;
		color: var(--text-muted);
		margin-top: 2px;
	}

	.entry-versions {
		font-size: 0.68rem;
		color: var(--accent);
		margin-top: 4px;
		font-weight: 700;
	}

	.entry-error {
		font-size: 0.72rem;
		color: var(--color-error);
		margin-top: 4px;
	}

	.entry-actions {
		display: flex;
		flex-direction: column;
		gap: 6px;
		flex-shrink: 0;
	}

	.entry-actions select {
		padding: 6px 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-card);
		color: var(--text-primary);
		font-size: 0.74rem;
		max-width: 180px;
	}

	.install-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 7px 10px;
		font-size: .72rem;
		font-weight: 800;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: var(--accent);
		color: var(--accent-text);
		cursor: pointer;
	}
	.install-label { margin-left: 3px; }

	.install-btn:disabled {
		opacity: 0.7;
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
