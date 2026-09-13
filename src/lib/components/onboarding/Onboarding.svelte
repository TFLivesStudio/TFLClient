<script lang="ts">
	import Tfl from '$lib/icons/Tfl.svelte';
	import { getDeviceCode, authenticateWithDeviceCode, addOfflineAccount } from '$lib/api/tflApi';
	import type { MinecraftUser } from '$lib/types/types';
	import { User, Gamepad2, Loader2 } from 'lucide-svelte';

	let { onDone }: { onDone: (user: MinecraftUser) => void } = $props();

	let mode = $state<'choose' | 'offline' | 'microsoft'>('choose');
	let offlineName = $state('');
	let msCode = $state<string | null>(null);
	let msVerificationUri = $state<string | null>(null);
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

	async function handleMicrosoft() {
		mode = 'microsoft';
		busy = true;
		msError = null;
		try {
			const dc = await getDeviceCode();
			msCode = dc.user_code;
			msVerificationUri = dc.verification_uri;
			const user = await authenticateWithDeviceCode(dc.device_code, dc.interval, dc.expires_in);
			onDone(user);
		} catch (e) {
			msError = String(e);
		} finally {
			busy = false;
		}
	}
</script>

<div class="onboarding">
	<div class="onboarding-card">
		<div class="brand">
			<Tfl width="40" height="40" />
			<h1>TFL Client</h1>
			<p>Iniciá sesión para empezar a jugar</p>
		</div>

		{#if mode === 'choose'}
			<div class="options">
				<button type="button" class="option-btn primary" onclick={handleMicrosoft}>
					<Gamepad2 size={18} />
					Iniciar con Microsoft
				</button>
				<button type="button" class="option-btn" onclick={() => (mode = 'offline')}>
					<User size={18} />
					Cuenta offline
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
				<label for="offline-username">Nombre de usuario</label>
				<input
					id="offline-username"
					type="text"
					bind:value={offlineName}
					placeholder="Steve"
					maxlength="16"
					autocomplete="off"
				/>
				<p class="hint">Las cuentas offline solo pueden jugar en singleplayer.</p>
				<div class="form-actions">
					<button type="button" class="option-btn" onclick={() => (mode = 'choose')}>
						Volver
					</button>
					<button type="submit" class="option-btn primary" disabled={busy || !offlineName.trim()}>
						{#if busy}<Loader2 size={16} class="spin" />{/if}
						Continuar
					</button>
				</div>
			</form>
		{:else if mode === 'microsoft'}
			<div class="ms-flow">
				{#if busy && !msCode}
					<Loader2 size={24} class="spin" />
					<p>Obteniendo código...</p>
				{:else if msCode}
					<p>Andá a</p>
					<a href={msVerificationUri} target="_blank" rel="noreferrer">{msVerificationUri}</a>
					<p>e ingresá el código:</p>
					<div class="code">{msCode}</div>
					<p class="hint">Esperando confirmación...</p>
				{/if}
				{#if msError}
					<p class="error">{msError}</p>
				{/if}
				<button
					type="button"
					class="option-btn"
					onclick={() => {
						mode = 'choose';
						msCode = null;
						msError = null;
					}}
				>
					Cancelar
				</button>
			</div>
		{/if}
	</div>
</div>

<style>
	.onboarding {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		width: 100%;
	}

	.onboarding-card {
		width: 380px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 32px;
		box-shadow: var(--shadow-lg);
	}

	.brand {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		margin-bottom: 28px;
		text-align: center;
	}

	.brand :global(svg) {
		color: var(--accent);
	}

	.brand h1 {
		font-family: var(--font-brand);
		font-size: 1.1rem;
		margin-top: 8px;
	}

	.brand p {
		color: var(--text-secondary);
		font-size: 0.82rem;
	}

	.options,
	.offline-form,
	.ms-flow {
		display: flex;
		flex-direction: column;
		gap: 10px;
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

	.ms-flow {
		align-items: center;
		text-align: center;
	}

	.ms-flow a {
		color: var(--accent);
	}

	.code {
		font-family: var(--font-brand);
		font-size: 1.4rem;
		letter-spacing: 4px;
		background: var(--bg-input);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		padding: 10px 18px;
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
</style>
