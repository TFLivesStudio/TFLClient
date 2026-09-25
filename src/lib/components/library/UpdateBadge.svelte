<script lang="ts">
	import {
		updateState,
		installDownloadedUpdate,
		dismissUpdateBadge
	} from '$lib/state/updateState.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { Download, X, Loader2 } from 'lucide-svelte';
</script>

{#if updateState.downloaded && updateState.update}
	<div class="update-badge">
		<div class="badge-body">
			<button
				type="button"
				class="badge-main"
				onclick={installDownloadedUpdate}
				disabled={updateState.installing}
			>
				{#if updateState.installing}
					<Loader2 size={16} class="spin" />
				{:else}
					<Download size={16} />
				{/if}
				<span class="badge-text">
					<span class="badge-title">{t('updateBadge.available')}</span>
					<span class="badge-sub">
						{updateState.installing
							? t('updateBadge.installing')
							: t('updateBadge.clickToInstall', { version: updateState.update.version })}
					</span>
				</span>
			</button>
			{#if updateState.error}<p class="badge-error">{updateState.error}</p>{/if}
		</div>
		<button
			type="button"
			class="badge-close"
			onclick={dismissUpdateBadge}
			aria-label={t('settings.close')}
			disabled={updateState.installing}
		>
			<X size={12} />
		</button>
	</div>
{/if}

<style>
	.update-badge {
		position: fixed;
		left: 16px;
		bottom: 16px;
		z-index: 200;
		background: var(--bg-card);
		border: 1px solid var(--accent);
		border-radius: var(--border-radius);
		box-shadow: var(--shadow-md);
		padding: 4px;
		display: flex;
		align-items: flex-start;
		gap: 2px;
		animation: tfl-badge-in 0.25s ease;
	}

	.badge-body {
		display: flex;
		flex-direction: column;
	}

	.badge-main {
		display: flex;
		align-items: center;
		gap: 10px;
		background: none;
		border: none;
		color: var(--accent);
		cursor: pointer;
		padding: 8px 10px;
		text-align: left;
	}

	.badge-main:disabled {
		cursor: default;
		opacity: 0.8;
	}

	.badge-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.badge-title {
		font-size: 0.78rem;
		font-weight: 700;
		color: var(--text-primary);
	}

	.badge-sub {
		font-size: 0.68rem;
		color: var(--text-secondary);
	}

	.badge-close {
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		padding: 4px;
		border-radius: 4px;
	}

	.badge-close:hover:not(:disabled) {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.08);
	}

	.badge-error {
		font-size: 0.68rem;
		color: var(--color-error);
		padding: 0 10px 6px;
	}

	@keyframes tfl-badge-in {
		from {
			opacity: 0;
			transform: translateY(8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
