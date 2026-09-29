<script lang="ts">
	import { appState } from '$lib/state/state.svelte';
	import { updateSettings } from '$lib/api/tflApi';
	import { Wifi, Globe } from 'lucide-svelte';
	import { t } from '$lib/i18n/index.svelte';

	let { onDone }: { onDone: () => void } = $props();

	let busy = $state(false);
	let error = $state<string | null>(null);

	async function dismiss() {
		if (!appState.settings) return;
		busy = true;
		error = null;
		try {
			await updateSettings({ ...appState.settings, server_connection_info_shown: true });
			appState.settings.server_connection_info_shown = true;
			onDone();
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
</script>

<div class="overlay">
	<div class="card" role="dialog" aria-modal="true">
		<h1>{t('serverConnectionInfo.title')}</h1>
		<p class="lead">{t('serverConnectionInfo.lead')}</p>

		<div class="rows">
			<div class="row">
				<span class="row-icon"><Wifi size={18} /></span>
				<span class="row-text">
					<span class="row-title">{t('serverConnectionInfo.upnpTitle')}</span>
					<span class="row-sub">{t('serverConnectionInfo.upnpSub')}</span>
				</span>
			</div>
			<div class="row">
				<span class="row-icon"><Globe size={18} /></span>
				<span class="row-text">
					<span class="row-title">{t('serverConnectionInfo.playitTitle')}</span>
					<span class="row-sub">{t('serverConnectionInfo.playitSub')}</span>
				</span>
			</div>
		</div>

		{#if error}<p class="prompt-error">{error}</p>{/if}

		<button type="button" class="ok-btn" disabled={busy} onclick={dismiss}>
			{t('serverConnectionInfo.gotIt')}
		</button>
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

	.card {
		width: min(460px, calc(100vw - 32px));
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 28px;
		box-shadow: var(--shadow-lg);
	}

	.card h1 {
		font-size: var(--text-lg);
		margin-bottom: 8px;
	}

	.lead {
		color: var(--text-secondary);
		font-size: 0.82rem;
		margin-bottom: 18px;
	}

	.rows {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px 16px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-input);
	}

	.row-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
		color: var(--accent);
	}

	.row-text {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.row-title {
		font-size: 0.86rem;
		font-weight: 700;
		color: var(--text-primary);
	}

	.row-sub {
		font-size: 0.72rem;
		color: var(--text-muted);
		line-height: 1.4;
	}

	.prompt-error {
		margin-top: 10px;
		font-size: 0.74rem;
		color: var(--color-error);
	}

	.ok-btn {
		width: 100%;
		margin-top: 18px;
		padding: 11px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: var(--accent);
		color: var(--accent-text);
		font-weight: 700;
		font-size: 0.85rem;
		cursor: pointer;
	}

	.ok-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
