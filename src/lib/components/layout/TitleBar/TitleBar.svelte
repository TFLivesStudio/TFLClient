<script lang="ts">
	import NotificationCenter from '$lib/components/refresh/NotificationCenter.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { al } from '$lib/activity/labels';
	import { navigation } from '$lib/visual/navigation.svelte';
	import { appState } from '$lib/state/state.svelte';
	import { Search } from 'lucide-svelte';

	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Minus, Square, Copy, X } from 'lucide-svelte';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import {
		SIDEBAR_COLLAPSED_WIDTH,
		SIDEBAR_EXPANDED_WIDTH,
		isSidebarEffectiveCollapsed
	} from '$lib/state/viewport.svelte';

	const win = getCurrentWindow();
	let maximized = $state(false);

	async function refreshMaximized() {
		maximized = await win.isMaximized();
	}

	onMount(() => {
		refreshMaximized();
		const unlisten = win.onResized(() => refreshMaximized());
		return () => {
			unlisten.then((f) => f());
		};
	});

	// Mismas constantes que Sidebar.svelte (viewport.svelte.ts) para el ancho
	// del bloque de marca — así nunca queda desalineado con el borde derecho
	// real de la sidebar al colapsar/expandir o al angostar la ventana
	// (Entrega B, sección 6.2). Ninguno de los dos componentes decide su
	// propio número.
	const brandCollapsed = $derived(isSidebarEffectiveCollapsed());
</script>

<div class="titlebar" data-tauri-drag-region>
	<div
		class="titlebar-brand"
		class:collapsed={brandCollapsed}
		data-tauri-drag-region
		style="width: {brandCollapsed ? SIDEBAR_COLLAPSED_WIDTH : SIDEBAR_EXPANDED_WIDTH}px"
	>
		<Tfl width="16" height="16" />
		{#if !brandCollapsed}<span>TFL Client</span>{/if}
	</div>
	<div class="titlebar-drag" data-tauri-drag-region>
		<span class="titlebar-location" data-tauri-drag-region
			>{appState.selectedInstance?.name ??
				(navigation.section === 'activity' ? al('title') : vr(navigation.section))}</span
		>
	</div>
	<button
		class="titlebar-search"
		aria-label={vr('focusSearch')}
		onclick={() => window.dispatchEvent(new Event('tfl:command-palette'))}
		><Search size={15} /><kbd>⌘ / Ctrl K</kbd></button
	>
	<NotificationCenter />

	<div class="titlebar-controls">
		<button
			type="button"
			class="titlebar-btn"
			aria-label={t('titleBar.minimize')}
			onclick={() => win.minimize()}
		>
			<Minus size={14} strokeWidth={1.75} />
		</button>
		<button
			type="button"
			class="titlebar-btn"
			aria-label={maximized ? t('titleBar.restore') : t('titleBar.maximize')}
			onclick={() => win.toggleMaximize()}
		>
			{#if maximized}
				<Copy size={12} strokeWidth={1.75} />
			{:else}
				<Square size={11} strokeWidth={1.75} />
			{/if}
		</button>
		<button
			type="button"
			class="titlebar-btn titlebar-btn-close"
			aria-label={t('settings.close')}
			onclick={() => win.close()}
		>
			<X size={14} strokeWidth={1.75} />
		</button>
	</div>
</div>

<style>
	.titlebar {
		position: relative;
		z-index: 30;
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: var(--titlebar-height);
		flex-shrink: 0;
		background: var(--surface-workspace);
		border-bottom: 1px solid var(--border);
		user-select: none;
	}

	.titlebar-brand {
		display: flex;
		align-items: center;
		gap: 8px;
		/* El ancho real lo pone el style="width: ...px" inline (mismas
		   constantes que Sidebar.svelte, ver script arriba) — transition
		   acá nomás para que el cambio de ancho se vea animado, no saltado. */
		height: 100%;
		padding: 0 12px;
		flex-shrink: 0;
		overflow: hidden;
		border-right: 1px solid var(--border);
		transition:
			width var(--duration-panel) var(--easing-standard),
			padding var(--duration-panel) var(--easing-standard);
	}

	.titlebar-brand.collapsed {
		justify-content: center;
		padding: 0;
	}

	.titlebar-brand :global(svg) {
		color: var(--accent);
		flex-shrink: 0;
	}

	.titlebar-brand span {
		font-family: var(--font-brand);
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.titlebar-drag {
		flex: 1;
		height: 100%;
	}

	.titlebar-controls {
		display: flex;
		height: 100%;
	}

	.titlebar-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 46px;
		height: 100%;
		background: transparent;
		border: none;
		color: var(--text-secondary);
		cursor: pointer;
		transition: background-color var(--duration-micro) var(--easing-standard);
	}

	.titlebar-btn:hover {
		background: rgba(255, 255, 255, 0.06);
		color: var(--text-primary);
	}

	.titlebar-btn-close:hover {
		background: var(--color-error);
		color: #ffffff;
	}

	.titlebar-location {
		padding-left: 22px;
		font-size: var(--text-caption);
		color: var(--text-muted);
		pointer-events: none;
	}
	.titlebar-drag {
		display: flex;
		align-items: center;
	}
	.titlebar-search {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 12px;
		background: none;
		border: 0;
		border-right: 1px solid var(--border);
		color: var(--text-muted);
	}
	.titlebar-search kbd {
		font-size: 10px;
	}
	.titlebar-brand span {
		font-family: var(--font-family);
		font-size: 13px;
		font-weight: 700;
		letter-spacing: -0.03em;
	}
</style>
