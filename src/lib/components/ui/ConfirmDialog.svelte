<script lang="ts">
	import { AlertTriangle } from 'lucide-svelte';

	let {
		title,
		message,
		confirmLabel = 'Confirmar',
		danger = false,
		onConfirm,
		onCancel
	}: {
		title: string;
		message: string;
		confirmLabel?: string;
		danger?: boolean;
		onConfirm: () => void;
		onCancel: () => void;
	} = $props();
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onCancel()} />

<div
	class="overlay"
	onclick={onCancel}
	onkeydown={(e) => e.key === 'Escape' && onCancel()}
	role="button"
	tabindex="-1"
>
	<div
		class="dialog"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="alertdialog"
		aria-modal="true"
		tabindex="-1"
	>
		{#if danger}
			<div class="icon danger"><AlertTriangle size={20} /></div>
		{/if}
		<h3>{title}</h3>
		<p>{message}</p>
		<div class="actions">
			<button type="button" class="btn" onclick={onCancel}>Cancelar</button>
			<button type="button" class="btn" class:danger onclick={onConfirm}>
				{confirmLabel}
			</button>
		</div>
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}

	.dialog {
		width: 320px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 22px;
		box-shadow: var(--shadow-lg);
		text-align: center;
	}

	.icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border-radius: 50%;
		margin: 0 auto 12px;
	}

	.icon.danger {
		background: rgba(var(--color-error-rgb), 0.15);
		color: var(--color-error);
	}

	h3 {
		font-size: 0.95rem;
		margin-bottom: 6px;
	}

	p {
		font-size: 0.8rem;
		color: var(--text-secondary);
		margin-bottom: 18px;
	}

	.actions {
		display: flex;
		gap: 8px;
	}

	.btn {
		flex: 1;
		padding: 9px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-weight: 600;
		font-size: 0.8rem;
		cursor: pointer;
	}

	.btn.danger {
		background: var(--color-error);
		border-color: var(--color-error);
		color: #ffffff;
	}
</style>
