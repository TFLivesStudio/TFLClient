<script lang="ts">
	import { onMount } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import type { InstanceData } from '$lib/types/types';
	import {
		launchInstance,
		deleteInstance,
		renameInstance,
		duplicateInstance,
		exportInstanceAsMrpack,
		verifyInstanceIntegrity,
		updateInstanceMemory,
		getRecommendedRamForInstance,
		openInstanceFolder,
		getInstanceMods,
		getInstanceIconPath,
		launchServer,
		stopServer
	} from '$lib/api/tflApi';
	import { gameSession } from '$lib/state/gameSession.svelte';
	import { serverSessions } from '$lib/state/serverSessions.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import ModsPanel from './ModsPanel.svelte';
	import ShadersPanel from './ShadersPanel.svelte';
	import ModpacksPanel from './ModpacksPanel.svelte';
	import ResourcePacksPanel from './ResourcePacksPanel.svelte';
	import ScreenshotsPanel from './ScreenshotsPanel.svelte';
	import PluginsPanel from './PluginsPanel.svelte';
	import ServerWorlds from './ServerWorlds.svelte';
	import ServerFileManager from './ServerFileManager.svelte';
	import ServerConsole from './ServerConsole.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import InstanceIconPicker from './InstanceIconPicker.svelte';
	import {
		Play,
		Square,
		Loader2,
		Pencil,
		Trash2,
		Check,
		X,
		Package,
		Blocks,
		Clock,
		FolderOpen,
		Puzzle,
		ImagePlus,
		ChevronRight,
		Copy,
		FileOutput,
		ShieldCheck
	} from 'lucide-svelte';

	let {
		instance,
		onChanged
	}: {
		instance: InstanceData;
		onChanged: (instance: InstanceData | null) => void;
	} = $props();

	let launching = $state(false);
	const blockedByOther = $derived(!!gameSession.running && gameSession.running !== instance.name);
	const isServer = $derived(!!instance.server_type);
	const serverRunning = $derived(serverSessions.running.has(instance.name));
	let error = $state<string | null>(null);
	let tab = $state<
		| 'details'
		| 'mods'
		| 'shaders'
		| 'resourcepacks'
		| 'modpacks'
		| 'screenshots'
		| 'plugins'
		| 'worlds'
		| 'files'
		| 'console'
	>('details');
	let showDeleteConfirm = $state(false);
	let editingName = $state(false);
	let nameDraft = $state(instance.name);
	let ramRecommended = $state<number | null>(null);
	let modCount = $state<number | null>(null);
	let iconUrl = $state<string | null>(null);
	let showIconPicker = $state(false);

	$effect(() => {
		nameDraft = instance.name;
	});

	async function loadIcon() {
		const path = await getInstanceIconPath(instance.name);
		// El ícono siempre se guarda como "icon.png" — el path no cambia
		// aunque el CONTENIDO sí (subir uno nuevo, elegir otro preset), así
		// que el WebView lo sirve cacheado y no se ve el cambio. Cache-bust
		// con un timestamp en el query string.
		iconUrl = path ? `${convertFileSrc(path)}?t=${Date.now()}` : null;
	}

	onMount(() => {
		loadRamHint();
		loadModCount();
		loadIcon();
	});

	const SERVER_META: Record<string, { label: string; color: string }> = {
		vanilla: { label: 'Vanilla', color: 'var(--loader-vanilla)' },
		paper: { label: 'Paper', color: '#3b82f6' },
		purpur: { label: 'Purpur', color: '#8b5cf6' }
	};
	const serverMeta = $derived(instance.server_type ? SERVER_META[instance.server_type] : null);

	const LOADER_META: Record<string, { label: string; color: string }> = {
		vanilla: { label: 'Vanilla', color: 'var(--loader-vanilla)' },
		fabric: { label: 'Fabric', color: 'var(--loader-fabric)' },
		forge: { label: 'Forge', color: 'var(--loader-forge)' },
		neoforge: { label: 'NeoForge', color: 'var(--loader-neoforge)' },
		quilt: { label: 'Quilt', color: 'var(--loader-quilt)' }
	};
	const loaderMeta = $derived(serverMeta ?? LOADER_META[instance.loader]);

	const lastPlayedLabel = $derived.by(() => {
		if (!instance.last_played) return t('instanceDetail.neverPlayed');
		const diffMs = Date.now() - instance.last_played * 1000;
		const days = Math.floor(diffMs / 86_400_000);
		if (days <= 0) return t('instanceDetail.playedToday');
		if (days === 1) return t('instanceDetail.playedYesterday');
		return t('instanceDetail.playedDaysAgo', { days });
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

	let serverToggling = $state(false);
	async function handleServerToggle() {
		serverToggling = true;
		error = null;
		try {
			if (serverRunning) await stopServer(instance.name);
			else await launchServer(instance.name);
		} catch (e) {
			error = String(e);
		} finally {
			serverToggling = false;
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
			const r = await getRecommendedRamForInstance(instance.name);
			ramRecommended = r.recommended_max_mb;
		} catch {
			// silencioso — solo es una sugerencia
		}
	}

	let duplicating = $state(false);
	async function handleDuplicate() {
		duplicating = true;
		error = null;
		try {
			const copy = await duplicateInstance(instance.name);
			onChanged(copy);
		} catch (e) {
			error = String(e);
		} finally {
			duplicating = false;
		}
	}

	let exporting = $state(false);
	let exportedPath = $state<string | null>(null);
	async function handleExport() {
		exporting = true;
		error = null;
		exportedPath = null;
		try {
			exportedPath = await exportInstanceAsMrpack(instance.name);
		} catch (e) {
			error = String(e);
		} finally {
			exporting = false;
		}
	}

	let verifying = $state(false);
	let integrityIssues = $state<string[] | null>(null);
	async function handleVerifyIntegrity() {
		verifying = true;
		error = null;
		integrityIssues = null;
		try {
			integrityIssues = await verifyInstanceIntegrity(instance.name);
		} catch (e) {
			error = String(e);
		} finally {
			verifying = false;
		}
	}

	async function loadModCount() {
		if (instance.loader === 'vanilla') return;
		try {
			modCount = (await getInstanceMods(instance.name)).length;
		} catch {
			modCount = 0;
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

	async function handleOpenFolder() {
		try {
			await openInstanceFolder(instance.name);
		} catch (e) {
			error = String(e);
		}
	}
</script>

<div class="instance-detail">
	<div class="hero">
		<button
			type="button"
			class="hero-icon"
			style="--loader-color: {loaderMeta.color}"
			onclick={() => (showIconPicker = true)}
			aria-label={t('instanceDetail.changeIcon')}
		>
			{#if iconUrl}
				<img src={iconUrl} alt="" class="hero-icon-img" />
			{:else}
				{instance.name.charAt(0).toUpperCase()}
			{/if}
			<span class="hero-icon-edit">
				<ImagePlus size={14} />
			</span>
		</button>

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
					<button type="button" class="icon-btn" onclick={confirmRename} aria-label={t('settings.ram.save')}>
						<Check size={15} />
					</button>
					<button
						type="button"
						class="icon-btn"
						onclick={() => {
							editingName = false;
							nameDraft = instance.name;
						}}
						aria-label={t('common.cancel')}
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
						aria-label={t('instanceDetail.rename')}
					>
						<Pencil size={13} />
					</button>
					<button
						type="button"
						class="icon-btn"
						disabled={duplicating}
						onclick={handleDuplicate}
						aria-label={t('instanceDetail.duplicate')}
						title={t('instanceDetail.duplicateTitle')}
					>
						{#if duplicating}<Loader2 size={13} class="spin" />{:else}<Copy size={13} />{/if}
					</button>
					<button
						type="button"
						class="icon-btn"
						disabled={exporting}
						onclick={handleExport}
						aria-label={t('instanceDetail.exportMrpack')}
						title={t('instanceDetail.exportMrpack')}
					>
						{#if exporting}<Loader2 size={13} class="spin" />{:else}<FileOutput size={13} />{/if}
					</button>
					<button
						type="button"
						class="icon-btn"
						disabled={verifying}
						onclick={handleVerifyIntegrity}
						aria-label={t('instanceDetail.verifyIntegrity')}
						title={t('instanceDetail.verifyIntegrity')}
					>
						{#if verifying}<Loader2 size={13} class="spin" />{:else}<ShieldCheck size={13} />{/if}
					</button>
					<button
						type="button"
						class="icon-btn danger"
						onclick={() => (showDeleteConfirm = true)}
						aria-label={t('instanceDetail.delete')}
					>
						<Trash2 size={13} />
					</button>
				</div>
				{#if exportedPath}
					<p class="hint">{t('instanceDetail.exported', { path: exportedPath })}</p>
				{/if}
				{#if integrityIssues !== null}
					{#if integrityIssues.length === 0}
						<p class="hint">{t('instanceDetail.integrityOk')}</p>
					{:else}
						<p class="error">
							{t('instanceDetail.integrityMissing', {
								count: integrityIssues.length,
								plural: integrityIssues.length === 1 ? '' : 's',
								list: integrityIssues.join(', ')
							})}
						</p>
					{/if}
				{/if}
			{/if}

			<div class="stat-chips">
				<span class="loader-badge" style="--loader-color: {loaderMeta.color}"
					>{loaderMeta.label}</span
				>
				<span class="stat-chip"><Package size={12} /> {instance.mc_version}</span>
				{#if instance.loader !== 'vanilla'}
					<span class="stat-chip"><Blocks size={12} /> {instance.loader_version}</span>
				{/if}
				<span class="stat-chip"><Clock size={12} /> {lastPlayedLabel}</span>
			</div>
		</div>

		{#if isServer}
			<button
				type="button"
				class="play-btn"
				class:stop-btn={serverRunning}
				disabled={serverToggling}
				onclick={handleServerToggle}
			>
				{#if serverToggling}
					<Loader2 size={16} class="spin" />
					{serverRunning ? t('serverInstance.stopping') : t('serverInstance.starting')}
				{:else if serverRunning}
					<Square size={16} fill="currentColor" />
					{t('serverInstance.stop')}
				{:else}
					<Play size={16} fill="currentColor" />
					{t('serverInstance.start')}
				{/if}
			</button>
		{:else}
			<button type="button" class="play-btn" disabled={launching || blockedByOther} onclick={handlePlay}>
				{#if launching}
					<Loader2 size={16} class="spin" />
					{t('instanceDetail.preparing')}
				{:else if blockedByOther}
					<Play size={16} fill="currentColor" />
					{t('instanceDetail.instanceRunning', { name: gameSession.running ?? '' })}
				{:else}
					<Play size={16} fill="currentColor" />
					{t('instanceDetail.play')}
				{/if}
			</button>
		{/if}
	</div>

	{#if error}
		<p class="error">{error}</p>
	{/if}

	<div class="quick-actions">
		{#if isServer}
			<button type="button" class="quick-card" onclick={() => (tab = 'plugins')}>
				<div class="quick-icon"><Puzzle size={16} /></div>
				<div class="quick-text">
					<span class="quick-title">{t('serverInstance.tabPlugins')}</span>
					<span class="quick-sub">
						{instance.server_type === 'vanilla'
							? t('pluginsPanel.vanillaHint')
							: instance.server_build}
					</span>
				</div>
				<ChevronRight size={14} class="quick-arrow" />
			</button>
		{:else}
			<button type="button" class="quick-card" onclick={() => (tab = 'mods')}>
				<div class="quick-icon"><Puzzle size={16} /></div>
				<div class="quick-text">
					<span class="quick-title">{t('instanceDetail.tabs.mods')}</span>
					<span class="quick-sub">
						{instance.loader === 'vanilla'
							? t('instanceDetail.modsNotSupportedVanilla')
							: modCount === null
								? t('instanceDetail.loading')
								: t('instanceDetail.modsInstalledCount', { count: modCount })}
					</span>
				</div>
				<ChevronRight size={14} class="quick-arrow" />
			</button>
		{/if}

		<button type="button" class="quick-card" onclick={handleOpenFolder}>
			<div class="quick-icon"><FolderOpen size={16} /></div>
			<div class="quick-text">
				<span class="quick-title">{t('instanceDetail.folder')}</span>
				<span class="quick-sub">{t('instanceDetail.instanceFiles')}</span>
			</div>
			<ChevronRight size={14} class="quick-arrow" />
		</button>
	</div>

	<div class="tabs">
		<button
			type="button"
			class="tab-btn"
			class:active={tab === 'details'}
			onclick={() => (tab = 'details')}
		>
			{t('instanceDetail.tabs.details')}
		</button>
		{#if isServer}
			<button
				type="button"
				class="tab-btn"
				class:active={tab === 'plugins'}
				onclick={() => (tab = 'plugins')}
			>
				{t('serverInstance.tabPlugins')}
			</button>
			<button
				type="button"
				class="tab-btn"
				class:active={tab === 'worlds'}
				onclick={() => (tab = 'worlds')}
			>
				{t('serverInstance.tabWorlds')}
			</button>
			<button
				type="button"
				class="tab-btn"
				class:active={tab === 'files'}
				onclick={() => (tab = 'files')}
			>
				{t('serverInstance.tabFiles')}
			</button>
			<button
				type="button"
				class="tab-btn"
				class:active={tab === 'console'}
				onclick={() => (tab = 'console')}
			>
				{t('serverInstance.tabConsole')}
			</button>
		{:else}
			<button
				type="button"
				class="tab-btn"
				class:active={tab === 'mods'}
				onclick={() => (tab = 'mods')}
			>
				{t('instanceDetail.tabs.mods')}
			</button>
			<button
				type="button"
				class="tab-btn"
				class:active={tab === 'shaders'}
				onclick={() => (tab = 'shaders')}
			>
				{t('instanceDetail.tabs.shaders')}
			</button>
			<button
				type="button"
				class="tab-btn"
				class:active={tab === 'resourcepacks'}
				onclick={() => (tab = 'resourcepacks')}
			>
				{t('instanceDetail.tabs.resourcePacks')}
			</button>
			<button
				type="button"
				class="tab-btn"
				class:active={tab === 'modpacks'}
				onclick={() => (tab = 'modpacks')}
			>
				{t('instanceDetail.tabs.modpacks')}
			</button>
			<button
				type="button"
				class="tab-btn"
				class:active={tab === 'screenshots'}
				onclick={() => (tab = 'screenshots')}
			>
				{t('instanceDetail.tabs.screenshots')}
			</button>
		{/if}
	</div>

	{#if tab === 'details'}
		<section class="ram-section">
			<span class="section-label">{t('instanceDetail.ram.sectionLabel')}</span>
			<p class="hint">
				{t('instanceDetail.ram.hint', {
					recommended: ramRecommended
						? t('instanceDetail.ram.recommendedSuffix', { mb: ramRecommended })
						: ''
				})}
			</p>
			<div class="ram-row">
				<label>
					{t('settings.ram.min')}
					<input
						type="number"
						value={instance.min_memory ?? ''}
						placeholder={t('instanceDetail.ram.globalPlaceholder')}
						min="512"
						step="256"
						onchange={(e) => {
							const v = e.currentTarget.valueAsNumber;
							setMemory(Number.isNaN(v) ? null : v, instance.max_memory);
						}}
					/>
				</label>
				<label>
					{t('settings.ram.max')}
					<input
						type="number"
						value={instance.max_memory ?? ''}
						placeholder={t('instanceDetail.ram.globalPlaceholder')}
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
	{:else if tab === 'mods'}
		<ModsPanel {instance} />
	{:else if tab === 'shaders'}
		<ShadersPanel {instance} />
	{:else if tab === 'resourcepacks'}
		<ResourcePacksPanel {instance} />
	{:else if tab === 'modpacks'}
		<ModpacksPanel {instance} />
	{:else if tab === 'screenshots'}
		<ScreenshotsPanel {instance} />
	{:else if tab === 'plugins'}
		<PluginsPanel {instance} />
	{:else if tab === 'worlds'}
		<ServerWorlds {instance} />
	{:else if tab === 'files'}
		<ServerFileManager {instance} />
	{:else if tab === 'console'}
		<ServerConsole {instance} />
	{/if}
</div>

{#if showDeleteConfirm}
	<ConfirmDialog
		title={t('instanceDetail.deleteTitle')}
		message={t('instanceDetail.deleteMessage', { name: instance.name })}
		confirmLabel={t('instanceDetail.delete')}
		danger
		onConfirm={handleDelete}
		onCancel={() => (showDeleteConfirm = false)}
	/>
{/if}

{#if showIconPicker}
	<InstanceIconPicker
		instanceName={instance.name}
		onClose={() => (showIconPicker = false)}
		onChanged={loadIcon}
	/>
{/if}

<style>
	.instance-detail {
		padding: 32px;
		display: flex;
		flex-direction: column;
		gap: 18px;
		height: 100%;
		overflow-y: auto;
	}

	.hero {
		position: relative;
		display: flex;
		align-items: center;
		gap: 20px;
		padding: 22px 24px;
		border-radius: var(--border-radius-lg);
		border: 1px solid var(--border);
		background:
			radial-gradient(
				ellipse 500px 220px at 0% 0%,
				color-mix(in srgb, var(--loader-color) 14%, transparent),
				transparent 70%
			),
			var(--bg-card);
		overflow: hidden;
		/* .instance-detail es flex-column + overflow-y:auto — sin esto,
		   cuando el contenido de abajo (ej. la lista de mods) crece,
		   flexbox intenta ACHICAR este bloque antes de scrollear (el
		   shrink por defecto corre antes que el overflow), y la cabecera
		   se veía cada vez más comprimida a medida que se instalaban más
		   mods. flex-shrink:0 la deja con altura fija siempre, el scroll
		   se encarga del resto. */
		flex-shrink: 0;
	}

	.hero-icon {
		position: relative;
		flex-shrink: 0;
		width: 72px;
		height: 72px;
		padding: 0;
		border-radius: var(--border-radius-lg);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.8rem;
		font-weight: 800;
		font-family: inherit;
		color: var(--loader-color);
		background: color-mix(in srgb, var(--loader-color) 20%, transparent);
		border: 1px solid color-mix(in srgb, var(--loader-color) 45%, transparent);
		box-shadow: 0 0 0 4px color-mix(in srgb, var(--loader-color) 8%, transparent);
		cursor: pointer;
		overflow: hidden;
	}

	.hero-icon:disabled {
		cursor: not-allowed;
	}

	.hero-icon-img {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.hero-icon-edit {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.55);
		color: #ffffff;
		opacity: 0;
		transition: opacity 0.12s;
	}

	.hero-icon:hover .hero-icon-edit,
	.hero-icon:focus-visible .hero-icon-edit {
		opacity: 1;
	}

	.hero-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.name-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.name-row h2 {
		font-size: var(--text-2xl);
	}

	.name-edit {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.name-edit input {
		font-size: var(--text-xl);
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

	.stat-chips {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 6px;
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

	.stat-chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 0.74rem;
		color: var(--text-secondary);
		background: var(--bg-card);
		border: 1px solid var(--border);
		padding: 3px 9px;
		border-radius: 999px;
	}

	.play-btn {
		flex-shrink: 0;
		display: inline-flex;
		align-items: center;
		gap: 9px;
		background: linear-gradient(155deg, var(--accent-hover), var(--accent));
		color: var(--accent-text);
		border: none;
		padding: 15px 32px;
		border-radius: var(--border-radius);
		font-weight: 800;
		font-size: 0.9rem;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		cursor: pointer;
		box-shadow:
			var(--shadow-md),
			0 0 24px color-mix(in srgb, var(--accent) 35%, transparent);
		transition:
			transform 0.12s ease,
			box-shadow 0.12s ease;
	}

	.play-btn.stop-btn {
		background: linear-gradient(155deg, var(--color-error), color-mix(in srgb, var(--color-error) 80%, black));
		box-shadow:
			var(--shadow-md),
			0 0 24px color-mix(in srgb, var(--color-error) 35%, transparent);
	}

	.play-btn:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow:
			var(--shadow-lg),
			0 0 32px color-mix(in srgb, var(--accent) 45%, transparent);
	}

	.play-btn:disabled {
		opacity: 0.7;
		cursor: not-allowed;
	}

	.error {
		color: var(--color-error);
		font-size: 0.82rem;
	}

	.quick-actions {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 10px;
		flex-shrink: 0;
	}

	.quick-card {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px 14px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-card);
		color: var(--text-primary);
		cursor: pointer;
		text-align: left;
		transition:
			border-color 0.15s,
			transform 0.12s;
	}

	.quick-card:hover {
		border-color: var(--accent);
		transform: translateY(-1px);
	}

	.quick-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
		color: var(--accent);
	}

	.quick-text {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.quick-title {
		font-size: 0.82rem;
		font-weight: 700;
	}

	.quick-sub {
		font-size: 0.7rem;
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.quick-card :global(.quick-arrow) {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.tabs {
		display: flex;
		gap: 4px;
		border-bottom: 1px solid var(--border);
		overflow-x: auto;
		scrollbar-width: none;
		/* .instance-detail es el único contenedor con scroll (ver nota en
		   .hero) — sin esto, scrollear una lista larga en Descargar se
		   lleva puesta la barra de pestañas, y para volver a Detalles hay
		   que scrollear todo hasta arriba de nuevo. */
		position: sticky;
		top: 0;
		z-index: 2;
		background: var(--bg-main);
	}

	.tabs::-webkit-scrollbar {
		display: none;
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
		white-space: nowrap;
		flex-shrink: 0;
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
