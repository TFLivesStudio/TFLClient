<script lang="ts">
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { Copy, RefreshCw } from 'lucide-svelte';
	import { tr } from '$lib/i18n/experience';
	import {
		getMcpTargets,
		writeMcpTarget,
		MCP_CHECK_PROMPT,
		type McpClient,
		type McpTarget
	} from '$lib/api/mcpQuickConnect';

	let {
		enabled,
		busy,
		helperAvailable,
		onEnable
	}: {
		enabled: boolean;
		busy: boolean;
		helperAvailable: boolean;
		onEnable: () => Promise<void>;
	} = $props();
	const clients: { id: McpClient; name: string }[] = [
		{ id: 'codex', name: 'Codex (app/CLI)' },
		{ id: 'opencode', name: 'OpenCode (app/CLI)' },
		{ id: 'claude', name: 'Claude Code' }
	];
	let targets = $state<McpTarget[]>([]);
	let selected = $state<Partial<Record<McpClient, string>>>({});
	let loading = $state(true);
	let writing = $state(false);
	let error = $state('');
	let feedback = $state('');
	let copied = $state(false);
	let disposed = false;
	function forClient(id: McpClient) {
		return targets.filter((target) => target.client === id);
	}
	function destination(id: McpClient) {
		const choices = forClient(id);
		return (
			choices.find((choice) => choice.id === selected[id]) ??
			(choices.length === 1 ? choices[0] : undefined)
		);
	}
	async function load() {
		loading = true;
		try {
			const next = await getMcpTargets();
			if (!disposed) {
				targets = next;
				error = '';
			}
		} catch (e) {
			if (!disposed) error = String(e);
		} finally {
			if (!disposed) loading = false;
		}
	}
	async function configure(target: McpTarget, next: boolean) {
		writing = true;
		feedback = '';
		error = '';
		try {
			targets = await writeMcpTarget(target, next);
			feedback = next
				? tr(
						'Configurado. Reinicia o recarga tu asistente y autoriza su sesión aquí.',
						'Configured. Restart or reload your assistant, then approve its session here.'
					)
				: tr(
						'Se retiró únicamente la entrada administrada por TFL. Recarga tu asistente.',
						'Only the TFL-managed entry was removed. Reload your assistant.'
					);
		} catch (e) {
			error = String(e);
		} finally {
			writing = false;
		}
	}
	async function copyPrompt() {
		try {
			await writeText(tr(MCP_CHECK_PROMPT.es, MCP_CHECK_PROMPT.en));
			copied = true;
		} catch (e) {
			error = String(e);
		}
	}
	onMount(() => {
		void load();
		let unlisten: (() => void) | undefined;
		void listen('mcp-quick-connect-changed', () => void load()).then((fn) => {
			if (disposed) fn();
			else unlisten = fn;
		});
		return () => {
			disposed = true;
			unlisten?.();
		};
	});
</script>

