<script lang="ts">
	import { IconButton } from '$lib/components/ui';
	import { t } from '$lib/i18n/index.svelte';
	import { ChevronLeft, ChevronRight } from 'lucide-svelte';
	import type { InstanceData } from '$lib/types/types';

	/** Carrusel "Otras instancias" de la Home — Entrega C. Markup/estilos
	 * extraídos de WelcomeState.svelte (líneas 491-532 originales); el
	 * orden/selección de instancias (`sortedByRecent.slice(1,4)`,
	 * `selectInstance`) sigue decidiéndose en el orquestador. Solo se
	 * renderiza cuando hay algo para mostrar — el caller ya filtra eso. */
	let {
		instances,
		iconUrls,
		onSelect
	}: {
		instances: InstanceData[];
		iconUrls: Record<string, string | null>;
		onSelect: (instance: InstanceData) => void;
	} = $props();

	let carouselEl: HTMLDivElement | undefined = $state();
	function scrollCarousel(dir: 1 | -1) {
		carouselEl?.scrollBy({ left: dir * 220, behavior: 'smooth' });
	}
</script>

<div class="others-block">
	<div class="others-head">
		<span class="others-label">{t('home.otherInstances')}</span>
		<div class="others-nav">
			<IconButton
				label={t('home.scrollLeft')}
				variant="secondary"
				size="compact"
				onclick={() => scrollCarousel(-1)}
			>
				<ChevronLeft size={14} />
			</IconButton>
			<IconButton
				label={t('home.scrollRight')}
				variant="secondary"
				size="compact"
				onclick={() => scrollCarousel(1)}
			>
				<ChevronRight size={14} />
			</IconButton>
		</div>
	</div>
	<div class="others-grid" bind:this={carouselEl}>
		{#each instances as instance (instance.uuid)}
			<button type="button" class="other-card" onclick={() => onSelect(instance)}>
				<span class="other-icon">
					{#if iconUrls[instance.uuid]}
						<img src={iconUrls[instance.uuid]} alt="" />
					{:else}
						{instance.name.charAt(0).toUpperCase()}
					{/if}
				</span>
				<span class="other-text">
					<span class="other-name">{instance.name}</span>
					<span class="other-meta">{instance.mc_version} · {instance.loader}</span>
				</span>
			</button>
		{/each}
	</div>
</div>

<style>
	.others-block {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.others-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.others-label {
		font-size: var(--text-caption);
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
	}
	.others-nav {
		display: flex;
		gap: 4px;
	}
	.others-grid {
		display: flex;
		gap: 10px;
		overflow-x: auto;
		scroll-snap-type: x proximity;
		padding-bottom: 4px;
		scrollbar-width: thin;
	}
	.other-card {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px 10px 10px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--surface-card);
		color: var(--text-primary);
		cursor: pointer;
		flex-shrink: 0;
		min-width: 170px;
		scroll-snap-align: start;
	}
	.other-card:hover {
		border-color: color-mix(in srgb, var(--accent) 40%, var(--border));
	}
	.other-icon {
		flex-shrink: 0;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		font-size: var(--text-small);
		font-weight: 800;
		color: var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
	}
	.other-icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.other-text {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}
	.other-name {
		font-size: var(--text-small);
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.other-meta {
		font-size: var(--text-caption);
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
</style>
