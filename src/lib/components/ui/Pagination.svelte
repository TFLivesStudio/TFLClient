<script lang="ts">
	import { ChevronLeft, ChevronRight } from 'lucide-svelte';
	import IconButton from './IconButton.svelte';
	import { t } from '$lib/i18n/index.svelte';

	/** Paginación explícita (anterior/siguiente + indicador), no scroll
	 * infinito ni barra de números — pedido explícito del cliente: "permite
	 * orientarse en el catálogo, volver a resultados anteriores y controlar
	 * las consultas". `<button>` nativos (vía `IconButton`): foco y
	 * navegación por teclado gratis, sin reinventar nada. */
	let {
		offset,
		limit,
		totalHits,
		loading = false,
		onPrev,
		onNext
	}: {
		offset: number;
		limit: number;
		totalHits: number;
		loading?: boolean;
		onPrev: () => void;
		onNext: () => void;
	} = $props();

	const page = $derived(Math.floor(offset / limit) + 1);
	const totalPages = $derived(Math.max(1, Math.ceil(totalHits / limit)));
	const rangeStart = $derived(totalHits === 0 ? 0 : offset + 1);
	const rangeEnd = $derived(Math.min(offset + limit, totalHits));
	const hasPrev = $derived(offset > 0);
	const hasNext = $derived(offset + limit < totalHits);
</script>

<div class="tfl-pagination">
	<span class="tfl-pagination-range">
		{#if totalHits > 0}
			{t('pagination.showing', { start: rangeStart, end: rangeEnd, total: totalHits })}
		{/if}
	</span>
	<div class="tfl-pagination-controls">
		<IconButton
			label={t('pagination.prev')}
			variant="ghost"
			size="compact"
			disabled={!hasPrev || loading}
			onclick={onPrev}
		>
			<ChevronLeft size={15} />
		</IconButton>
		<span class="tfl-pagination-page">{t('pagination.pageOf', { page, totalPages })}</span>
		<IconButton
			label={t('pagination.next')}
			variant="ghost"
			size="compact"
			disabled={!hasNext || loading}
			onclick={onNext}
		>
			<ChevronRight size={15} />
		</IconButton>
	</div>
</div>

<style>
	.tfl-pagination {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 8px;
		padding: 6px 2px;
	}

	.tfl-pagination-range {
		font-size: var(--text-caption);
		color: var(--text-muted);
	}

	.tfl-pagination-controls {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.tfl-pagination-page {
		font-size: var(--text-caption);
		font-weight: 600;
		color: var(--text-secondary);
		white-space: nowrap;
		padding: 0 4px;
	}
</style>
