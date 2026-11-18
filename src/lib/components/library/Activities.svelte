<script lang="ts">
	import { X, Download, Check, RefreshCw, Square, ArrowUpRight } from 'lucide-svelte';
	import { serverTasks, refreshServerTasks } from '$lib/state/serverTasks.svelte';
	import { cancelServerTask, retryServerTask, type ServerTask } from '$lib/api/serverTasks';
	import { appState } from '$lib/state/state.svelte';
	import { selectInstance } from '$lib/state/instanceState.svelte';
	import { modalFocus } from '$lib/visual/modalFocus';
	import { tr } from '$lib/i18n/experience';
	let { onClose }: { onClose: () => void } = $props();
	let error = $state(''),
		busy = $state('');
	const labels = () => ({
		preparing: tr('Resolviendo servidor', 'Resolving server'),
		downloading: tr('Descargando', 'Downloading'),
		ready: tr('Listo', 'Ready'),
		failed: tr('Error de preparación', 'Preparation failed'),
		cancelled: tr('Cancelado', 'Cancelled'),
		interrupted: tr('Interrumpido', 'Interrupted')
	});
	function active(task: ServerTask) {
		return ['preparing', 'downloading'].includes(task.status);
	}
	async function action(task: ServerTask, cancel = false) {
		busy = task.id;
		error = '';
		try {
			await (cancel ? cancelServerTask(task.id) : retryServerTask(task.id));
			await refreshServerTasks();
		} catch (e) {
			error = String(e);
		} finally {
			busy = '';
		}
	}
	function open(task: ServerTask) {
		const i = appState.instances.find((i) => i.name === task.instance_name);
		if (i) {
			selectInstance(i);
			onClose();
		}
	}
</script>

<div
	class="activity-scrim"
	role="presentation"
	onclick={(e) => {
		if (e.target === e.currentTarget) onClose();
	}}
>
	<section
		class="activities"
		role="dialog"
		aria-modal="true"
		aria-label={tr('Actividades', 'Activities')}
		use:modalFocus={onClose}
		tabindex="-1"
	>
		<header>
			<div>
				<h2>{tr('Actividades', 'Activities')}</h2>
				<p>
					{tr(
						'Puedes cerrar este panel mientras se prepara tu servidor.',
						'You can close this panel while your server is being prepared.'
					)}
				</p>
			</div>
			<button class="vr-control" onclick={onClose} aria-label={tr('Cerrar', 'Close')}
				><X size={18} /></button
			>
		</header>
		{#if error || serverTasks.error}<p class="error" role="alert">
				{error || serverTasks.error}
			</p>{/if}
		<div class="task-list">
			{#each serverTasks.items as task (task.id)}
				<article>
					<div class="task-heading">
						{#if task.status === 'ready'}<Check size={20} />{:else}<Download size={20} />{/if}
						<div>
							<strong>{task.instance_name}</strong><small
								>{task.server_type} · {task.mc_version}</small
							>
						</div>
						<span role="status">{labels()[task.status]}</span>
					</div>
					{#if active(task)}{#if task.bytes_total}<progress
								value={task.bytes_current}
								max={task.bytes_total}
								aria-label={tr('Progreso de descarga', 'Download progress')}
							></progress>{:else}<p class="hint">
								{tr('Esperando datos del proveedor…', 'Waiting for provider data…')}
							</p>{/if}<small
							>{(task.bytes_current / 1048576).toFixed(1)} MiB{task.bytes_total
								? ` / ${(task.bytes_total / 1048576).toFixed(1)} MiB`
								: ''}</small
						>{/if}
					{#if task.error}<p class="error">{task.error}</p>{/if}
					<footer>
						{#if active(task)}<button
								class="vr-control"
								disabled={busy === task.id}
								onclick={() => action(task, true)}
								><Square size={14} />{tr('Cancelar preparación', 'Cancel preparation')}</button
							>{:else if task.status !== 'ready'}<button
								class="vr-control"
								disabled={busy === task.id}
								onclick={() => action(task)}
								><RefreshCw size={14} />{tr('Reintentar', 'Retry')}</button
							>{/if}<button class="vr-control" onclick={() => open(task)}
							><ArrowUpRight size={14} />{tr('Ver instancia', 'View instance')}</button
						>
					</footer>
				</article>
			{:else}<p class="empty">
					{tr(
						'No hay preparaciones de servidor. Las nuevas aparecerán aquí.',
						'No server preparations. New ones will appear here.'
					)}
				</p>{/each}
		</div>
	</section>
</div>

<style>
	.activity-scrim {
		position: fixed;
		inset: 0;
		z-index: 90;
		background: var(--bg-overlay);
		display: flex;
		justify-content: flex-end;
	}
	.activities {
		width: min(620px, 100%);
		padding: 24px;
		background: var(--bg-card);
		color: var(--text-primary);
		border-inline-start: 1px solid var(--border);
		overflow-y: auto;
	}
	header,
	.task-heading,
	footer {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	header {
		justify-content: space-between;
		margin-bottom: 24px;
	}
	h2 {
		margin: 0 0 8px;
	}
	p,
	small {
		color: var(--text-secondary);
		font-size: 12px;
		line-height: 1.6;
	}
	.task-list {
		display: grid;
		gap: 16px;
	}
	article {
		padding: 18px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
	}
	.task-heading > div {
		flex: 1;
		min-width: 0;
	}
	strong,
	small {
		display: block;
		overflow-wrap: anywhere;
	}
	.task-heading > span {
		font-size: 12px;
	}
	progress {
		width: 100%;
		margin: 16px 0 6px;
		accent-color: var(--accent);
	}
	footer {
		margin-top: 16px;
		flex-wrap: wrap;
	}
	.error {
		color: var(--color-error);
	}
	.empty {
		padding: 30px 0;
	}
</style>
