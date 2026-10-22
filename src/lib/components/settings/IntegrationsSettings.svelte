<script lang="ts">
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { invoke } from '$lib/api/invoke';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { tr } from '$lib/i18n/experience';
	import { appState } from '$lib/state/state.svelte';
	import {
		ShieldCheck,
		PlugZap,
		Copy,
		RefreshCw,
		Check,
		X,
		Terminal,
		LockKeyhole
	} from 'lucide-svelte';
	type Client = {
		id: string;
		name: string;
		connected: boolean;
		approved: boolean;
		permissions: string[];
		instances: string[];
		expires: number;
		last_activity: number;
	};
	type Plan = {
		id: string;
		client_id: string;
		params: Record<string, string>;
		expires: number;
		approved: boolean;
		decided: boolean;
	};
	type Status = {
		enabled: boolean;
		sidecar_path: string | null;
		clients: Client[];
		plans: Plan[];
		audit: { at: number; client_id: string; action: string; decision: string }[];
	};
	let status = $state<Status | null>(null),
		error = $state(''),
		busy = $state(false),
		copied = $state('');
	let grants = $state<Record<string, string[]>>({}),
		scopes = $state<Record<string, string[]>>({});
	let clientKind = $state('codex');
	const connection = $derived(
		status?.enabled
			? status.clients.some((c) => c.connected && c.approved)
				? tr('Conectado', 'Connected')
				: tr('Esperando aprobación o conexión', 'Waiting for approval or connection')
			: tr('Desactivado', 'Disabled')
	);
	const snippet = $derived.by(() => {
		const command = status?.sidecar_path ?? '';
		if (clientKind === 'codex')
			return `[mcp_servers.tflclient]\ncommand = ${JSON.stringify(command)}\nstartup_timeout_sec = 15\ntool_timeout_sec = 45`;
		if (clientKind === 'opencode')
			return JSON.stringify(
				{ mcp: { tflclient: { type: 'local', command: [command], enabled: true } } },
				null,
				2
			);
		return JSON.stringify({ mcpServers: { tflclient: { command, args: [] } } }, null, 2);
	});
	async function load() {
		try {
			status = await invoke<Status>('mcp_status');
			error = '';
		} catch (e) {
			error = String(e);
		}
	}
	async function action(fn: () => Promise<unknown>) {
		busy = true;
		try {
			await fn();
			await load();
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
	function toggle(client: string, value: string) {
		const current = grants[client] ?? ['read'];
		grants[client] = current.includes(value)
			? current.filter((p) => p !== value)
			: [...current, value];
	}
	function toggleScope(client: string, id: string) {
		const current = scopes[client] ?? [];
		scopes[client] = current.includes(id) ? current.filter((p) => p !== id) : [...current, id];
	}
	async function copy() {
		try {
			await writeText(snippet);
			copied = tr('Copiado', 'Copied');
		} catch (e) {
			error = String(e);
		}
	}
	const permissionLabels: Record<string, [string, string]> = {
		read: ['Consultar estado y diagnósticos', 'Read status & diagnostics'],
		configure: ['Cambiar apariencia y escenas', 'Change appearance & scenes'],
		operate: ['Crear, iniciar e instalar contenido', 'Create, launch & install content'],
		critical: [
			'Solicitar acciones críticas con confirmación local',
			'Request critical actions with local confirmation'
		]
	};
	const actionLabels: Record<string, [string, string]> = {
		delete_instance: ['Borrar instancia y sus archivos', 'Delete instance and its files'],
		stop_instance: ['Detener Minecraft o servidor', 'Stop Minecraft or server'],
		restore_backup: ['Restaurar backup y reemplazar mundos', 'Restore backup and replace worlds'],
		delete_backup: ['Eliminar backup', 'Delete backup'],
		remove_content: ['Eliminar contenido', 'Remove content'],
		rollback_mods: ['Restaurar versiones anteriores de mods', 'Roll back mod versions']
	};
	onMount(() => {
		void load();
		let dispose = () => {};
		void listen('mcp-changed', () => void load()).then((fn) => (dispose = fn));
		return () => dispose();
	});
</script>

<div class="integrations" data-setting="mcp">
	<div class="integration-hero">
		<div class="hero-icon"><PlugZap size={26} /></div>
		<div>
			<h3>TFL Client MCP</h3>
			<p>
				{tr(
					'Tus asistentes, conectados a tus mundos. Tú decides quién entra y qué puede hacer.',
					'Your assistants, connected to your worlds. You choose who connects and what they can do.'
				)}
			</p>
		</div>
		<span class:enabled={status?.enabled} class="status">{connection}</span>
	</div>
	{#if error}<p role="alert" class="error">{error}</p>{/if}
	<section>
		<div class="section-heading">
			<div>
				<h4>{tr('Permitir control mediante IA', 'Allow control through AI')}</h4>
				<p>
					{tr(
						'MCP local por stdio. Requiere el launcher abierto y una aprobación por cada sesión.',
						'Local MCP over stdio. Requires the launcher to be open and approval for each session.'
					)}
				</p>
			</div>
			<button
				class="primary"
				disabled={busy || !status}
				onclick={() => action(() => invoke('mcp_enable', { enabled: !status?.enabled }))}
				>{status?.enabled
					? tr('Desactivar y revocar', 'Disable & revoke')
					: tr('Activar MCP', 'Enable MCP')}</button
			>
		</div>
		<p class="security">
			<LockKeyhole size={15} />{tr(
				'Sin servidor HTTP ni acceso remoto. Tus cuentas y tokens nunca se comparten.',
				'No HTTP server or remote access. Your accounts and tokens are never shared.'
			)}
		</p>
	</section>
	<section>
		<div class="section-heading">
			<h4>{tr('Conectar un asistente', 'Connect an assistant')}</h4>
			<button disabled={busy} onclick={() => action(() => invoke('mcp_probe'))}
				><RefreshCw size={14} />{tr('Probar enlace local', 'Test local connection')}</button
			>
		</div>
		<p>
			{tr(
				'1. Activa MCP. 2. Copia la configuración en tu cliente. 3. Inicia la conexión y aprueba la sesión aquí.',
				'1. Enable MCP. 2. Copy the configuration into your client. 3. Start the connection and approve the session here.'
			)}
		</p>
		<div class="client-tabs">
			{#each [['codex', 'Codex'], ['opencode', 'OpenCode'], ['claude', 'Claude Code']] as [id, label]}<button
					class:selected={clientKind === id}
					onclick={() => (clientKind = id)}>{label}</button
				>{/each}
		</div>
		{#if status?.sidecar_path}<label class="config-label" for="mcp-config"
				>{clientKind === 'codex'
					? '~/.codex/config.toml'
					: clientKind === 'opencode'
						? 'opencode.json'
						: tr(
								'Configuración MCP del proyecto (.mcp.json)',
								'Project MCP configuration (.mcp.json)'
							)}</label
			><textarea id="mcp-config" readonly value={snippet} rows={7} spellcheck="false"
			></textarea><button onclick={copy}
				><Copy size={14} />{copied || tr('Copiar configuración', 'Copy configuration')}</button
			>{:else}<p class="notice">
				{tr(
					'El auxiliar MCP no está instalado junto al launcher. Usa un instalador que incluya esta integración; en desarrollo ejecuta el script de empaquetado del auxiliar.',
					'The MCP helper is not installed beside the launcher. Use an installer that includes this integration; in development run the helper packaging script.'
				)}
			</p>{/if}
		<p class="caption">
			{tr(
				'ChatGPT en la nube no puede acceder a este stdio local. No hay integración remota certificada. Consulta docs/mcp/CLIENTS.md para rutas y comandos por sistema.',
				'Cloud ChatGPT cannot access this local stdio. No remote integration is certified. See docs/mcp/CLIENTS.md for platform paths and commands.'
			)}
		</p>
	</section>
	<section>
		<div class="section-heading">
			<h4><ShieldCheck size={17} />{tr('Sesiones y permisos', 'Sessions & permissions')}</h4>
			<button onclick={load}><RefreshCw size={14} />{tr('Actualizar', 'Refresh')}</button>
		</div>
		<p>
			{tr(
				'Los nombres declarados por clientes no verifican su identidad. Aprueba solo la conexión que acabas de iniciar. Los permisos caducan en una hora.',
				'Reported client names do not verify identity. Only approve the connection you just started. Grants expire after one hour.'
			)}
		</p>
		{#each status?.clients ?? [] as client (client.id)}<article class="client">
				<div class="section-heading">
					<div>
						<strong>{client.name}</strong><code
							>{client.id.slice(0, 8)} · {new Date(
								client.last_activity * 1000
							).toLocaleTimeString()}</code
						>
					</div>
					<span
						>{!client.connected
							? tr('Revocado', 'Revoked')
							: client.approved
								? tr('Autorizado', 'Approved')
								: tr('Espera tu permiso', 'Needs your approval')}</span
					>
				</div>
				{#if client.connected && !client.approved}<div class="permissions">
						{#each Object.entries(permissionLabels) as [id, label]}<label
								><input
									type="checkbox"
									checked={(grants[client.id] ?? ['read']).includes(id)}
									onchange={() => toggle(client.id, id)}
								/>{tr(...label)}</label
							>{/each}
					</div>
					<details>
						<summary
							>{tr(
								'Limitar a instancias concretas (ninguna seleccionada = todas)',
								'Restrict to specific instances (none selected = all)'
							)}</summary
						>{#each appState.instances as instance}<label
								><input
									type="checkbox"
									checked={(scopes[client.id] ?? []).includes(instance.uuid)}
									onchange={() => toggleScope(client.id, instance.uuid)}
								/>{instance.name}</label
							>{/each}
					</details>
					<button
						class="primary"
						disabled={busy}
						onclick={() =>
							action(() =>
								invoke('mcp_grant', {
									clientId: client.id,
									permissions: grants[client.id] ?? ['read'],
									instances: scopes[client.id] ?? []
								})
							)}><Check size={14} />{tr('Autorizar esta sesión', 'Approve this session')}</button
					>{/if}{#if client.connected}<button
						disabled={busy}
						onclick={() => action(() => invoke('mcp_revoke', { clientId: client.id }))}
						><X size={14} />{tr('Revocar', 'Revoke')}</button
					>{/if}
			</article>{:else}<p class="empty">
				{tr(
					'Sin clientes. Conecta un asistente para que aparezca aquí.',
					'No clients. Connect an assistant to see it here.'
				)}
			</p>{/each}
	</section>
	<section>
		<h4>{tr('Acciones que requieren tu confirmación', 'Actions requiring your confirmation')}</h4>
		{#each (status?.plans ?? []).filter((p) => !p.decided && p.expires * 1000 > Date.now()) as plan}<article
				class="approval"
			>
				<h5>
					{tr(...(actionLabels[plan.params.action] ?? ['Acción crítica', 'Critical action']))}
				</h5>
				<p>
					{appState.instances.find((i) => i.uuid === plan.params.instance_id)?.name ??
						plan.params.instance_id}
				</p>
				<code>{plan.params.instance_id} · {plan.params.resource_id ?? ''}</code>
				<p class="caption">
					{tr('Solicitud de', 'Requested by')}
					{plan.client_id.slice(0, 8)} · {tr('Caduca', 'Expires')}
					{new Date(plan.expires * 1000).toLocaleTimeString()}
				</p>
				<button
					disabled={busy}
					onclick={() => action(() => invoke('mcp_decide', { planId: plan.id, approve: false }))}
					>{tr('Rechazar', 'Reject')}</button
				><button
					class="danger"
					disabled={busy}
					onclick={() => action(() => invoke('mcp_decide', { planId: plan.id, approve: true }))}
					>{tr('Aprobar esta acción exacta', 'Approve this exact action')}</button
				>
			</article>{:else}<p class="empty">
				{tr('No hay acciones pendientes.', 'No pending actions.')}
			</p>{/each}
	</section>
	<section>
		<div class="section-heading">
			<h4><Terminal size={17} />{tr('Historial local', 'Local history')}</h4>
			<button onclick={() => action(() => invoke('mcp_clear_audit'))}
				>{tr('Limpiar historial', 'Clear history')}</button
			>
		</div>
		<p class="caption">
			{tr(
				'Hasta 300 decisiones. Sin tokens, logs del juego ni parámetros sensibles.',
				'Up to 300 decisions. No tokens, game logs or sensitive parameters.'
			)}
		</p>
		<div class="audit">
			{#each [...(status?.audit ?? [])].reverse().slice(0, 30) as entry}<div>
					<time>{new Date(entry.at * 1000).toLocaleTimeString()}</time><code>{entry.action}</code
					><span>{entry.decision}</span>
				</div>{/each}
		</div>
	</section>
</div>

<style>
	.integrations {
		display: grid;
		gap: 18px;
		min-width: 0;
	}
	.integration-hero {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 20px;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg, 16px);
		background: linear-gradient(
			125deg,
			color-mix(in srgb, var(--accent) 12%, var(--surface-card)),
			var(--surface-card)
		);
	}
	.hero-icon {
		display: grid;
		place-items: center;
		width: 52px;
		height: 52px;
		flex: none;
		border-radius: 12px;
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		color: var(--accent);
	}
	h3 {
		margin: 0;
		font-size: 20px;
	}
	h4 {
		display: flex;
		gap: 7px;
		align-items: center;
		margin: 0 0 8px;
		font-size: 15px;
	}
	h5 {
		font-size: 15px;
		margin: 0 0 8px;
	}
	p {
		color: var(--text-secondary);
		font-size: 13px;
		line-height: 1.6;
		margin: 6px 0 12px;
	}
	.status {
		margin-left: auto;
		flex: none;
		font-size: 11px;
		color: var(--text-muted);
		max-width: 145px;
	}
	.status.enabled {
		color: var(--accent);
	}
	section {
		padding: 18px;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md, 12px);
		background: var(--surface-card);
	}
	.section-heading {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
	}
	.security {
		display: flex;
		gap: 8px;
		align-items: center;
		margin-bottom: 0;
	}
	.caption {
		font-size: 11px;
	}
	.client-tabs {
		display: flex;
		gap: 6px;
		margin: 12px 0;
	}
	.config-label {
		display: block;
		font-size: 12px;
		color: var(--text-muted);
		margin: 12px 0 6px;
	}
	textarea {
		display: block;
		width: 100%;
		box-sizing: border-box;
		background: var(--surface-input);
		color: var(--text-primary);
		border: 1px solid var(--border-subtle);
		padding: 12px;
		border-radius: 8px;
		resize: vertical;
		font-family: monospace;
		font-size: 12px;
		margin-bottom: 10px;
	}
	button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 9px 13px;
		background: var(--surface-hover);
		color: var(--text-primary);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm, 8px);
		font: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	button:hover,
	button.selected {
		border-color: var(--accent);
	}
	button.primary {
		background: var(--accent);
		color: var(--accent-text);
		border-color: var(--accent);
	}
	button.danger {
		color: #ffaaa4;
		border-color: #b94a44;
		margin-left: 8px;
	}
	button:disabled {
		opacity: 0.5;
		cursor: wait;
	}
	.client,
	.approval {
		margin-top: 14px;
		padding: 15px;
		border: 1px solid var(--border-subtle);
		border-radius: 8px;
	}
	.approval {
		border-color: #b57932;
	}
	.client code {
		display: block;
		margin-top: 5px;
	}
	.permissions {
		display: grid;
		gap: 10px;
		margin: 16px 0;
	}
	label {
		font-size: 12px;
		display: flex;
		gap: 8px;
		align-items: center;
	}
	details {
		margin: 12px 0;
		font-size: 12px;
	}
	details label {
		margin: 8px;
	}
	.audit {
		max-height: 220px;
		overflow: auto;
	}
	.audit > div {
		display: grid;
		grid-template-columns: 70px 1fr 1fr;
		gap: 8px;
		padding: 7px 0;
		border-bottom: 1px solid var(--border-subtle);
		font-size: 11px;
	}
	code {
		font-size: 11px;
		overflow-wrap: anywhere;
	}
	.error {
		color: #ffaaa4;
	}
	.notice {
		padding: 12px;
		background: color-mix(in srgb, var(--accent) 8%, transparent);
		border-left: 2px solid var(--accent);
	}
	.empty {
		color: var(--text-muted);
	}
	:global([data-visual='pixel']) .hero-icon,
	:global([data-visual='pixel']) section,
	:global([data-visual='pixel']) article,
	:global([data-visual='pixel']) textarea,
	:global([data-visual='pixel']) .integration-hero {
		border-radius: 0;
	}
	:global([data-visual='pixel']) button {
		border-radius: 0;
		box-shadow:
			inset 0 2px #ffffff22,
			inset 0 -3px #0005;
	}
	@media (max-width: 900px) {
		.integration-hero {
			flex-wrap: wrap;
		}
		.status {
			margin-left: 0;
			max-width: none;
		}
		.audit > div {
			grid-template-columns: 65px 1fr;
		}
		.audit span {
			grid-column: 2;
		}
	}
</style>
