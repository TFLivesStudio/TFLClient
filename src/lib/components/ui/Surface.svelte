<script lang="ts">
	import type { Snippet } from 'svelte';

	/** Wrapper de layout que aplica un nivel de la jerarquía de superficies
	 * (sección 1e de global.css: Canvas → Workspace → Card → Interactive →
	 * Elevated → Overlay) — Entrega A. Cada nivel trae un radio/sombra por
	 * defecto razonable, pisable con `radius`/`shadow`. No es un componente
	 * de posicionamiento: para overlays fijos de pantalla completa (modal,
	 * menú) seguí usando Dialog/ConfirmDialog/ContextMenu, que ya manejan
	 * `position: fixed` — esto solo pinta la superficie. */
	let {
		level = 'card',
		radius,
		shadow,
		as = 'div',
		children,
		...rest
	}: {
		level?: 'canvas' | 'workspace' | 'card' | 'interactive' | 'elevated' | 'overlay';
		radius?: 'sm' | 'md' | 'lg' | 'none';
		shadow?: 'sm' | 'md' | 'lg' | 'none';
		as?: string;
		children?: Snippet;
		[key: string]: unknown;
	} = $props();
</script>

<svelte:element
	this={as}
	class="tfl-surface {level}"
	class:radius-sm={radius === 'sm'}
	class:radius-md={radius === 'md'}
	class:radius-lg={radius === 'lg'}
	class:radius-none={radius === 'none'}
	class:shadow-sm={shadow === 'sm'}
	class:shadow-md={shadow === 'md'}
	class:shadow-lg={shadow === 'lg'}
	class:shadow-none={shadow === 'none'}
	{...rest}
>
	{#if children}
		{@render children()}
	{/if}
</svelte:element>

<style>
	.tfl-surface {
		transition: background-color var(--duration-panel) var(--easing-standard);
	}

	.tfl-surface.canvas {
		background: var(--surface-canvas);
	}
	.tfl-surface.workspace {
		background: var(--surface-workspace);
	}
	.tfl-surface.card {
		background: var(--surface-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		box-shadow: var(--shadow-sm);
	}
	.tfl-surface.interactive {
		background: var(--surface-interactive);
		border-radius: var(--border-radius-sm);
	}
	.tfl-surface.elevated {
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-md);
	}
	.tfl-surface.overlay {
		background: var(--surface-overlay);
	}

	/* Overrides explícitos — pisan el radio/sombra por defecto del nivel. */
	.tfl-surface.radius-sm {
		border-radius: var(--border-radius-sm);
	}
	.tfl-surface.radius-md {
		border-radius: var(--border-radius);
	}
	.tfl-surface.radius-lg {
		border-radius: var(--border-radius-lg);
	}
	.tfl-surface.radius-none {
		border-radius: 0;
	}
	.tfl-surface.shadow-sm {
		box-shadow: var(--shadow-sm);
	}
	.tfl-surface.shadow-md {
		box-shadow: var(--shadow-md);
	}
	.tfl-surface.shadow-lg {
		box-shadow: var(--shadow-lg);
	}
	.tfl-surface.shadow-none {
		box-shadow: none;
	}
</style>
