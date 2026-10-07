<script lang="ts">
	import {
		updateState,
		downloadUpdate,
		installDownloadedUpdate,
		retryUpdate,
		dismissUpdateBadge
	} from '$lib/state/updateState.svelte';
	import { isBadgeVisible, progressPercent } from '$lib/state/updateMachine';
	import { t } from '$lib/i18n/index.svelte';
	import { Download, RotateCw, AlertTriangle, X, Loader2 } from 'lucide-svelte';

	const visible = $derived(isBadgeVisible(updateState, updateState.dismissed));
	const percent = $derived(progressPercent(updateState));
	const version = $derived(updateState.version ?? '');
	const phase = $derived(updateState.phase);
</script>

{#if visible}
	<div class="update-badge" class:is-error={phase === 'error'} role="status" aria-live="polite">
		<span class="badge-icon">
			{#if phase === 'installing'}
				<Loader2 size={16} class="spin" />
			{:else if phase === 'downloading'}
				<Download size={16} />
			{:else if phase === 'downloaded'}
				<RotateCw size={16} />
			{:else if phase === 'error'}
				<AlertTriangle size={16} />
			{:else}
				<Download size={16} />
			{/if}
		</span>

		<span class="badge-text">
			{#if phase === 'available'}
				<span class="badge-title">{t('updateBadge.available')}</span>
				<span class="badge-sub">{t('updateBadge.availableSub', { version })}</span>
			{:else if phase === 'downloading'}
				<span class="badge-title">{t('updateBadge.downloading')}</span>
				<span class="badge-sub">
					{percent === null
						? t('updateBadge.downloadingSubUnknown', { version })
						: t('updateBadge.downloadingSub', { version, percent })}
				</span>
				<span
					class="badge-progress"
					class:indeterminate={percent === null}
					role="progressbar"
					aria-valuemin={0}
					aria-valuemax={100}
					aria-valuenow={percent ?? undefined}
				>
					<span
						class="badge-progress-fill"
						style:width={percent === null ? undefined : `${percent}%`}
					></span>
				</span>
			{:else if phase === 'downloaded'}
				<span class="badge-title">{t('updateBadge.ready')}</span>
				<span class="badge-sub">{t('updateBadge.readySub', { version })}</span>
			{:else if phase === 'installing'}
				<span class="badge-title">{t('updateBadge.installing')}</span>
			{:else if phase === 'error'}
				<span class="badge-title">{t('updateBadge.error')}</span>
				<span class="badge-sub badge-error" title={updateState.error?.message}>
					{updateState.error?.message}
				</span>
			{/if}
		</span>

		{#if phase === 'available'}
			<button type="button" class="badge-action" onclick={downloadUpdate}>
				{t('updateBadge.download')}
			</button>
		{:else if phase === 'downloaded'}
			<button type="button" class="badge-action primary" onclick={installDownloadedUpdate}>
				{t('updateBadge.restart')}
			</button>
		{:else if phase === 'error'}
			<button type="button" class="badge-action" onclick={retryUpdate}>
				{t('updateBadge.retry')}
			</button>
		{/if}

		<button
			type="button"
			class="badge-close"
			onclick={dismissUpdateBadge}
			aria-label={t('settings.close')}
			disabled={phase === 'installing'}
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
		max-width: min(420px, calc(100vw - 32px));
		background: var(--bg-card);
		border: 1px solid var(--accent);
		border-radius: var(--border-radius);
		box-shadow: var(--shadow-md);
		padding: 8px 6px 8px 12px;
		display: flex;
		align-items: center;
		gap: 10px;
		animation: tfl-badge-in 0.25s ease;
	}

	.update-badge.is-error {
		border-color: var(--color-error);
	}

	.badge-icon {
		display: inline-flex;
		flex-shrink: 0;
		color: var(--accent);
	}

	.is-error .badge-icon {
		color: var(--color-error);
	}

	.badge-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
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

	.badge-error {
		color: var(--color-error);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 240px;
	}

	.badge-progress {
		position: relative;
		display: block;
		width: 160px;
		max-width: 100%;
		height: 3px;
		margin-top: 4px;
		border-radius: 2px;
		background: var(--border);
		overflow: hidden;
	}

	.badge-progress-fill {
		display: block;
		height: 100%;
		width: 0;
		border-radius: 2px;
		background: var(--accent);
		transition: width 0.2s ease;
	}

	.badge-progress.indeterminate .badge-progress-fill {
		width: 40%;
		animation: tfl-badge-slide 1.2s ease-in-out infinite;
	}

	.badge-action {
		flex-shrink: 0;
		padding: 6px 12px;
		border: 1px solid color-mix(in srgb, var(--accent) 55%, var(--border));
		border-radius: var(--border-radius-sm);
		background: transparent;
		color: var(--accent);
		font-size: 0.72rem;
		font-weight: 700;
		cursor: pointer;
		white-space: nowrap;
		transition:
			background 0.16s,
			transform 0.16s;
	}

	.badge-action:hover {
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		transform: translateY(-1px);
	}

	.badge-action.primary {
		border-color: transparent;
		background: var(--accent);
		color: var(--accent-text);
	}

	.badge-action.primary:hover {
		background: var(--accent);
		box-shadow: 0 6px 18px rgba(var(--accent-rgb), 0.26);
	}

	.is-error .badge-action {
		border-color: color-mix(in srgb, var(--color-error) 55%, var(--border));
		color: var(--color-error);
	}

	.is-error .badge-action:hover {
		background: color-mix(in srgb, var(--color-error) 14%, transparent);
	}

	.badge-close {
		align-self: flex-start;
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

	.badge-close:disabled {
		visibility: hidden;
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

	@keyframes tfl-badge-slide {
		0% {
			transform: translateX(-100%);
		}
		100% {
			transform: translateX(260%);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.update-badge,
		.badge-progress.indeterminate .badge-progress-fill {
			animation: none;
		}
	}
</style>
