<script lang="ts">
	import { Surface, EmptyState } from '$lib/components/ui';
	import { t } from '$lib/i18n/index.svelte';
	import { History } from 'lucide-svelte';
	import type { ActivityEntry } from '$lib/types/types';

	/** Panel "Actividad reciente" de la Home — Entrega C. Markup/estilos
	 * extraídos de WelcomeState.svelte (líneas 423-451 originales); los
	 * datos (`getRecentActivity(5)` + íconos por nombre de instancia)
	 * siguen cargándose en el orquestador — acá solo se formatean para
	 * mostrar, con las mismas funciones (`relativeDayLabel`/`durationLabel`)
	 * que ya existían. */
	let {
		activity,
		iconUrls,
		relativeDayLabel,
		durationLabel
	}: {
		activity: ActivityEntry[];
		iconUrls: Record<string, string | null>;
		relativeDayLabel: (unixSecs: number) => string;
		durationLabel: (secs: number) => string;
	} = $props();
</script>

<Surface level="card" as="section">
	<div class="panel">
		<div class="panel-title">
			<History size={13} />
			<span>{t('home.recentActivity')}</span>
		</div>
		{#if activity.length === 0}
			<EmptyState title={t('home.noActivityYet')} description={t('home.noActivityYetHint')}>
				{#snippet icon()}<History size={22} />{/snippet}
			</EmptyState>
		{:else}
			<ul class="panel-list">
				{#each activity as entry (entry.id)}
					<li class="activity-row">
						<span class="activity-icon">
							{#if iconUrls[entry.instance_name]}
								<img src={iconUrls[entry.instance_name]} alt="" />
							{:else}
								{entry.instance_name.charAt(0).toUpperCase()}
							{/if}
						</span>
						<div class="activity-info">
							<span class="activity-name">{entry.instance_name}</span>
							<span class="activity-meta">
								{relativeDayLabel(entry.ended_at)} · {durationLabel(entry.duration_secs)}
							</span>
						</div>
					</li>
				{/each}
			</ul>
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

	.panel-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.activity-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.activity-icon {
		flex-shrink: 0;
		width: 26px;
		height: 26px;
		border-radius: var(--border-radius-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		font-size: var(--text-caption);
		font-weight: 800;
		color: var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
	}
	.activity-icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.activity-info {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}
	.activity-name {
		font-size: var(--text-small);
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.activity-meta {
		font-size: var(--text-caption);
		color: var(--text-muted);
	}
</style>
