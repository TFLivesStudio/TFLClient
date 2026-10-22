<script lang="ts">
	import { onMount } from 'svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { ShieldCheck, ExternalLink, KeyRound, Check, Copy, X, RotateCcw } from 'lucide-svelte';
	import {
		startMicrosoftAuth,
		microsoftAuthStatus,
		cancelMicrosoftAuth,
		microsoftMemorySession,
		microsoftRetryStorage,
		microsoftOpenDevice,
		type MicrosoftLogin,
		type AuthMethod
	} from '$lib/api/auth';
	import type { MinecraftUser } from '$lib/types/types';
	import { authText as text, authError, authPhase } from '$lib/i18n/auth';

	let {
		onDone,
		onCancel,
		expectedUuid
	}: {
		onDone: (user: MinecraftUser) => void | Promise<void>;
		onCancel: () => void;
		expectedUuid?: string;
	} = $props();
	let login = $state<MicrosoftLogin | null>(null);
	let localError = $state<string | null>(null);
	let starting = $state(false);
	let copied = $state(false);
	let active = $derived(
		!!login && !['success', 'error', 'cancelled', 'expired'].includes(login.phase)
	);
	let saving = $derived(login?.phase === 'saving');
	let alive = true;
	let finished = false;
	let generation = 0;
	let timer: ReturnType<typeof setTimeout> | undefined;
	let copiedTimer: ReturnType<typeof setTimeout> | undefined;

	function accept(snapshot: MicrosoftLogin) {
		if (!alive) return;
		login = snapshot;
		if (snapshot.phase === 'success' && snapshot.user && !finished) {
			finished = true;
			Promise.resolve(onDone(snapshot.user)).catch(() => {
				localError = text(
					'La cuenta está conectada. Cierra y vuelve a abrir este panel.',
					'The account is connected. Close and reopen this panel.'
				);
			});
		}
	}
	async function poll(id: string, version: number) {
		try {
			const snapshot = await microsoftAuthStatus(id);
			if (!alive || version !== generation) return;
			accept(snapshot);
			if (!['success', 'error', 'cancelled', 'expired'].includes(snapshot.phase))
				timer = setTimeout(() => poll(id, version), 500);
		} catch {
			if (alive && version === generation)
				localError = text(
					'No se pudo consultar el estado. Cancela y vuelve a intentarlo.',
					'Status could not be retrieved. Cancel and try again.'
				);
		}
	}
	async function begin(method: AuthMethod = 'auto') {
		if (starting || saving) return;
		starting = true;
		localError = null;
		clearTimeout(timer);
		const version = ++generation;
		try {
			if (login && active) await cancelMicrosoftAuth(login.id);
			const snapshot = await startMicrosoftAuth(method, expectedUuid);
			if (!alive) {
				await cancelMicrosoftAuth(snapshot.id);
				return;
			}
			accept(snapshot);
			void poll(snapshot.id, version);
		} catch (error) {
			if (alive) localError = authError(String(error));
		} finally {
			if (alive) starting = false;
		}
	}
	async function cancel() {
		if (saving || starting) return;
		try {
			if (login && active) accept(await cancelMicrosoftAuth(login.id));
			clearTimeout(timer);
			generation++;
			onCancel();
		} catch (error) {
			localError = authError(String(error));
		}
	}
	async function storage(memory: boolean) {
		if (!login || starting) return;
		starting = true;
		try {
			accept(await (memory ? microsoftMemorySession(login.id) : microsoftRetryStorage(login.id)));
		} catch (error) {
			localError = authError(String(error));
		} finally {
			starting = false;
		}
	}
	async function copyCode() {
		if (!login?.user_code) return;
		try {
			await writeText(login.user_code);
			copied = true;
			clearTimeout(copiedTimer);
			copiedTimer = setTimeout(() => (copied = false), 1800);
		} catch {
			localError = text('No se pudo copiar el código.', 'The code could not be copied.');
		}
	}
	async function openDevice() {
		if (!login) return;
		try {
			await microsoftOpenDevice(login.id);
		} catch (error) {
			localError = authError(String(error));
		}
	}
	onMount(() => {
		void begin();
		return () => {
			alive = false;
			clearTimeout(timer);
			clearTimeout(copiedTimer);
			if (login && active && !finished) void cancelMicrosoftAuth(login.id).catch(() => {});
		};
	});
