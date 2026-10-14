<script lang="ts">
	import { Surface, ErrorState, Skeleton } from '$lib/components/ui';
	import { t } from '$lib/i18n/index.svelte';
	import { Cpu, Coffee, MemoryStick, HardDrive } from 'lucide-svelte';
	import type { SystemStatus } from '$lib/types/types';

	/** Panel "Estado del sistema" de la Home — Entrega C. Markup/estilos
	 * extraídos de WelcomeState.svelte (líneas 453-488 originales).
	 *
	 * El original solo tenía dos estados implícitos (cargado / `null` sin
	 * ningún mensaje — un panel mudo si `getSystemStatus()` fallaba, sección
	 * 6 del encargo: "sin datos de sistema" tiene que tener un estado
	 * explícito). Acá se agregan los otros dos que ya hacían falta:
	 * cargando (antes de la primera respuesta, con `Skeleton`) y error (con
	 * `ErrorState` + reintentar, que vuelve a llamar la MISMA
	 * `getSystemStatus()` del orquestador — nada de lógica nueva). */
	let {
		status,
		loading,
		onRetry
	}: {
		status: SystemStatus | null;
		loading: boolean;
		onRetry: () => void;
	} = $props();

	function formatGb(gb: number): string {
		return `${gb.toFixed(gb < 10 ? 1 : 0)} GB`;
	}
</script>

<Surface level="card" as="section">
	<div class="panel">
		<div class="panel-title">
			<Cpu size={13} />
			<span>{t('home.systemStatus')}</span>
		</div>
		{#if loading}
			<div class="status-list">
				<Skeleton variant="text" width="70%" />
				<Skeleton variant="text" width="85%" />
				<Skeleton variant="text" width="60%" />
			</div>
		{:else if status}
			<ul class="status-list">
				<li>
					<Coffee size={13} />
					<span>
						{status.java_major
							? t('home.javaInstalled', { major: status.java_major })
							: t('home.javaMissing')}
					</span>
				</li>
				<li>
					<MemoryStick size={13} />
					<span>
						{t('home.ramStatus', {
							assigned: (status.ram_assigned_mb / 1024).toFixed(1),
							total: (status.ram_total_mb / 1024).toFixed(1)
						})}
					</span>
				</li>
				<li>
					<HardDrive size={13} />
					<span>
						{t('home.diskStatus', {
							free: formatGb(status.disk_free_gb),
							total: formatGb(status.disk_total_gb)
						})}
					</span>
				</li>
			</ul>
		{:else}
			<ErrorState
				title={t('home.systemStatusUnavailable')}
				retryLabel={t('home.systemStatusRetry')}
				{onRetry}
			>
				{#snippet icon()}<Cpu size={22} />{/snippet}
			</ErrorState>
		{/if}
	</div>
</Surface>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px;
		min-width: 0;
	}

	.panel-title {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		font-weight: 700;
		letter-spacing: 0.4px;
		text-transform: uppercase;
		color: var(--text-muted);
	}
	.panel-title :global(svg) {
		color: var(--accent);
		flex-shrink: 0;
	}

	.status-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.status-list li {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: var(--text-small);
		color: var(--text-secondary);
	}
	.status-list :global(svg) {
		flex-shrink: 0;
		color: var(--text-muted);
	}
</style>
