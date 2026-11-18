<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { InstanceData } from '$lib/types/types';
	import {
		listInstanceWorlds,
		renameInstanceWorld,
		deleteInstanceWorld,
		backupInstanceWorld,
		listInstanceWorldBackups,
		restoreInstanceWorld,
		listWorldDatapacks,
		installWorldDatapack,
		changeWorldDatapack,
		searchWorldDatapacks,
		downloadWorldDatapack,
		type ManagedWorld,
		type ManagedWorldBackup,
		type WorldDatapack,
		type DatapackHit
	} from '$lib/api/worlds';
	import { pickContentFiles } from '$lib/api/system';
	import { i18nState } from '$lib/i18n/index.svelte';
	import { tr } from '$lib/i18n/experience';
	import { gameSession } from '$lib/state/gameSession.svelte';
	import { serverSessions } from '$lib/state/serverSessions.svelte';
	import { experience } from '$lib/state/experience.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import {
		Globe,
		Archive,
		ArrowLeft,
		Pencil,
		Trash2,
		RefreshCw,
		Upload,
		Search,
		Download,
		Check,
		Loader2
	} from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();
	let worlds = $state<ManagedWorld[]>([]);
	let selected = $state<ManagedWorld | null>(null);
	let backups = $state<ManagedWorldBackup[]>([]);
	let datapacks = $state<WorldDatapack[]>([]);
	let loading = $state(true);
	let busy = $state(false);
	let error = $state<string | null>(null);
	let message = $state<string | null>(null);
	let rename = $state(false);
	let nameDraft = $state('');
	let query = $state('');
	let results = $state<DatapackHit[]>([]);
	let searched = $state(false);
	let searching = $state(false);
	let confirmation = $state<{
		title: string;
		message: string;
		label: string;
		action: () => Promise<void>;
	} | null>(null);
	let generation = 0;
	let alive = true;
	const running = $derived(
		gameSession.running.some((r) => r.name === instance.name) ||
			serverSessions.running.has(instance.name)
	);
	onDestroy(() => {
		alive = false;
		generation += 1;
	});

	async function refresh() {
		const request = ++generation;
		loading = true;
		error = null;
		try {
			const list = await listInstanceWorlds(instance.name);
			if (!alive || request !== generation) return;
			worlds = list;
			if (selected) selected = list.find((world) => world.name === selected?.name) ?? null;
		} catch (e) {
			if (alive && request === generation) error = String(e);
		} finally {
			if (alive && request === generation) loading = false;
		}
	}
	$effect(() => {
		void instance.name;
		selected = null;
		void refresh();
	});

	async function details(world: ManagedWorld) {
		selected = world;
		rename = false;
		query = '';
		results = [];
		searched = false;
		backups = [];
		datapacks = [];
		error = null;
		const request = ++generation;
		loading = true;
		const response = await Promise.allSettled([
			listInstanceWorldBackups(instance.name, world.name),
			listWorldDatapacks(instance.name, world.name)
		]);
		if (!alive || request !== generation) return;
		if (response[0].status === 'fulfilled') backups = response[0].value;
		if (response[1].status === 'fulfilled') datapacks = response[1].value;
		const failures = response.filter((item) => item.status === 'rejected');
		if (failures.length)
			error = failures.map((item) => String((item as PromiseRejectedResult).reason)).join(' · ');
		loading = false;
	}

	async function perform(action: () => Promise<unknown>, success: string) {
		if (busy) return;
		busy = true;
		error = null;
		message = null;
		try {
			await action();
			if (!alive) return;
			message = success;
			await refresh();
			if (selected) await details(selected);
		} catch (e) {
			if (alive) error = String(e);
		} finally {
			if (alive) busy = false;
		}
	}
	function ask(title: string, text: string, label: string, action: () => Promise<void>) {
		confirmation = { title, message: text, label, action };
	}
	async function addLocal(world: ManagedWorld) {
		try {
			const files = await pickContentFiles('zip', 'Datapack ZIP');
			if (!files.length || !alive) return;
			await perform(
				async () => {
					for (const file of files) await installWorldDatapack(instance.name, world.name, file);
				},
				tr(
					'Datapack instalado. Comprueba la compatibilidad antes de abrir el mundo.',
					'Datapack installed. Check compatibility before opening the world.'
				)
			);
		} catch (e) {
			error = String(e);
		}
	}
	async function search(world: ManagedWorld) {
		if (searching) return;
		const worldId = world.name;
		searching = true;
		error = null;
		try {
			const hits = await searchWorldDatapacks(instance.name, worldId, query);
			if (!alive || selected?.name !== worldId) return;
			results = hits;
			searched = true;
		} catch (e) {
			if (alive) error = String(e);
		} finally {
			if (alive) searching = false;
		}
	}
	function size(bytes: number | null): string {
		if (bytes === null) return tr('Tamaño no disponible', 'Size unavailable');
		if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GiB`;
		return `${(bytes / 1024 ** 2).toFixed(1)} MiB`;
	}
</script>

<section
	class="worlds-panel"
	aria-label={tr('Gestor de mundos', 'World manager')}
	aria-busy={busy || loading}
>
	<header>
		<div class="heading">
			{#if selected}<button
					type="button"
					class="icon-btn"
					aria-label={tr('Volver a mundos', 'Back to worlds')}
					onclick={() => {
						selected = null;
						void refresh();
					}}><ArrowLeft size={17} /></button
				>{:else}<Globe size={20} />{/if}
			<div>
				<h3>{selected?.display_name ?? tr('Mundos', 'Worlds')}</h3>
				<p>
					{selected
						? `${tr('Carpeta', 'Folder')}: ${selected.name}`
						: tr(
								'Tus mundos y sus datapacks, en un solo lugar.',
								'Your worlds and their datapacks, together.'
							)}
				</p>
			</div>
		</div>
		<button
			type="button"
			class="icon-btn"
			disabled={busy || loading}
			aria-label={tr('Actualizar mundos', 'Refresh worlds')}
			onclick={() => (selected ? details(selected) : refresh())}><RefreshCw size={16} /></button
		>
	</header>
	{#if running}<p class="notice" role="status">
			{tr(
				'La instancia está en ejecución. Detén el juego o servidor para guardar, respaldar o modificar sus mundos.',
				'This instance is running. Stop the game or server before backing up or changing its worlds.'
			)}
		</p>{/if}
	{#if message}<p class="success" role="status"><Check size={14} />{message}</p>{/if}
	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#if loading}<p class="hint">
			<Loader2 size={14} class="spin" />{tr('Leyendo los mundos…', 'Reading worlds…')}
		</p>
	{:else if selected}
		{@const world = selected}
		<div class="world-summary">
			<span>{world.version ?? tr('Versión no disponible', 'Version unavailable')}</span><span
				>{size(world.size_bytes)}</span
			>{#if world.server_primary}<span
					>{tr('Mundo principal del servidor', 'Primary server world')}</span
				>{/if}
		</div>
		{#if world.missing}<p class="notice">
				{tr(
					'Este mundo fue eliminado. Elige un respaldo para recuperarlo en su carpeta original.',
					'This world was deleted. Choose a backup to recover it in its original folder.'
				)}
			</p>{/if}
		{#if world.metadata_error}<p class="notice">
				{world.metadata_error}
				{tr(
					'Los archivos se conservan. Puedes crear un respaldo antes de repararlos.',
					'Files are preserved. Back them up before repairing them.'
				)}
			</p>{/if}
		{#if experience.mode === 'advanced' && world.dimensions.length}<p class="hint">
				{tr('Dimensiones', 'Dimensions')}: {world.dimensions.join(', ')} · DataVersion: {world.data_version ??
					'—'}
			</p>{/if}
		{#if !world.missing}<div class="actions">
				<button
					type="button"
					disabled={busy || running || !!world.metadata_error}
					onclick={() => {
						rename = !rename;
						nameDraft = world.display_name;
					}}><Pencil size={14} />{tr('Renombrar mundo', 'Rename world')}</button
				>
				<button
					type="button"
					disabled={busy || running}
					onclick={() =>
						perform(
							() => backupInstanceWorld(instance.name, world.name),
							tr('Respaldo creado y verificado.', 'Backup created and verified.')
						)}><Archive size={14} />{tr('Respaldar mundo', 'Back up world')}</button
				>
				<button
					type="button"
					class="danger"
					disabled={busy || running}
					onclick={() =>
						ask(
							tr('Eliminar mundo', 'Delete world'),
							tr(
								`Se eliminará «${world.display_name}» de esta instancia. Primero se creará un respaldo verificado; si falla, no se eliminará.`,
								`“${world.display_name}” will be deleted from this instance. A verified backup is created first; deletion stops if it fails.`
							),
							tr('Eliminar mundo', 'Delete world'),
							() =>
								perform(
									() => deleteInstanceWorld(instance.name, world.name),
									tr(
										'Mundo eliminado. Su respaldo se conserva.',
										'World deleted. Its backup is preserved.'
									)
								)
						)}><Trash2 size={14} />{tr('Eliminar', 'Delete')}</button
				>
			</div>
			{#if rename}<form
					class="rename"
					onsubmit={(e) => {
						e.preventDefault();
						void perform(
							() => renameInstanceWorld(instance.name, world.name, nameDraft),
							tr(
								'Nombre del mundo actualizado. La carpeta conserva su nombre.',
								'World name updated. The folder keeps its name.'
							)
						);
						rename = false;
					}}
				>
					<label for="world-name">{tr('Nombre visible del mundo', 'Visible world name')}</label>
					<div class="actions">
						<input
							id="world-name"
							bind:value={nameDraft}
							maxlength="128"
							required
							disabled={busy || running}
						/><button type="submit" disabled={busy || running || !nameDraft.trim()}
							>{tr('Renombrar', 'Rename')}</button
						><button type="button" onclick={() => (rename = false)}
							>{tr('Cancelar', 'Cancel')}</button
						>
					</div>
				</form>{/if}
			<section class="subsection" aria-label={tr('Datapacks del mundo', 'World datapacks')}>
				<div class="subhead">
					<h4>Datapacks</h4>
					<button type="button" disabled={busy || running} onclick={() => addLocal(world)}
						><Upload size={14} />{tr('Añadir ZIP', 'Add ZIP')}</button
					>
				</div>
				<p class="hint">
					{tr(
						'Se aplican al abrir este mundo. La versión del mundo y el formato del paquete deben ser compatibles.',
						'They apply when this world opens. The world version and pack format must be compatible.'
					)}
				</p>
				{#if !datapacks.length}<p class="empty compact">
						{tr('Este mundo aún no tiene datapacks.', 'This world has no datapacks yet.')}
					</p>{/if}
				{#each datapacks as pack (`${pack.enabled}:${pack.id}`)}
					<div class="pack-row">
						<div class="item-text">
							<strong>{pack.id}</strong><span
								>{pack.enabled ? tr('Habilitado', 'Enabled') : tr('Deshabilitado', 'Disabled')} · {tr(
									'Formato',
									'Format'
								)}
								{pack.pack_format ?? (pack.min_format ? JSON.stringify(pack.min_format) : '—')} · {pack.compatibility ===
								'incompatible'
									? tr('Incompatible con esta versión', 'Incompatible with this version')
									: pack.compatibility === 'compatible'
										? tr('Formato compatible', 'Compatible format')
										: tr('Compatibilidad sin verificar', 'Compatibility unverified')}</span
							>{#if pack.error}<span class="error">{pack.error}</span>{/if}
						</div>
						<button
							type="button"
							disabled={busy || running}
							onclick={() =>
								perform(
									() =>
										changeWorldDatapack(instance.name, world.name, pack.id, pack.enabled, false),
									tr('Estado del datapack actualizado.', 'Datapack state updated.')
								)}
							>{pack.enabled ? tr('Deshabilitar', 'Disable') : tr('Habilitar', 'Enable')}</button
						><button
							type="button"
							class="icon-btn danger"
							disabled={busy || running}
							aria-label={`${tr('Eliminar datapack', 'Delete datapack')} ${pack.id}`}
							onclick={() =>
								ask(
									tr('Eliminar datapack', 'Delete datapack'),
									`${tr('Se eliminará este paquete del mundo', 'This pack will be removed from the world')}: ${pack.id}`,
									tr('Eliminar', 'Delete'),
									() =>
										perform(
											() =>
												changeWorldDatapack(instance.name, world.name, pack.id, pack.enabled, true),
											tr('Datapack eliminado.', 'Datapack deleted.')
										)
								)}><Trash2 size={14} /></button
						>
					</div>
				{/each}
				<form
					class="search-form"
					onsubmit={(e) => {
						e.preventDefault();
						void search(world);
					}}
				>
					<label for="datapack-query"
						>{tr('Descargar desde Modrinth', 'Download from Modrinth')}</label
					>
					<div class="actions">
						<input
							id="datapack-query"
							bind:value={query}
							maxlength="200"
							placeholder={tr('Buscar datapacks para este mundo', 'Find datapacks for this world')}
							disabled={searching}
						/><button type="submit" disabled={searching || !world.version}
							><Search size={14} />{tr('Buscar', 'Search')}</button
						>
					</div>
				</form>
				{#if searching}<p class="hint">
						{tr('Buscando…', 'Searching…')}
					</p>{:else if searched && !results.length}<p class="hint">
						{tr(
							'El proveedor no declara resultados compatibles con esta versión del mundo.',
							'The provider has no results declaring compatibility with this world version.'
						)}
					</p>{/if}
				{#each results as hit (hit.project_id)}<div class="pack-row">
						<div class="item-text">
							<strong>{hit.title}</strong><span>{hit.description}</span><span
								>{tr('Autor declarado', 'Declared author')}: {hit.author} · Minecraft {world.version}</span
							>
						</div>
						<button
							type="button"
							disabled={busy || running}
							onclick={() =>
								ask(
									tr('Instalar datapack', 'Install datapack'),
									`${hit.title} · ${tr('El proveedor declara compatibilidad con', 'The provider declares compatibility with')} Minecraft ${world.version}.`,
									tr('Descargar e instalar', 'Download and install'),
									() =>
										perform(
											() => downloadWorldDatapack(instance.name, world.name, hit.project_id),
											tr(
												'Datapack descargado, verificado e instalado.',
												'Datapack downloaded, verified and installed.'
											)
										)
								)}><Download size={14} />{tr('Instalar', 'Install')}</button
						>
					</div>{/each}
			</section>
		{/if}
		<section class="subsection" aria-label={tr('Respaldos de este mundo', 'Backups of this world')}>
			<h4>{tr('Respaldos de este mundo', 'Backups of this world')}</h4>
			<p class="hint">
				{tr(
					'La restauración verifica el hash y el ZIP completo, y conserva una copia del estado actual.',
					'Restore verifies the hash and full ZIP, and preserves a copy of the current state.'
				)}
			</p>
			{#if !backups.length}<p class="empty compact">
					{tr('Aún no hay respaldos de este mundo.', 'No backups of this world yet.')}
				</p>{/if}
			{#each backups as backup (backup.id)}<div class="pack-row">
					<div class="item-text">
						<strong>{new Date(backup.created_at * 1000).toLocaleString(i18nState.locale)}</strong
						><span>{size(backup.size_bytes)}</span>{#if experience.mode === 'advanced'}<code
								title={backup.sha256}>SHA-256 {backup.sha256.slice(0, 16)}…</code
							>{/if}
					</div>
					<button
						type="button"
						disabled={busy || running}
						onclick={() =>
							ask(
								tr('Restaurar mundo', 'Restore world'),
								tr(
									'Se reemplazará este mundo por el respaldo elegido. Antes se guardará el estado actual.',
									'This world will be replaced by the selected backup. The current state is backed up first.'
								),
								tr('Restaurar', 'Restore'),
								() =>
									perform(
										() => restoreInstanceWorld(instance.name, world.name, backup.id),
										tr('Mundo restaurado.', 'World restored.')
									)
							)}>{tr('Restaurar', 'Restore')}</button
					>
				</div>{/each}
		</section>
	{:else if !error && !worlds.length}<div class="empty">
			<Globe size={28} />
			<h4>{tr('Aún no hay mundos', 'No worlds yet')}</h4>
			<p>
				{instance.server_type
					? tr(
							'Los mundos aparecerán después de iniciar el servidor y generar su mundo.',
							'Worlds appear after the server starts and generates its world.'
						)
					: tr(
							'Crea un mundo desde Minecraft. Al cerrar el juego, aparecerá aquí.',
							'Create a world in Minecraft. It will appear here after the game closes.'
						)}
			</p>
		</div>
	{:else}<div class="worlds-list">
			{#each worlds as world (world.name)}<button
					type="button"
					class="world-row"
					onclick={() => details(world)}
					><Globe size={20} />
					<div class="item-text">
						<strong>{world.display_name}</strong><span
							>{world.name} · {world.version ?? tr('Versión no disponible', 'Version unavailable')} ·
							{size(world.size_bytes)}</span
						>{#if world.metadata_error}<span class="error"
								>{tr(
									'No se pudieron leer todos los metadatos. Los archivos se conservan.',
									'Some metadata could not be read. Files are preserved.'
								)}</span
							>{/if}
					</div>
					{#if world.missing}<span class="badge">{tr('Recuperar', 'Recover')}</span
						>{:else if world.server_primary}<span class="badge">{tr('Principal', 'Primary')}</span
						>{/if}</button
				>{/each}
		</div>{/if}
</section>

{#if confirmation}<ConfirmDialog
		title={confirmation.title}
		message={confirmation.message}
		confirmLabel={confirmation.label}
		danger
		onConfirm={() => {
			const action = confirmation?.action;
			confirmation = null;
			void action?.();
		}}
		onCancel={() => (confirmation = null)}
	/>{/if}

<style>
	.worlds-panel {
		display: flex;
		flex-direction: column;
		gap: 16px;
		min-width: 0;
	}
	header,
	.heading,
	.actions,
	.subhead,
	.pack-row,
	.world-row,
	.world-summary {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	header,
	.subhead {
		justify-content: space-between;
	}
	.heading {
		min-width: 0;
	}
	h3,
	h4,
	p {
		margin: 0;
	}
	h3 {
		font-size: 1rem;
		overflow-wrap: anywhere;
	}
	h4 {
		font-size: 0.9rem;
	}
	p,
	.hint,
	.item-text span,
	.world-summary {
		color: var(--text-secondary);
		font-size: 0.78rem;
		line-height: 1.55;
	}
	.hint,
	.success {
		display: flex;
		align-items: center;
		gap: 7px;
	}
	.heading p {
		overflow-wrap: anywhere;
	}
	.world-summary,
	.actions {
		flex-wrap: wrap;
	}
	.world-summary span + span {
		border-inline-start: 1px solid var(--border);
		padding-inline-start: 10px;
	}
	.worlds-list {
		display: grid;
		gap: 8px;
	}
	button,
	input {
		font: inherit;
		color: var(--text-primary);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
	}
	button {
		min-height: 32px;
		padding: 7px 10px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 7px;
		cursor: pointer;
		font-size: 0.77rem;
	}
	button:hover:enabled {
		background: var(--bg-hover);
		border-color: var(--accent);
	}
	button:focus-visible,
	input:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 3px;
	}
	button:disabled {
		opacity: 0.5;
		cursor: default;
	}
	button.icon-btn {
		padding: 7px;
		flex: 0 0 auto;
	}
	button.danger {
		color: var(--color-error);
	}
	.world-row {
		width: 100%;
		text-align: start;
		justify-content: flex-start;
		padding: 15px;
		background: var(--bg-card);
	}
	.item-text {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
		gap: 3px;
		overflow-wrap: anywhere;
	}
	.item-text strong {
		font-size: 0.84rem;
		font-weight: 600;
	}
	.item-text code {
		font-size: 0.68rem;
		color: var(--text-muted);
	}
	.badge {
		font-size: 0.7rem;
		padding: 3px 7px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		color: var(--accent);
	}
	.empty {
		padding: 34px 20px;
		display: flex;
		align-items: center;
		flex-direction: column;
		text-align: center;
		gap: 10px;
		color: var(--text-muted);
	}
	.empty.compact {
		padding: 15px;
		font-size: 0.78rem;
	}
	.notice {
		border-inline-start: 3px solid var(--accent);
		padding: 10px 12px;
		background: var(--bg-input);
	}
	.error,
	.item-text span.error {
		color: var(--color-error);
		font-size: 0.78rem;
	}
	.success {
		color: var(--color-success, var(--text-primary));
		font-size: 0.8rem;
	}
	.subsection {
		display: flex;
		flex-direction: column;
		gap: 10px;
		border-top: 1px solid var(--border);
		padding-top: 18px;
	}
	.pack-row {
		padding: 10px 0;
		border-bottom: 1px solid var(--border);
		align-items: flex-start;
	}
	form {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	label {
		font-size: 0.78rem;
		color: var(--text-secondary);
	}
	input {
		min-width: 140px;
		flex: 1;
		padding: 9px 10px;
		font-size: 0.8rem;
	}
	.spin {
		animation: world-spin 0.8s linear infinite;
	}
	@keyframes world-spin {
		to {
			transform: rotate(360deg);
		}
	}
	@media (max-width: 650px) {
		.pack-row {
			flex-wrap: wrap;
		}
		.pack-row .item-text {
			flex-basis: 100%;
		}
		.world-row {
			flex-wrap: wrap;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.spin {
			animation: none;
		}
	}
</style>
