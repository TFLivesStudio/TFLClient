<script lang="ts">
	import type { Snippet } from 'svelte';

	/** Tooltip genérico posicionable — Entrega A. A diferencia de
	 * `InfoTooltip` (ícono de ayuda fijo, burbuja abajo), este envuelve
	 * CUALQUIER trigger vía `children` y soporta las 4 posiciones. No se
	 * tocó `InfoTooltip.svelte` en esta entrega (fuera del alcance pedido:
	 * "no toques ningún otro archivo fuera de lo descrito"); queda como
	 * trabajo de una entrega futura unificar su lógica de hover/focus con
	 * la de acá si se quiere deduplicar del todo. */
	let {
		content,
		position = 'top',
		disabled = false,
		children
	}: {
		content: string | Snippet;
		position?: 'top' | 'bottom' | 'left' | 'right';
		disabled?: boolean;
		children: Snippet;
	} = $props();

	let open = $state(false);
	let trigger = $state<HTMLSpanElement>();
	const id = $props.id();
	/** Portalled only while open: scrolling panes keep their clipping boundaries. */
	function positionBubble(node: HTMLSpanElement) {
		document.body.appendChild(node);
		const control = trigger?.querySelector('button, a, input, [tabindex]');
		const previousDescription = control?.getAttribute('aria-describedby');
		const description = [previousDescription, id].filter(Boolean).join(' ');
		control?.setAttribute('aria-describedby', description);
		const place = () => {
			if (!trigger) return;
			const r = trigger.getBoundingClientRect(),
				b = node.getBoundingClientRect();
			let x = r.left + (r.width - b.width) / 2,
				y = r.top - b.height - 8;
			if (position === 'bottom') y = r.bottom + 8;
			if (position === 'left' || position === 'right') {
				x = position === 'left' ? r.left - b.width - 8 : r.right + 8;
				y = r.top + (r.height - b.height) / 2;
			}
			node.style.left = Math.max(8, Math.min(x, innerWidth - b.width - 8)) + 'px';
			node.style.top = Math.max(8, Math.min(y, innerHeight - b.height - 8)) + 'px';
			node.style.visibility = 'visible';
		};
		const escape = (e: KeyboardEvent) => {
			if (e.key === 'Escape') open = false;
		};
		const observer = new ResizeObserver(place);
		if (trigger) observer.observe(trigger);
		observer.observe(node);
		window.addEventListener('resize', place);
		window.addEventListener('scroll', place, true);
		window.addEventListener('keydown', escape);
		place();
		return {
			destroy() {
				observer.disconnect();
				window.removeEventListener('resize', place);
				window.removeEventListener('scroll', place, true);
				window.removeEventListener('keydown', escape);
				if (control?.getAttribute('aria-describedby') === description) {
					if (previousDescription) control.setAttribute('aria-describedby', previousDescription);
					else control.removeAttribute('aria-describedby');
				}
				node.remove();
			}
		};
	}
</script>

<span
	class="tfl-tooltip-wrap"
	bind:this={trigger}
	role="group"
	onmouseenter={() => !disabled && (open = true)}
	onmouseleave={() => (open = false)}
	onfocusin={() => !disabled && (open = true)}
	onfocusout={() => (open = false)}
>
	{@render children()}
	{#if open && !disabled}
		<span {id} class="tfl-tooltip-bubble" role="tooltip" use:positionBubble>
			{#if typeof content === 'string'}
				{content}
			{:else}
				{@render content()}
			{/if}
		</span>
	{/if}
</span>

<style>
	.tfl-tooltip-wrap {
		position: relative;
		display: inline-flex;
	}

	.tfl-tooltip-bubble {
		position: fixed;
		visibility: hidden;
		z-index: var(--z-tooltip);
		width: max-content;
		max-width: min(260px, calc(100vw - 16px));
		overflow-wrap: anywhere;
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-card);
		box-shadow: var(--shadow-md);
		color: var(--text-secondary);
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		font-weight: 400;
		text-align: left;
		pointer-events: none;
		animation: tfl-tooltip-in var(--duration-micro) var(--easing-decelerate);
	}

	@keyframes tfl-tooltip-in {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}
</style>
