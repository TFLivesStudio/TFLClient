<script lang="ts">
	import { onDestroy } from 'svelte';
	import {
		inspectInstanceSource,
		importInstanceSource,
		cancelInstanceImport,
		type ImportPreview
	} from '$lib/api/importSources';
	import type { InstanceData } from '$lib/types/types';
	import { tr } from '$lib/i18n/experience';
	import { experience } from '$lib/state/experience.svelte';
	import {
		ArrowLeft,
		Download,
		Loader2,
		FileCheck,
		AlertTriangle,
		Server,
		Gamepad2
	} from 'lucide-svelte';
	let {
		path,
		onImported,
		onBack,
		onBusyChange
	}: {
		path: string;
		onImported: (instance: InstanceData) => void;
		onBack: () => void;
		onBusyChange: (busy: boolean) => void;
	} = $props();
	let preview = $state<ImportPreview | null>(null);
	let loading = $state(true);
	let importing = $state(false);
	let cancelling = $state(false);
	let name = $state('');
	let error = $state<string | null>(null);
	let requestId: string | null = null;
	let generation = 0;
	let alive = true;
	const labels: Record<string, string> = {
		'multimc-v1': 'MultiMC / Prism / PolyMC / Freesm',
		'curseforge-installed': 'CurseForge',
		'modrinth-sqlite': 'Modrinth App',
		'tfl-server-v1': 'TFL Server ZIP'
	};
	onDestroy(() => {
		alive = false;
		generation += 1;
		if (importing && requestId) void cancelInstanceImport(requestId);
	});
	$effect(() => {
		const source = path;
		const request = ++generation;
		loading = true;
		preview = null;
		error = null;
		void inspectInstanceSource(source)
			.then((result) => {
				if (!alive || request !== generation) return;
				preview = result;
				name = result.candidate.name;
			})
			.catch((reason) => {
				if (alive && request === generation) error = String(reason);
			})
			.finally(() => {
				if (alive && request === generation) loading = false;
			});
	});
	async function start() {
		if (!preview?.importable || !name.trim() || importing) return;
		importing = true;
		cancelling = false;
		error = null;
		requestId = crypto.randomUUID();
		onBusyChange(true);
		try {
			const instance = await importInstanceSource(
				path,
				name.trim(),
				preview.fingerprint,
				requestId
			);
			if (alive) onImported(instance);
		} catch (reason) {
			if (alive) error = String(reason);
		} finally {
			if (alive) {
				importing = false;
				cancelling = false;
				onBusyChange(false);
			}
			requestId = null;
		}
	}
	async function cancel() {
		if (!requestId || cancelling) return;
		cancelling = true;
		try {
			await cancelInstanceImport(requestId);
		} catch (reason) {
			error = String(reason);
			cancelling = false;
		}
	}
	function size(bytes: number): string {
		return bytes >= 1024 ** 3
			? `${(bytes / 1024 ** 3).toFixed(2)} GiB`
			: `${(bytes / 1024 ** 2).toFixed(1)} MiB`;
	}
</script>

<section
	class="review"
	aria-label={tr('Previsualizar importación', 'Import preview')}
	aria-busy={loading || importing}
