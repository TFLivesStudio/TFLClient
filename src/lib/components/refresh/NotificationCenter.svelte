<script lang="ts">
	import { Bell, CheckCircle2, AlertCircle, Info, X } from 'lucide-svelte';
	import { notifications } from '$lib/visual/notifications.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { i18nState } from '$lib/i18n/index.svelte';
	import { modalFocus } from '$lib/visual/modalFocus';
	let open = $state(false);
	const unread = $derived(notifications.items.filter((n) => !n.read).length);
	function toggle() {
		open = !open;
		if (open) for (const item of notifications.items) item.read = true;
	}
	$effect(() => {
		const list = notifications.toasts;
		if (!list.length) return;
		const next = Math.min(...list.map((n) => n.time + 6000));
		const timer = setTimeout(
			() => {
				notifications.toasts = notifications.toasts.filter((n) => n.time + 6000 > Date.now());
			},
			Math.max(0, next - Date.now())
		);
		return () => clearTimeout(timer);
	});
</script>

<button
	class="notice-trigger"
	aria-label={vr('notifications')}
	aria-expanded={open}
	onclick={toggle}
	><Bell size={17} />{#if unread}<span class="notice-dot">{unread}</span>{/if}</button
>
{#if open}<div
		class="notice-scrim"
		role="presentation"
		onclick={(e) => {
			if (e.target === e.currentTarget) open = false;
		}}
	>
		<div
			class="notice-center"
			role="dialog"
			aria-modal="true"
			aria-label={vr('notifications')}
			use:modalFocus={() => (open = false)}
		>
			<header>
				<div>
					<h2>{vr('notifications')}</h2>
					<p>{vr('notificationsRetention')}</p>
				</div>
				<button class="vr-control" aria-label={vr('cancel')} onclick={() => (open = false)}
					><X size={17} /></button
				>
			</header>
			<div class="notice-list">
				{#each notifications.items as notice (notice.id)}<article
						class:error={notice.kind === 'error'}
					>
						{#if notice.kind === 'success'}<CheckCircle2
								size={19}
							/>{:else if notice.kind === 'error'}<AlertCircle size={19} />{:else}<Info
								size={19}
							/>{/if}
						<div>
							<strong>{vr(notice.title)}</strong>
							<p>{notice.message}</p>
							<small>{new Date(notice.time).toLocaleTimeString(i18nState.locale)}</small>
						</div>
					</article>{:else}<div class="notice-empty">
						<CheckCircle2 size={36} />
						<h3>{vr('noNotifications')}</h3>
						<p>{vr('noNotificationsHint')}</p>
					</div>{/each}
			</div>
			{#if notifications.items.length}<button
					class="vr-control"
					onclick={() => {
						notifications.items = [];
						notifications.toasts = [];
					}}>{vr('clear')}</button
				>{/if}
		</div>
	</div>{/if}
<div class="notice-toasts" aria-live="polite" aria-atomic="false">
	{#each notifications.toasts as notice (notice.id)}<div
			class="notice-toast"
			class:error={notice.kind === 'error'}
		>
			<span
				>{#if notice.kind === 'error'}<AlertCircle size={19} />{:else}<CheckCircle2
						size={19}
					/>{/if}</span
			>
			<div>
				<strong>{vr(notice.title)}</strong>
				<p>{notice.message}</p>
			</div>
			<button
				aria-label={vr('cancel')}
				onclick={() =>
					(notifications.toasts = notifications.toasts.filter((n) => n.id !== notice.id))}
				><X size={15} /></button
			>
		</div>{/each}
</div>

<style>
	.notice-trigger {
		position: relative;
		display: grid;
		place-items: center;
		width: 40px;
		height: 32px;
		border: 0;
		background: none;
		color: var(--text-secondary);
	}
	.notice-dot {
		position: absolute;
		top: 0;
		right: 0;
		min-width: 14px;
		height: 14px;
		border-radius: 8px;
		background: var(--accent);
		color: var(--accent-text);
		font-size: 9px;
		display: grid;
		place-items: center;
	}
	.notice-scrim {
		position: fixed;
		inset: 42px 0 0;
		background: #0004;
		z-index: var(--z-modal);
		display: flex;
		justify-content: flex-end;
		align-items: flex-start;
		padding: 16px;
	}
	.notice-center {
		width: 410px;
		max-width: calc(100vw - 32px);
		max-height: calc(100vh - 80px);
		overflow: auto;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		background: var(--surface-elevated);
		padding: 22px;
	}
	.notice-center header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 22px;
	}
	.notice-center h2 {
		font-size: 1.3rem;
		margin: 0 0 8px;
	}
	.notice-center p {
		font-size: var(--text-small);
		line-height: 1.5;
		color: var(--text-secondary);
	}
	.notice-list article {
		display: flex;
		gap: 12px;
		padding: 16px 0;
		border-top: 1px solid var(--border);
		color: var(--accent);
	}
	.notice-list article > div {
		min-width: 0;
		flex: 1;
		color: var(--text-primary);
	}
	.notice-list p {
		margin: 8px 0;
		overflow-wrap: anywhere;
	}
	.notice-list small {
		color: var(--text-muted);
	}
	.notice-empty {
		padding: 28px 6px;
		text-align: center;
		color: var(--text-muted);
	}
	.notice-empty h3 {
		color: var(--text-primary);
		margin: 14px 0;
	}
	.notice-toasts {
		position: fixed;
		right: 20px;
		top: 62px;
		z-index: var(--z-toast);
		display: flex;
		flex-direction: column;
		gap: 10px;
		pointer-events: none;
		width: min(380px, calc(100vw - 40px));
	}
	.notice-toast {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		background: var(--surface-elevated);
		border: 1px solid var(--border);
		border-left: 3px solid var(--accent);
		border-radius: var(--border-radius);
		padding: 18px;
		box-shadow: var(--shadow-md);
		pointer-events: auto;
		animation: vr-reveal var(--duration-panel) var(--easing-decelerate);
	}
	.notice-toast > span {
		color: var(--accent);
	}
	.notice-toast > div {
		flex: 1;
		min-width: 0;
	}
	.notice-toast strong {
		font-size: var(--text-small);
	}
	.notice-toast p {
		font-size: var(--text-caption);
		color: var(--text-secondary);
		margin-top: 8px;
		overflow-wrap: anywhere;
	}
	.notice-toast button {
		background: none;
		border: 0;
		color: var(--text-muted);
	}
	.notice-toast.error {
		border-left-color: var(--color-error);
	}
</style>
