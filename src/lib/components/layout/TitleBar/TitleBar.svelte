<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Minus, Square, Copy, X } from 'lucide-svelte';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { t } from '$lib/i18n/index.svelte';

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
</script>

<div class="titlebar" data-tauri-drag-region>
	<div class="titlebar-brand" data-tauri-drag-region>
		<Tfl width="16" height="16" />
		<span>TFL Client</span>
	</div>
	<div class="titlebar-drag" data-tauri-drag-region></div>

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
		z-index: 1;
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: var(--titlebar-height);
		flex-shrink: 0;
		background: var(--bg-sidebar);
		border-bottom: 1px solid var(--border);
		user-select: none;
	}

	.titlebar-brand {
		display: flex;
		align-items: center;
		gap: 8px;
		width: var(--sidebar-width);
		height: 100%;
		padding: 0 12px;
		flex-shrink: 0;
		border-right: 1px solid var(--border);
	}

	.titlebar-brand :global(svg) {
		color: var(--accent);
		flex-shrink: 0;
	}

	.titlebar-brand span {
		font-family: var(--font-brand);
		font-size: 0.78rem;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.titlebar-drag { flex: 1; height: 100%; }

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
		transition: background-color 0.12s;
	}

	.titlebar-btn:hover {
		background: rgba(255, 255, 255, 0.06);
		color: var(--text-primary);
	}

	.titlebar-btn-close:hover {
		background: var(--color-error);
		color: #ffffff;
	}
</style>
