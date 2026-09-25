<script lang="ts">
	import { SlidersHorizontal, PackageOpen, ChevronRight } from 'lucide-svelte';
	import { t } from '$lib/i18n/index.svelte';

	let {
		onClose,
		onChooseCustom,
		onChooseModpack
	}: {
		onClose: () => void;
		onChooseCustom: () => void;
		onChooseModpack: () => void;
	} = $props();
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div
	class="overlay"
	onclick={onClose}
	onkeydown={(e) => e.key === 'Escape' && onClose()}
	role="button"
	tabindex="-1"
>
	<div
		class="modal"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<h2>{t('createInstance.title')}</h2>
		<p class="subtitle">{t('createInstance.chooserSubtitle')}</p>

		<div class="choices">
			<button type="button" class="choice-card" onclick={onChooseCustom}>
				<span class="choice-icon"><SlidersHorizontal size={22} /></span>
				<span class="choice-text">
					<span class="choice-title">{t('createInstance.customTitle')}</span>
					<span class="choice-desc">{t('createInstance.customDesc')}</span>
				</span>
				<ChevronRight size={16} class="choice-arrow" />
			</button>
			<button type="button" class="choice-card" onclick={onChooseModpack}>
				<span class="choice-icon"><PackageOpen size={22} /></span>
				<span class="choice-text">
					<span class="choice-title">{t('createInstance.modpackTitle')}</span>
					<span class="choice-desc">{t('createInstance.modpackDesc')}</span>
				</span>
				<ChevronRight size={16} class="choice-arrow" />
			</button>
		</div>

		<button type="button" class="cancel-btn" onclick={onClose}>{t('common.cancel')}</button>
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
		z-index: 100;
	}

	.modal {
		width: 440px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		box-shadow: var(--shadow-lg);
	}

	.modal h2 {
		font-size: var(--text-lg);
	}

	.subtitle {
		font-size: 0.8rem;
		color: var(--text-secondary);
		margin-bottom: 16px;
	}

	.choices {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.choice-card {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
		transition:
			border-color 0.15s,
			background 0.15s;
	}

	.choice-card:hover {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 8%, var(--bg-input));
	}

	.choice-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
		color: var(--accent);
	}

	.choice-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
		min-width: 0;
	}

	.choice-title {
		font-size: 0.92rem;
		font-weight: 700;
	}

	.choice-desc {
		font-size: 0.74rem;
		color: var(--text-secondary);
	}

	.choice-card :global(.choice-arrow) {
		flex-shrink: 0;
		color: var(--text-muted);
	}

	.cancel-btn {
		margin-top: 16px;
		padding: 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-weight: 600;
		font-size: 0.82rem;
		cursor: pointer;
	}
</style>
