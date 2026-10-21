<script lang="ts">
	import Select from './Select.svelte';
	import type { SortIndex } from '$lib/types/types';
	import { t } from '$lib/i18n/index.svelte';

	/** Selector de orden — solo los índices que Modrinth API v2 admite de
	 * verdad (relevance/downloads/follows/newest/updated), nada inventado.
	 * Envuelve el `Select` genérico del design system en vez de reinventar
	 * un `<select>` — acá lo específico es SOLO la lista de opciones y la
	 * regla de "sin texto de búsqueda, Relevancia no significa nada".
	 *
	 * `onchange` en vez de `bind:value`: a propósito, porque cambiar el
	 * orden necesita además volver a página 1 y relanzar la búsqueda (ver
	 * `ContentExplorer.setSort`) — un `bind:` de dos vías pisaría ese
	 * efecto secundario sin que el caller se entere del cambio. */
	let {
		value,
		hasQuery,
		disabled = false,
		onchange
	}: {
		value: SortIndex;
		hasQuery: boolean;
		disabled?: boolean;
		onchange: (value: SortIndex) => void;
	} = $props();

	const options = $derived(
		(
			[
				{ value: 'relevance', label: t('sort.relevance') },
				{ value: 'downloads', label: t('sort.downloads') },
				{ value: 'follows', label: t('sort.follows') },
				{ value: 'newest', label: t('sort.newest') },
				{ value: 'updated', label: t('sort.updated') }
			] as { value: SortIndex; label: string }[]
		).filter((o) => hasQuery || o.value !== 'relevance')
	);

	// "Writable `$derived`" — Svelte 5 permite reasignar un `$derived`
	// (override temporal hasta que `value` cambie de nuevo), así no hace
	// falta un segundo `$state` + efecto de sincronización manual.
	let localValue = $derived(value);
	$effect(() => {
		if (localValue !== value) onchange(localValue);
	});
</script>

<Select bind:value={localValue} {options} {disabled} aria-label={t('sort.label')} />
