<script lang="ts">
	import { onMount } from 'svelte';
	import { getUserList, switchUser } from '$lib/api';
	import { appState } from '$lib/state/state.svelte';
	import { authText } from '$lib/i18n/auth';
	import { closeModal, openSettingsAt } from '$lib/state/uiState.svelte';
	import { vr } from '$lib/i18n/refresh';
	import type { MinecraftUser } from '$lib/types/types';
	import { UserRound, Check } from 'lucide-svelte';
	let { onChange }: { onChange: () => Promise<void> } = $props();
	let accounts = $state<MinecraftUser[]>([]),
		busy = $state(false),
		error = $state('');
	onMount(() => {
		getUserList()
			.then((users) => (accounts = users))
			.catch((e) => (error = String(e)));
	});
	async function select(uuid: string) {
		busy = true;
		error = '';
		try {
			appState.currentUser = await switchUser(uuid);
			await onChange();
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
</script>

<aside class="studio-accounts">
	<span class="vr-eyebrow">{vr('account')}</span>{#each accounts as account (account.uuid)}<button
			disabled={busy}
			aria-pressed={account.uuid === appState.currentUser?.uuid}
			onclick={() => select(account.uuid)}
			><UserRound size={20} /><span
				><strong>{account.username}</strong><small
					>{account.user_type === 'Cracked'
						? 'Offline'
						: account.session_only
							? authText('Sesión temporal', 'Temporary session')
							: account.user_type}</small
				></span
			>{#if account.uuid === appState.currentUser?.uuid}<Check size={15} />{/if}</button
		>{/each}{#if error}<p role="alert">{error}</p>{/if}
	<button
		type="button"
		class="session-manage"
		onclick={() => {
			closeModal('skinManager');
			openSettingsAt('accounts');
		}}>{authText('Gestionar sesiones', 'Manage sessions')}</button
	>
</aside>

<style>
	.studio-accounts {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.studio-accounts button {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 14px;
		background: var(--surface-interactive);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		color: var(--text-primary);
		text-align: left;
	}
	.studio-accounts button[aria-pressed='true'] {
		border-color: var(--accent);
	}
	.studio-accounts button > span {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}
	.studio-accounts small {
		color: var(--text-muted);
		font-size: var(--text-caption);
	}
	.studio-accounts p {
		color: var(--color-error);
		font-size: var(--text-small);
	}
</style>