</script>

<div
	class="microsoft-sign-in"
	aria-label={text('Iniciar sesión con Microsoft', 'Sign in with Microsoft')}
>
	<header>
		<div class="auth-mark"><ShieldCheck size={22} aria-hidden="true" /></div>
		<div>
			<h3>{text('Tu cuenta, con Microsoft', 'Your account, with Microsoft')}</h3>
			<p>
				{text(
					'Tu contraseña se introduce únicamente en la página oficial de Microsoft.',
					'Your password is entered only on Microsoft’s official page.'
				)}
			</p>
		</div>
	</header>
	<div class="auth-status" role="status" aria-live="polite" aria-atomic="true">
		<strong>{authPhase(login?.phase || 'preparing')}</strong>
		{#if login?.domain}<span class="auth-domain"
				><ShieldCheck size={14} aria-hidden="true" />{login.domain}</span
			>{/if}
	</div>
	{#if active && !['storage_required', 'saving'].includes(login?.phase || '')}
		<p class="auth-hint">
			{login?.method === 'embedded'
				? text(
						'Completa la autorización en la ventana de Microsoft. Si no carga o necesitas otro método de verificación, continúa en tu navegador.',
						'Complete authorization in the Microsoft window. If it does not load or you need another verification method, continue in your browser.'
					)
				: login?.method === 'external'
					? text(
							'Completa el inicio de sesión en tu navegador y vuelve al launcher. Esta ventana se actualizará automáticamente.',
							'Finish signing in in your browser and return to the launcher. This panel will update automatically.'
						)
					: text(
							'Abre Microsoft e introduce el código que aparece aquí. El launcher seguirá esperando tu autorización.',
							'Open Microsoft and enter the code shown here. The launcher will wait for your authorization.'
						)}
		</p>
	{/if}
	{#if login?.user_code}
		<p class="auth-hint">{login.verification_uri}</p>
		<div class="auth-device">
			<code aria-label={text('Código de dispositivo', 'Device code')}>{login.user_code}</code>
			<button type="button" disabled={starting} onclick={copyCode}
				>{#if copied}<Check size={16} />{:else}<Copy size={16} />{/if}{text(
					'Copiar',
					'Copy'
				)}</button
			>
			<button type="button" class="auth-primary" disabled={starting} onclick={openDevice}
				><ExternalLink size={16} />{text('Abrir Microsoft', 'Open Microsoft')}</button
			>
		</div>
	{/if}
	{#if login?.test_build}<p class="auth-notice">
			{text(
				'Compilación de prueba: la identidad Microsoft configurada no implica aprobación de Minecraft.',
				'Test build: the configured Microsoft identity does not imply Minecraft approval.'
			)}
		</p>{/if}
	{#if login?.error}<p class="auth-error" role="alert">{authError(login.error)}</p>{/if}
	{#if localError}<p class="auth-error" role="alert">{localError}</p>{/if}
	{#if login?.phase === 'storage_required'}
		<div class="auth-storage">
			<p>
				{text(
					'Una sesión temporal solo funciona hasta que cierres el launcher. No se guardarán sus tokens y tendrás que iniciar sesión de nuevo después.',
					'A temporary session lasts only until you close the launcher. Its tokens will not be saved and you will need to sign in again afterwards.'
				)}
			</p>
			<button type="button" disabled={starting} onclick={() => storage(false)}
				><RotateCcw size={16} />{text('Reintentar guardado seguro', 'Retry secure storage')}</button
			>
			<button type="button" disabled={starting} onclick={() => storage(true)}
				>{text('Usar sesión temporal', 'Use temporary session')}</button
			>
		</div>
	{:else if !active && !starting && login?.phase !== 'success'}
		<button type="button" class="auth-primary" onclick={() => begin()}
			><RotateCcw size={16} />{text('Reintentar', 'Retry')}</button
		>
	{/if}
	{#if login?.phase !== 'success'}
		<div class="auth-actions">
			<button type="button" disabled={starting || saving} onclick={() => begin('external')}
				><ExternalLink size={15} />{text('Usar navegador', 'Use browser')}</button
			>
			<button type="button" disabled={starting || saving} onclick={() => begin('device')}
				><KeyRound size={15} />{text('Usar código', 'Use device code')}</button
			>
			<button type="button" disabled={starting || saving} onclick={cancel}
				><X size={15} />{text('Cancelar', 'Cancel')}</button
			>
		</div>
	{/if}
	<p class="auth-footnote">
		{text(
			'La sesión se protege con el almacén seguro del sistema. Si no está disponible, podrás elegir una sesión temporal.',
			'The system secure store protects your session. If it is unavailable, you can choose a temporary session.'
		)}
	</p>
</div>

<style>
	.microsoft-sign-in {
		display: grid;
		gap: 16px;
		min-width: 0;
		text-align: left;
	}
	header {
		display: flex;
		align-items: flex-start;
		gap: 12px;
	}
	.auth-mark {
		display: grid;
		place-items: center;
		flex-shrink: 0;
		width: 44px;
		height: 44px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--surface-interactive);
		color: var(--accent);
	}
	h3 {
		margin: 0 0 5px;
		font-size: var(--text-body);
		color: var(--text-primary);
	}
	p {
		margin: 0;
		line-height: 1.6;
		color: var(--text-secondary);
		font-size: var(--text-small);
	}
	.auth-status {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		align-items: center;
		padding: 14px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--surface-interactive);
	}
	.auth-status strong {
		color: var(--text-primary);
		font-size: var(--text-small);
	}
	.auth-domain {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: var(--text-caption);
		color: var(--accent);
		overflow-wrap: anywhere;
	}
	.auth-device,
	.auth-actions,
	.auth-storage {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		align-items: center;
	}
	.auth-device code {
		font: 700 1.4rem var(--font-mono, monospace);
		letter-spacing: 0.12em;
		padding: 12px 16px;
		border: 1px solid var(--accent);
		border-radius: var(--border-radius-sm);
		color: var(--text-primary);
		background: var(--surface-interactive);
		user-select: text;
	}
	button {
		display: inline-flex;
		gap: 8px;
		align-items: center;
		justify-content: center;
		min-height: 40px;
		padding: 9px 12px;
		font: inherit;
		font-size: var(--text-small);
		font-weight: 600;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--surface-interactive);
		color: var(--text-primary);
		cursor: pointer;
	}
	button:hover:not(:disabled) {
		border-color: var(--accent);
	}
	button:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 3px;
	}
	button:disabled {
		opacity: 0.5;
		cursor: default;
	}
	.auth-primary {
		background: var(--accent);
		color: var(--accent-text);
		border-color: var(--accent);
	}
	.auth-error {
		padding: 12px;
		border-left: 3px solid var(--color-error);
		color: var(--color-error);
		background: var(--surface-interactive);
	}
	.auth-notice {
		padding: 10px;
		border: 1px solid var(--border);
	}
	.auth-footnote {
		color: var(--text-muted);
		font-size: var(--text-caption);
	}
	.auth-storage p {
		flex-basis: 100%;
	}
	:global([data-visual='pixel']) .auth-mark {
		box-shadow:
			inset -3px -3px 0 rgb(0 0 0 / 0.25),
			inset 3px 3px 0 rgb(255 255 255 / 0.08);
	}
</style>
