<script lang="ts">
	import { Surface, ErrorState, Skeleton } from '$lib/components/ui';
	import { t } from '$lib/i18n/index.svelte';
	import { tr } from '$lib/i18n/experience';
	import { experience } from '$lib/state/experience.svelte';
	import { Cpu, Monitor, MemoryStick, HardDrive } from 'lucide-svelte';
	import type { SystemStatus } from '$lib/types/types';

	let {
		status,
		loading,
		onRetry
	}: {
		status: SystemStatus | null;
		loading: boolean;
		onRetry: () => void;
	} = $props();
	const advanced = $derived(experience.mode === 'advanced');
	const unavailable = $derived(tr('No disponible', 'Unavailable'));
	function gib(value: number | null | undefined): string {
		return typeof value === 'number' && Number.isFinite(value) && value >= 0
			? `${value.toFixed(value < 10 ? 1 : 0)} GiB`
			: unavailable;
	}
</script>

<Surface level="card" as="section">
	<div class="panel">
		<div class="panel-title">
			<Cpu size={14} /><span>{t('home.systemStatus')}</span><small>{tr('Equipo', 'Computer')}</small
			>
		</div>
		{#if loading}
			<div
				class="status-list"
				aria-label={tr('Cargando hardware del equipo', 'Loading computer hardware')}
				aria-busy="true"
			>
				<Skeleton variant="text" width="85%" /><Skeleton variant="text" width="70%" /><Skeleton
					variant="text"
					width="60%"
				/>
			</div>
		{:else if status}
			<dl class="status-list">
				<div class="hardware-row">
					<Cpu size={16} />
					<div>
						<dt>CPU</dt>
						<dd>{status.cpu_model || unavailable}</dd>
					</div>
				</div>
				<div class="hardware-row">
					<Monitor size={16} />
					<div>
						<dt>GPU</dt>
						{#if status.gpu_adapters?.length}
							{#each status.gpu_adapters as adapter, index (`${adapter.name}:${index}`)}
								<dd>
									{adapter.name}{#if advanced}<small
											>{adapter.provider}{#if adapter.vram_bytes != null}
												· VRAM {gib(adapter.vram_bytes / 1073741824)}{/if}</small
										>{/if}
								</dd>
							{/each}
						{:else}<dd>{unavailable}</dd>{/if}
					</div>
				</div>
				<div class="hardware-row">
					<MemoryStick size={16} />
					<div>
						<dt>{tr('Memoria', 'Memory')}</dt>
						<dd>
							{status.ram_total_mb > 0
								? `${gib(status.ram_available_mb / 1024)} ${tr('disponibles de', 'available of')} ${gib(status.ram_total_mb / 1024)}`
								: unavailable}
						</dd>
						{#if advanced}<dd class="context">
								{tr('Límite configurado para Minecraft', 'Configured Minecraft limit')}: {gib(
									status.ram_assigned_mb / 1024
								)}
							</dd>{/if}
					</div>
				</div>
				<div class="hardware-row">
					<HardDrive size={16} />
					<div>
						<dt>{tr('Almacenamiento de instancias', 'Instance storage')}</dt>
						<dd>
							{status.disk_free_gb != null && status.disk_total_gb != null
								? `${gib(status.disk_free_gb)} ${tr('libres de', 'free of')} ${gib(status.disk_total_gb)}`
								: unavailable}
						</dd>
						{#if advanced && status.instances_dir}<dd
								class="context path"
								title={status.instances_dir}
							>
								{status.instances_dir}
							</dd>{/if}
					</div>
				</div>
			</dl>
			{#if status.hardware_error}<p class="hardware-notice" role="status">
					{tr(
						'Algunos datos de hardware no están disponibles.',
						'Some hardware information is unavailable.'
					)}
				</p>{/if}
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
		gap: 16px;
		padding: 16px;
		min-width: 0;
	}
	.panel-title {
		display: flex;
		align-items: center;
		gap: 7px;
		color: var(--text-primary);
		font-size: var(--text-small);
		font-weight: 700;
	}
	.panel-title :global(svg) {
		color: var(--accent);
	}
	.panel-title small {
		margin-left: auto;
		color: var(--text-secondary);
		font-size: var(--text-caption);
		font-weight: 400;
	}
	.status-list {
		display: flex;
		flex-direction: column;
		gap: 14px;
		margin: 0;
	}
	.hardware-row {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		min-width: 0;
	}
	.hardware-row > div {
		min-width: 0;
	}
	.hardware-row > :global(svg) {
		flex-shrink: 0;
		color: var(--text-secondary);
		margin-top: 3px;
	}
	dt {
		font-size: var(--text-caption);
		color: var(--text-secondary);
		margin-bottom: 3px;
	}
	dd {
		margin: 0;
		color: var(--text-primary);
		font-size: var(--text-small);
		line-height: 1.45;
		overflow-wrap: anywhere;
	}
	dd + dd {
		margin-top: 5px;
	}
	dd small {
		display: block;
		color: var(--text-secondary);
		font-size: var(--text-caption);
	}
	.context,
	.hardware-notice {
		color: var(--text-secondary);
		font-size: var(--text-caption);
	}
	.path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.hardware-notice {
		margin: 0;
		line-height: 1.5;
	}
</style>
