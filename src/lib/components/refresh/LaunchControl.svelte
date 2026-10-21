<script lang="ts">
	import { Play, LoaderCircle } from 'lucide-svelte';
	import { launchInstance, updateSettings } from '$lib/api';
	import { appState } from '$lib/state/state.svelte';
	import { gameSession } from '$lib/state/gameSession.svelte';
	import { multiInstanceWarningKind } from '$lib/state/launch';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { vr } from '$lib/i18n/refresh';
	import type { InstanceData } from '$lib/types/types';
	let { instance }: { instance: InstanceData } = $props();
	let busy = $state(false),
		error = $state(''),
		confirm = $state(false),
		dontAsk = $state(false);
	const warning = $derived(
		multiInstanceWarningKind(
			gameSession.running.filter((i) => i.name !== instance.name),
			appState.currentUser?.uuid
		)
	);
	async function launch() {
		busy = true;
		error = '';
		try {
			await launchInstance(instance.name);
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
	async function play() {
		if (warning && !appState.settings?.multi_instance_warning_dismissed) {
			confirm = true;
			return;
		}
		await launch();
	}
	async function proceed() {
		confirm = false;
		if (dontAsk && appState.settings) {
			try {
				await updateSettings({ ...appState.settings, multi_instance_warning_dismissed: true });
				appState.settings.multi_instance_warning_dismissed = true;
			} catch {
				/* A failed preference write must not block launch. */
			}
		}
		await launch();
	}
</script>

<button
	class="vr-control primary"
	disabled={busy}
	aria-label={`${vr('play')} ${instance.name}`}
	onclick={play}
	>{#if busy}<LoaderCircle size={16} />{:else}<Play size={16} fill="currentColor" />{/if}{vr(
		'play'
	)}</button
>
{#if error}<p class="launch-error" role="alert">{error}</p>{/if}
{#if confirm}<ConfirmDialog
		title={t('instanceDetail.multiInstanceTitle')}
		message={warning === 'same-account'
			? t('instanceDetail.multiInstanceSameAccount')
			: t('instanceDetail.multiInstanceDifferentAccount')}
		confirmLabel={t('instanceDetail.multiInstanceProceed')}
		checkboxLabel={t('instanceDetail.multiInstanceDontAskAgain')}
		bind:checked={dontAsk}
		onConfirm={proceed}
		onCancel={() => (confirm = false)}
	/>{/if}

<style>
	.launch-error {
		color: var(--color-error);
		max-width: 240px;
		font-size: var(--text-small);
		overflow-wrap: anywhere;
	}
</style>
