<script lang="ts">
	import { modalFocus } from '$lib/visual/modalFocus';
	import {
		SlidersHorizontal,
		PackageOpen,
		ChevronRight,
		Server,
		FolderInput,
		ArrowLeft
	} from 'lucide-svelte';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { t } from '$lib/i18n/index.svelte';

	let {
		onClose,
		onChooseCustom,
		onChooseTflSelection,
		onChooseModrinth,
		onChooseServer,
		onChooseImport
	}: {
		onClose: () => void;
		onChooseCustom: () => void;
		onChooseTflSelection: () => void;
		onChooseModrinth: () => void;
		onChooseServer: () => void;
		onChooseImport: () => void;
	} = $props();

	let step = $state<'main' | 'modpack'>('main');

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (step === 'modpack') {
				step = 'main';
			} else {
				onClose();
			}
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="overlay" onclick={onClose} onkeydown={handleKeydown} role="button" tabindex="-1">
	<div
		class="modal"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		use:modalFocus={onClose}
		aria-modal="true"
		tabindex="-1"
	>
		{#if step === 'main'}
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
				<button type="button" class="choice-card" onclick={() => (step = 'modpack')}>
					<span class="choice-icon"><PackageOpen size={22} /></span>
					<span class="choice-text">
						<span class="choice-title">{t('createInstance.modpackTitle')}</span>
						<span class="choice-desc">{t('createInstance.modpackDesc')}</span>
					</span>
					<ChevronRight size={16} class="choice-arrow" />
				</button>
				<button type="button" class="choice-card" onclick={onChooseServer}>
					<span class="choice-icon"><Server size={22} /></span>
					<span class="choice-text">
						<span class="choice-title">{t('createInstance.serverTitle')}</span>
						<span class="choice-desc">{t('createInstance.serverDesc')}</span>
					</span>
					<ChevronRight size={16} class="choice-arrow" />
				</button>
				<button type="button" class="choice-card" onclick={onChooseImport}>
					<span class="choice-icon"><FolderInput size={22} /></span>
					<span class="choice-text">
						<span class="choice-title">{t('createInstance.importTitle')}</span>
						<span class="choice-desc">{t('createInstance.importDesc')}</span>
					</span>
					<ChevronRight size={16} class="choice-arrow" />
				</button>
			</div>

			<button type="button" class="cancel-btn" onclick={onClose}>{t('common.cancel')}</button>
		{:else}
			<div class="header-with-back">
				<button
					type="button"
					class="back-btn"
					onclick={() => (step = 'main')}
					aria-label={t('common.back')}
				>
					<ArrowLeft size={18} />
				</button>
				<h2>{t('createInstance.modpackTitle')}</h2>
			</div>
			<p class="subtitle">Elige el origen del modpack para tu nueva instancia.</p>

			<div class="choices">
				<button type="button" class="choice-card" onclick={onChooseTflSelection}>
					<span class="choice-icon"><Tfl width={20} height={20} /></span>
					<span class="choice-text">
						<span class="choice-title">TFL Selection</span>
						<span class="choice-desc">Distribución oficial, curada y optimizada por TFLives.</span>
					</span>
					<ChevronRight size={16} class="choice-arrow" />
				</button>
				<button type="button" class="choice-card" onclick={onChooseModrinth}>
					<span class="choice-icon"><PackageOpen size={22} /></span>
					<span class="choice-text">
						<span class="choice-title">Modrinth</span>
						<span class="choice-desc"
							>Explora miles de modpacks creados por la comunidad en Modrinth.</span
						>
					</span>
					<ChevronRight size={16} class="choice-arrow" />
				</button>
			</div>

			<div class="footer-actions">
				<button type="button" class="cancel-btn secondary" onclick={() => (step = 'main')}>
					Volver
				</button>
				<button type="button" class="cancel-btn" onclick={onClose}>{t('common.cancel')}</button>
			</div>
		{/if}
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

	.header-with-back {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.back-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		cursor: pointer;
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.back-btn:hover {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 12%, var(--bg-input));
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

	.footer-actions {
		display: flex;
		gap: 8px;
		margin-top: 16px;
	}

	.footer-actions .cancel-btn {
		margin-top: 0;
		flex: 1;
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
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.cancel-btn.secondary {
		background: transparent;
	}

	.cancel-btn:hover {
		border-color: var(--text-muted);
	}
</style>
