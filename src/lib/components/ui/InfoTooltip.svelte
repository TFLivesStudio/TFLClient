<script lang="ts">
	import { CircleHelp } from 'lucide-svelte';

	let { text }: { text: string } = $props();

	let open = $state(false);
</script>

<span
	class="info-tooltip"
	role="group"
	onmouseenter={() => (open = true)}
	onmouseleave={() => (open = false)}
	onfocusin={() => (open = true)}
	onfocusout={() => (open = false)}
>
	<button type="button" class="info-btn" aria-label={text} tabindex="0">
		<CircleHelp size={14} />
	</button>
	{#if open}
		<span class="info-bubble" role="tooltip">{text}</span>
	{/if}
</span>

<style>
	.info-tooltip {
		position: relative;
		display: inline-flex;
		align-items: center;
	}

	.info-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		padding: 0;
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: help;
	}

	.info-btn:hover,
	.info-tooltip:focus-within .info-btn {
		color: var(--accent);
	}

	.info-bubble {
		position: absolute;
		z-index: 50;
		bottom: calc(100% + 8px);
		left: 50%;
		transform: translateX(-50%);
		width: max-content;
		max-width: 260px;
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-card);
		box-shadow: var(--shadow-md);
		color: var(--text-secondary);
		font-size: 0.7rem;
		font-weight: 400;
		line-height: 1.4;
		text-align: left;
		pointer-events: none;
		animation: tfl-tooltip-in 0.1s ease;
	}

	@keyframes tfl-tooltip-in {
		from {
			opacity: 0;
			transform: translateX(-50%) translateY(2px);
		}
		to {
			opacity: 1;
			transform: translateX(-50%) translateY(0);
		}
	}
</style>