<section class="quick-connect" aria-label={tr('Conexión de asistentes', 'Assistant connections')}>
	<div class="toggle-row">
		<div class="row-copy">
			<label for="mcp-local-toggle">{tr('Habilitar MCP local', 'Enable local MCP')}</label>
			<p>
				{enabled
					? tr(
							'Esperando conexiones con tu autorización.',
							'Waiting for connections with your approval.'
						)
					: tr('Desactivado', 'Disabled')}
			</p>
		</div>
		<input
			id="mcp-local-toggle"
			class="switch"
			type="checkbox"
			role="switch"
			checked={enabled}
			disabled={busy || writing}
			onchange={(event) => {
				event.currentTarget.checked = enabled;
				void onEnable();
			}}
		/>
	</div>
	{#each clients as client (client.id)}
		{@const choices = forClient(client.id)}
		{@const target = destination(client.id)}
		<div class="client-row">
			<div class="toggle-row">
				<div class="row-copy">
					<label for={`mcp-connect-${client.id}`}>{tr('Conectar', 'Connect')} {client.name}</label>
					<p>
						{loading
							? tr('Buscando instalación…', 'Looking for installation…')
							: target?.configured
								? target.managed
									? tr(
											'Configurado · la sesión requiere autorización',
											'Configured · session approval required'
										)
									: tr(
											'Configuración ajena: revisar manualmente',
											'External configuration: review manually'
										)
								: target?.installed
									? tr('Disponible para configurar', 'Available to configure')
									: choices.length > 1
										? tr('Selecciona un destino', 'Select a destination')
										: tr('Cliente no detectado', 'Client not detected')}
					</p>
				</div>
				<input
					id={`mcp-connect-${client.id}`}
					class="switch"
					type="checkbox"
					role="switch"
					checked={target?.configured ?? false}
					disabled={loading ||
						writing ||
						busy ||
						(!helperAvailable && !target?.managed) ||
						(!target?.installed && !target?.managed) ||
						!target.supported ||
						(target.configured && !target.managed)}
					onchange={(event) => {
						const next = event.currentTarget.checked;
						event.currentTarget.checked = target?.configured ?? false;
						if (target) void configure(target, next);
					}}
				/>
			</div>
			{#if choices.length > 1}
				<label class="destination-label" for={`mcp-destination-${client.id}`}
					>{tr('Destino', 'Destination')}</label
				>
				<select
					id={`mcp-destination-${client.id}`}
					value={selected[client.id] ?? ''}
					disabled={writing || busy}
					onchange={(event) => {
						selected[client.id] = event.currentTarget.value;
					}}
				>
					<option value="">{tr('Seleccionar perfil o formato', 'Select profile or format')}</option>
					{#each choices as choice (choice.id)}<option value={choice.id}>{choice.label}</option
						>{/each}
				</select>
			{/if}
			{#if target}
				<p class="destination"><span>{tr('Archivo:', 'File:')}</span> <code>{target.path}</code></p>
				{#if target.reason}<p class="reason">{target.reason}</p>{/if}
			{/if}
		</div>
	{/each}
	{#if !helperAvailable}<p class="reason">
			{tr(
				'El auxiliar MCP no está disponible. Usa un instalador que lo incluya.',
				'The MCP helper is unavailable. Use an installer that includes it.'
			)}
		</p>{/if}
	{#if error}<p role="alert" class="error">{error}</p>
		<button onclick={load} disabled={loading || writing}
			><RefreshCw size={14} />{tr('Actualizar destinos', 'Refresh destinations')}</button
		>{/if}
	{#if feedback}<p role="status">{feedback}</p>{/if}
	<div class="check-prompt">
		<div>
			<strong
				>{tr(
					'Comprueba la conexión desde tu agente',
					'Verify the connection from your agent'
				)}</strong
			>
			<p>
				{tr(
					'Copia este prompt. Solo solicita una consulta de lectura y nunca afirma conexión sin comprobarla.',
					'Copy this prompt. It requests a read-only query and never claims a connection without checking it.'
				)}
			</p>
		</div>
		<button onclick={copyPrompt}
			><Copy size={14} />{copied
				? tr('Copiado', 'Copied')
				: tr('Copiar prompt', 'Copy prompt')}</button
		>
	</div>
</section>

<style>
	.quick-connect {
		display: grid;
		gap: 0;
		padding: 0 18px;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md, 12px);
		background: var(--surface-card);
	}
	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 18px;
		min-height: 72px;
		padding: 16px 0;
	}
	.client-row,
	.check-prompt {
		border-top: 1px solid var(--border-subtle);
	}
	.row-copy {
		min-width: 0;
	}
	.row-copy label {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		cursor: pointer;
	}
	p {
		margin: 5px 0 0;
		font-size: 12px;
		line-height: 1.5;
		color: var(--text-secondary);
	}
	.switch {
		flex: none;
		appearance: none;
		width: 38px;
		height: 22px;
		border: 1px solid var(--border-strong, var(--border-subtle));
		border-radius: 20px;
		background: var(--bg-input);
		cursor: pointer;
		position: relative;
		transition: background-color 140ms ease;
	}
	.switch::after {
		content: '';
		position: absolute;
		top: 3px;
		left: 3px;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--text-secondary);
		transition: transform 140ms ease;
	}
	.switch:checked {
		background: var(--accent);
		border-color: var(--accent);
	}
	.switch:checked::after {
		background: var(--accent-text);
		transform: translateX(16px);
	}
	.switch:disabled {
		opacity: 0.5;
		cursor: default;
	}
	.switch:focus-visible,
	button:focus-visible,
	select:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 3px;
	}
	.destination {
		margin: -2px 0 12px;
		overflow-wrap: anywhere;
		font-size: 11px;
	}
	.destination span,
	.destination-label {
		color: var(--text-muted);
	}
	code {
		font: inherit;
	}
	.reason {
		margin-bottom: 14px;
	}
	.error {
		color: var(--error, #ee9c94);
		padding-top: 12px;
	}
	.check-prompt {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 18px;
		padding: 18px 0;
	}
	strong {
		font-size: 13px;
	}
	button,
	select {
		color: var(--text-primary);
		border: 1px solid var(--border-subtle);
		background: var(--bg-input);
		font: inherit;
		font-size: 12px;
		border-radius: var(--radius-sm, 8px);
		padding: 9px 12px;
	}
	button {
		display: inline-flex;
		flex-shrink: 0;
		align-items: center;
		gap: 6px;
		cursor: pointer;
	}
	button:hover {
		border-color: var(--accent);
	}
	select {
		max-width: 100%;
		margin-bottom: 12px;
	}
	.destination-label {
		display: block;
		font-size: 11px;
		margin-bottom: 5px;
	}
	:global([data-visual='pixel']) .quick-connect,
	:global([data-visual='pixel']) button,
	:global([data-visual='pixel']) select,
	:global([data-visual='pixel']) .switch,
	:global([data-visual='pixel']) .switch::after {
		border-radius: 0;
	}
	@media (max-width: 900px) {
		.check-prompt {
			align-items: flex-start;
			flex-direction: column;
			gap: 12px;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.switch,
		.switch::after {
			transition: none;
		}
	}
</style>
