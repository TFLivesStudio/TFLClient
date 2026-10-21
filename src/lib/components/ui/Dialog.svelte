<script lang="ts">
	import type { Snippet } from 'svelte';
	import { X } from 'lucide-svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { modalFocus } from '$lib/visual/modalFocus';
	let {
		onClose,
		closeOnEscape = true,
		closeOnOutsideClick = true,
		ariaLabel,
		width,
		children
	}: {
		onClose: () => void;
		closeOnEscape?: boolean;
		closeOnOutsideClick?: boolean;
		ariaLabel?: string;
		width?: string;
		children: Snippet;
	} = $props();
</script>

<div
	class="tfl-dialog-overlay"
	onclick={(e) => closeOnOutsideClick && e.target === e.currentTarget && onClose()}
	role="presentation"
>
	<div
		class="tfl-dialog-panel"
		style={width ? `width: ${width};` : undefined}
		role="dialog"
		use:modalFocus={closeOnEscape ? onClose : undefined}
		aria-modal="true"
		aria-label={ariaLabel}
		tabindex="-1"
	>
		<button type="button" class="tfl-dialog-close" aria-label={t('ui.close')} onclick={onClose}>
			<X size={16} />
		</button>
		{@render children()}
	</div>
</div>

<style>
	.tfl-dialog-overlay {
		position: fixed;
		inset: 0;
		z-index: var(--z-modal);
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-overlay);
	}

	.tfl-dialog-panel {
		position: relative;
		width: 400px;
		max-width: calc(100vw - 32px);
		max-height: calc(100vh - 32px);
		overflow: auto;
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		padding: 24px;
		animation: tfl-dialog-in var(--duration-panel) var(--easing-decelerate);
	}

	.tfl-dialog-close {
		position: absolute;
		top: 12px;
		right: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		width: var(--control-size-compact);
		height: var(--control-size-compact);
		border: none;
		border-radius: var(--border-radius-sm);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: background-color var(--duration-micro) var(--easing-standard);
	}

	.tfl-dialog-close:hover {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
		color: var(--text-primary);
	}

	@keyframes tfl-dialog-in {
		from {
			opacity: 0;
			transform: scale(0.97) translateY(4px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}
</style>
