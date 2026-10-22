<script lang="ts">
	import MicrosoftSignIn from '$lib/components/auth/MicrosoftSignIn.svelte';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { addOfflineAccount } from '$lib/api';
	import type { MinecraftUser } from '$lib/types/types';
	import { User, Gamepad2, Loader2 } from 'lucide-svelte';
	import { t } from '$lib/i18n/index.svelte';

	let { onDone }: { onDone: (user: MinecraftUser) => void } = $props();

	let mode = $state<'choose' | 'offline' | 'microsoft'>('choose');
	let offlineName = $state('');
	let msError = $state<string | null>(null);
	let busy = $state(false);

	async function handleOffline() {
		if (!offlineName.trim()) return;
		busy = true;
		try {
			const user = await addOfflineAccount(offlineName.trim());
			onDone(user);
		} catch (e) {
			msError = String(e);
		} finally {
			busy = false;
		}
	}

	function handleMicrosoft() {
		mode = 'microsoft';
	}
</script>

<div class="onboarding">
	<div class="ambient-bg"></div>
	<div class="onboarding-card">
		<div class="brand">
			<div class="brand-mark"><Tfl width="34" height="34" /></div>
			<h1>TFL Client</h1>
			<p>{t('onboarding.tagline')}</p>
		</div>

		{#if mode === 'choose'}
			<div class="options">
				<button type="button" class="option-card primary" onclick={handleMicrosoft}>
					<span class="option-icon"><Gamepad2 size={20} /></span>
					<span class="option-text">
						<span class="option-title">{t('onboarding.microsoftTitle')}</span>
						<span class="option-sub">{t('onboarding.microsoftSub')}</span>
					</span>
				</button>
				<button type="button" class="option-card" onclick={() => (mode = 'offline')}>
					<span class="option-icon"><User size={20} /></span>
					<span class="option-text">
						<span class="option-title">{t('onboarding.offlineTitle')}</span>
						<span class="option-sub">{t('onboarding.offlineSub')}</span>
					</span>
				</button>
			</div>
		{:else if mode === 'offline'}
			<form
				class="offline-form"
				onsubmit={(e) => {
					e.preventDefault();
					handleOffline();
				}}
			>
				<label for="offline-username">{t('onboarding.usernameLabel')}</label>
				<input
					id="offline-username"
					type="text"
					bind:value={offlineName}
					placeholder="Steve"
					maxlength="16"
					autocomplete="off"
				/>
				<p class="hint">{t('onboarding.offlineHint')}</p>
				<div class="form-actions">
					<button type="button" class="option-btn" onclick={() => (mode = 'choose')}>
						{t('common.back')}
					</button>
					<button type="submit" class="option-btn primary" disabled={busy || !offlineName.trim()}>
						{#if busy}<Loader2 size={16} class="spin" />{/if}
						{t('common.continue')}
					</button>
				</div>
				{#if msError}<p class="error" role="alert">{msError}</p>{/if}
			</form>
		{:else if mode === 'microsoft'}
			<MicrosoftSignIn {onDone} onCancel={() => (mode = 'choose')} />
		{/if}
	</div>
</div>

<style>
	.onboarding {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		width: 100%;
		overflow: hidden;
	}

	.onboarding-card {
		position: relative;
		z-index: 1;
		width: 400px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 36px;
		box-shadow: var(--shadow-lg);
	}

	.brand {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		margin-bottom: 30px;
		text-align: center;
	}

	.brand-mark {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 64px;
		height: 64px;
		border-radius: var(--border-radius-lg);
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent);
		margin-bottom: 4px;
	}

	.brand-mark :global(svg) {
		color: var(--accent);
	}

	.brand h1 {
		font-family: var(--font-brand);
		font-size: var(--text-xl);
	}

	.brand p {
		color: var(--text-secondary);
		font-size: 0.82rem;
	}

	.options,
	.offline-form,
	.option-card {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px 16px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
		transition:
			border-color 0.15s,
			transform 0.12s;
	}

	.option-card:hover {
		border-color: var(--accent);
		transform: translateY(-1px);
	}

	.option-card.primary {
		border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
		background: color-mix(in srgb, var(--accent) 10%, var(--bg-input));
	}

	.option-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 38px;
		height: 38px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
		color: var(--accent);
	}

	.option-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.option-title {
		font-size: 0.88rem;
		font-weight: 700;
	}

	.option-sub {
		font-size: 0.72rem;
		color: var(--text-muted);
	}

	.option-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 11px 16px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-weight: 600;
		font-size: 0.85rem;
		cursor: pointer;
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.option-btn:hover:not(:disabled) {
		border-color: var(--text-muted);
	}

	.option-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.option-btn.primary {
		background: var(--accent);
		color: var(--accent-text);
		border-color: var(--accent);
	}

	.offline-form label {
		font-size: 0.75rem;
		color: var(--text-secondary);
	}

	.offline-form input {
		padding: 10px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.9rem;
	}

	.hint {
		font-size: 0.72rem;
		color: var(--text-muted);
	}

	.error {
		font-size: 0.78rem;
		color: var(--color-error);
	}

	.form-actions {
		display: flex;
		gap: 8px;
	}

	.form-actions .option-btn {
		flex: 1;
	}

	:global(.spin) {
		animation: tfl-spin 0.8s linear infinite;
	}

	@keyframes tfl-spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	.onboarding {
		background: var(--surface-canvas);
	}
	.onboarding-card {
		width: min(720px, calc(100vw - 64px));
		max-height: calc(100vh - 80px);
		overflow: auto;
		padding: 40px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		background: var(--surface-card);
		box-shadow: var(--shadow-lg);
	}
	.brand {
		margin-bottom: 28px;
	}
	.brand h1 {
		font-family: var(--font-family);
		font-size: 2.5rem;
		letter-spacing: -0.06em;
	}
	.option-card {
		min-height: 84px;
		border-radius: var(--border-radius);
		padding: 20px;
	}
	.option-title {
		font-size: 1.1rem;
	}
	.option-sub {
		font-size: var(--text-small);
		line-height: 1.5;
	}
	.offline-form input {
		min-height: 44px;
	}
	.brand-mark {
		border-radius: var(--border-radius);
	}
	@media (max-height: 650px) {
		.onboarding-card {
			padding: 26px;
		}
		.brand {
			margin-bottom: 18px;
		}
		.brand h1 {
			font-size: 2rem;
		}
		.option-card {
			min-height: 70px;
		}
	}
</style>