>
	<h2>{tr('Revisar antes de importar', 'Review before importing')}</h2>
	{#if loading}<p class="status">
			<Loader2 size={17} class="spin" />{tr('Inspeccionando el origen…', 'Inspecting source…')}
		</p>
	{:else if preview}
		<div class="identity">
			{#if preview.server_type}<Server size={24} />{:else}<Gamepad2 size={24} />{/if}
			<div>
				<strong>{preview.candidate.name}</strong>
				<p>
					{labels[preview.format] ?? preview.format} · {preview.server_type ??
						preview.candidate.loader}
					{preview.candidate.loader_version ?? ''} · Minecraft {preview.candidate.mc_version ?? '—'}
				</p>
			</div>
		</div>
		<p class="summary">
			{preview.file_count}
			{tr('archivos', 'files')} · {size(preview.size_bytes)}
		</p>
		<div class="included">
			<FileCheck size={16} />
			<p>
				{tr('Se importará', 'Will import')}: {preview.included.length
					? preview.included.join(', ')
					: tr('No hay contenido compatible', 'No compatible content')}
			</p>
		</div>
		<p class="hint">
			{tr(
				'El origen se conserva. No se importan cuentas, tokens ni comandos de inicio del otro launcher.',
				'The source is preserved. Accounts, tokens and launcher start commands are not imported.'
			)}
		</p>
		{#if preview.server_type}<p class="notice">
				{tr(
					'Se asignará un puerto libre. El servidor no se inicia ni acepta la EULA automáticamente. Se conserva la política de autenticación declarada.',
					'An available port will be assigned. The server will not start or accept the EULA automatically. Its declared authentication policy is preserved.'
				)}
			</p>{/if}
		{#if preview.warnings.length}<div class="warnings" role="alert">
				<AlertTriangle size={17} />
				<div>
					{#each preview.warnings as warning}<p>{warning}</p>{/each}
				</div>
			</div>{/if}
		<details open={experience.mode === 'advanced'}>
			<summary
				>{tr(
					'Contenido omitido y detalles del formato',
					'Omitted content and format details'
				)}</summary
			>
			<ul>
				{#each preview.omitted as omitted}<li>{omitted}</li>{/each}
			</ul>
			{#if experience.mode === 'advanced'}<p class="path">{path}</p>
				<code>{preview.format}</code>{/if}
		</details>
		{#if preview.importable}<label
				>{tr('Nombre de la nueva instancia', 'New instance name')}<input
					bind:value={name}
					maxlength="60"
					disabled={importing}
				/></label
			>{/if}
	{/if}
	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#if importing}<p class="status" role="status">
			<Loader2 size={15} class="spin" />{cancelling
				? tr(
						'Cancelando y retirando la copia incompleta…',
						'Cancelling and removing the incomplete copy…'
					)
				: tr(
						'Copiando y verificando. No cierres el launcher.',
						'Copying and verifying. Keep the launcher open.'
					)}
		</p>{/if}
	<div class="actions">
		{#if importing}<button type="button" disabled={cancelling} onclick={cancel}
				>{tr('Cancelar importación', 'Cancel import')}</button
			>{:else}<button type="button" onclick={onBack}
				><ArrowLeft size={15} />{tr('Volver', 'Back')}</button
			><button
				type="button"
				class="primary"
				disabled={loading || !preview?.importable || !name.trim()}
				onclick={start}><Download size={15} />{tr('Importar instancia', 'Import instance')}</button
			>{/if}
	</div>
</section>

<style>
	.review {
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}
	h2,
	p {
		margin: 0;
	}
	h2 {
		font-size: 1.1rem;
	}
	.identity {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.identity strong {
		font-size: 0.95rem;
		overflow-wrap: anywhere;
	}
	p,
	li,
	summary {
		font-size: 0.78rem;
		line-height: 1.6;
		color: var(--text-secondary);
	}
	.included,
	.warnings,
	.status {
		display: flex;
		align-items: flex-start;
		gap: 9px;
	}
	.included :global(svg),
	.warnings :global(svg) {
		flex: 0 0 auto;
		margin-top: 3px;
	}
	.included {
		background: var(--bg-input);
		padding: 12px;
		border-radius: var(--border-radius);
	}
	.notice {
		border-inline-start: 3px solid var(--accent);
		padding-inline-start: 11px;
	}
	.warnings,
	.error {
		color: var(--color-error);
	}
	.warnings p {
		color: inherit;
	}
	details {
		padding: 11px 0;
		border-block: 1px solid var(--border);
	}
	summary {
		cursor: pointer;
	}
	ul {
		padding-inline-start: 20px;
	}
	.path,
	li {
		overflow-wrap: anywhere;
	}
	code {
		font-size: 0.7rem;
		color: var(--text-muted);
	}
	label {
		font-size: 0.8rem;
		display: flex;
		flex-direction: column;
		gap: 7px;
	}
	input {
		padding: 10px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		color: var(--text-primary);
		font: inherit;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 5px;
	}
	button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		color: var(--text-primary);
		padding: 9px 13px;
		cursor: pointer;
		font: inherit;
		font-size: 0.8rem;
	}
	button.primary {
		background: var(--accent);
		color: var(--accent-contrast, #fff);
		border-color: var(--accent);
	}
	button:disabled {
		opacity: 0.5;
		cursor: default;
	}
	button:focus-visible,
	input:focus-visible,
	summary:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 3px;
	}
	.review :global(.spin) {
		animation: source-spin 0.8s linear infinite;
	}
	@keyframes source-spin {
		to {
			transform: rotate(360deg);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.review :global(.spin) {
			animation: none;
		}
	}
</style>
