<script lang="ts">
	import { downloadTasks } from '$lib/state/downloadState.svelte';
	import { Download, AlertTriangle } from 'lucide-svelte';

	const STAGE_LABEL: Record<string, string> = {
		resolving: 'Resolviendo…',
		library: 'Descargando librerías',
		asset: 'Descargando assets',
		native: 'Descargando nativos',
		client: 'Descargando cliente',
		verifying: 'Verificando',
		extracting: 'Extrayendo',
		processing: 'Procesando',
		jre: 'Descargando Java',
		downloading: 'Descargando',
		generic: 'Trabajando…'
	};

	const tasks = $derived(Object.values(downloadTasks));
</script>

{#if tasks.length > 0}
	<div class="progress-stack">
		{#each tasks as t (t.task)}
			<div class="progress-card" class:failed={!!t.failed}>
				<div class="progress-header">
					{#if t.failed}
						<AlertTriangle size={14} />
					{:else}
						<Download size={14} />
					{/if}
					<span class="progress-title">{t.task}</span>
				</div>
				{#if t.failed}
					<p class="progress-error">{t.failed}</p>
				{:else}
					<p class="progress-sub">
						{STAGE_LABEL[t.stage] ?? t.stage}
						{#if t.currentItem}
							— {t.currentItem}
						{/if}
					</p>
					<div class="progress-track">
						<div
							class="progress-fill"
							class:indeterminate={t.itemTotal === 0}
							style={t.itemTotal > 0
								? `width: ${Math.min(100, (t.itemCurrent / t.itemTotal) * 100)}%`
								: ''}
						></div>
					</div>
					{#if t.itemTotal > 0}
						<p class="progress-count">{t.itemCurrent} / {t.itemTotal}</p>
					{/if}
				{/if}
			</div>
		{/each}
	</div>
{/if}

<style>
	.progress-stack {
		position: fixed;
		right: 16px;
		bottom: 16px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		z-index: 200;
		width: 300px;
	}

	.progress-card {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		padding: 12px 14px;
		box-shadow: var(--shadow-md);
	}

	.progress-card.failed {
		border-color: var(--color-error);
	}

	.progress-header {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--accent);
	}

	.progress-card.failed .progress-header {
		color: var(--color-error);
	}

	.progress-title {
		font-size: 0.78rem;
		font-weight: 700;
		color: var(--text-primary);
	}

	.progress-sub {
		font-size: 0.7rem;
		color: var(--text-secondary);
		margin: 4px 0 8px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.progress-error {
		font-size: 0.72rem;
		color: var(--color-error);
		margin-top: 4px;
	}

	.progress-track {
		height: 5px;
		border-radius: 4px;
		background: rgba(255, 255, 255, 0.08);
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background: var(--accent);
		border-radius: 4px;
		transition: width 0.2s ease;
	}

	.progress-fill.indeterminate {
		width: 40%;
		animation: tfl-indeterminate 1.1s ease-in-out infinite;
	}

	.progress-count {
		font-size: 0.65rem;
		color: var(--text-muted);
		margin-top: 4px;
		text-align: right;
	}

	@keyframes tfl-indeterminate {
		0% {
			transform: translateX(-100%);
		}
		100% {
			transform: translateX(250%);
		}
	}
</style>
