<script lang="ts">
	import type { InstanceData } from '$lib/types/types';
	import { instanceDisplayName, releaseLabel } from '$lib/selection-identity';
	import { tr } from '$lib/i18n/experience';
	import {
		checkSelectionUpdate,
		verifySelectionIntegrity,
		repairSelectionInstance,
		installSelectionTarget,
		type SelectionUpdateInfo
	} from '$lib/api/tflSelection';
	import { ShieldCheck, RefreshCw, Wrench, Loader2 } from 'lucide-svelte';
	let {
		instance,
		running = false,
		onUpdated
	}: {
		instance: InstanceData;
		running?: boolean;
		onUpdated: (instance: InstanceData) => void;
	} = $props();
	let busy = $state(''),
		message = $state(''),
		error = $state(''),
		update = $state<SelectionUpdateInfo | null>(null);
	async function run(kind: string, action: () => Promise<void>) {
		if (busy) return;
		busy = kind;
		error = '';
		message = '';
		try {
			await action();
		} catch (e) {
			error = String(e);
		} finally {
			busy = '';
		}
	}
</script>

{#if instance.selection}
	<section class="managed-client" aria-label={tr('Cliente oficial', 'Official client')}>
		<header>
			<ShieldCheck size={20} />
			<div>
				<span>TFL Selection</span>
				<h3>{instanceDisplayName(instance)}</h3>
				<p>
					{tr('Cliente oficial mantenido por TFLives', 'Official client maintained by TFLives')}
				</p>
			</div>
		</header>
		<dl>
			<div>
				<dt>Minecraft</dt>
				<dd>{instance.mc_version}</dd>
			</div>
			<div>
				<dt>{tr('Versión instalada', 'Installed release')}</dt>
				<dd>
					{releaseLabel(instance.selection.release, instance.selection.channel)}<small
						>{instance.selection.release}</small
					>
				</dd>
			</div>
			<div>
				<dt>{tr('Estado', 'Status')}</dt>
				<dd>{running ? tr('En ejecución', 'Running') : tr('Instalado', 'Installed')}</dd>
			</div>
		</dl>
		<div class="actions">
			<button
				data-pixel-control
				disabled={!!busy}
				onclick={() =>
					run('update', async () => {
						update = await checkSelectionUpdate(instance.name);
						if (!update) message = tr('Tu cliente está al día.', 'Your client is up to date.');
					})}><RefreshCw size={15} />{tr('Buscar actualizaciones', 'Check for updates')}</button
			>
			<button
				data-pixel-control
				disabled={!!busy || running}
				onclick={() =>
					run('verify', async () => {
						const result = await verifySelectionIntegrity(instance.name);
						message = result.is_valid
							? tr('Integridad verificada.', 'Integrity verified.')
							: tr('El cliente necesita reparación.', 'The client needs repair.');
					})}><ShieldCheck size={15} />{tr('Verificar', 'Verify')}</button
			>
			<button
				data-pixel-control
				disabled={!!busy || running}
				onclick={() =>
					run('repair', async () => {
						await repairSelectionInstance(instance.name);
						message = tr(
							'Cliente reparado. Tus mundos y opciones se conservan.',
							'Client repaired. Your worlds and settings are preserved.'
						);
					})}><Wrench size={15} />{tr('Reparar', 'Repair')}</button
			>
		</div>
		{#if update}<div class="update">
				<span
					>{tr('Disponible', 'Available')}: {releaseLabel(
						update.latest_release,
						update.channel
					)}</span
				><button
					data-pixel-control="primary"
					disabled={!!busy || running}
					onclick={() =>
						run('install', async () => {
							onUpdated(
								await installSelectionTarget(instance.name, update!.target_id, update!.channel)
							);
							update = null;
							message = tr('Cliente actualizado.', 'Client updated.');
						})}>{tr('Actualizar cliente', 'Update client')}</button
				>
			</div>{/if}
		<p>
			{tr(
				'Personaliza tus opciones, mundos, capturas y paquetes de recursos. Los shaders requieren compatibilidad con el cliente. La composición del cliente se actualiza y repara como un conjunto.',
				'Personalize your settings, worlds, screenshots and resource packs. Shaders require client compatibility. The complete client is updated and repaired as one product.'
			)}
		</p>
		<a
			href={`https://github.com/TFLivesStudio/TFLClientSelection/releases/tag/selection-${encodeURIComponent(instance.selection.target_id)}-v${encodeURIComponent(instance.selection.release)}`}
			target="_blank"
			rel="noreferrer"
			>{tr('Código, licencias y documentación', 'Source, licenses and documentation')} ↗</a
		>
		{#if busy}<p role="status">
				<Loader2 size={14} class="spin" />
				{tr('Procesando…', 'Working…')}
			</p>{/if}{#if message}<p role="status">{message}</p>{/if}{#if error}<p
				role="alert"
				class="error"
			>
				{error}
			</p>{/if}
	</section>
{/if}

<style>
	.managed-client {
		display: grid;
		gap: 20px;
		padding: 24px;
		background: var(--surface-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		margin-bottom: 24px;
	}
	header {
		display: flex;
		gap: 12px;
		align-items: center;
	}
	header > :global(svg),
	a {
		color: var(--accent);
	}
	header span {
		color: var(--accent);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.12em;
	}
	h3 {
		margin: 4px 0;
		font-size: 19px;
	}
	p {
		margin: 0;
		color: var(--text-secondary);
		font-size: 12px;
		line-height: 1.65;
	}
	dl {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 16px;
		margin: 0;
	}
	dt {
		font-size: 11px;
		color: var(--text-secondary);
		margin-bottom: 8px;
	}
	dd {
		margin: 0;
		font-size: 13px;
		overflow-wrap: anywhere;
	}
	small {
		display: block;
		margin-top: 6px;
		font-family: 'JetBrains Mono', monospace;
		font-size: 10px;
		color: var(--text-secondary);
	}
	.actions,
	.update {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		align-items: center;
	}
	button {
		display: inline-flex;
		gap: 8px;
		align-items: center;
		padding: 10px 12px;
		background: var(--surface-interactive);
		border: 1px solid var(--border);
		color: var(--text-primary);
		border-radius: var(--border-radius-sm);
		font-size: 12px;
		cursor: pointer;
	}
	button:disabled {
		opacity: 0.5;
		cursor: default;
	}
	a,
	.update {
		font-size: 12px;
	}
	.error {
		color: var(--danger, #ee6d6d);
		overflow-wrap: anywhere;
	}
	@media (max-width: 850px) {
		.managed-client {
			padding: 16px;
		}
		dl {
			grid-template-columns: 1fr;
		}
	}
</style>
