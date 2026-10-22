<script lang="ts">
	import { Activity, CheckCircle2, AlertTriangle, Clock, Package } from 'lucide-svelte';
	import { activityState, resyncActivities, cancelActivity } from '$lib/activity/state.svelte';
	import { terminal, type ActivityTask } from '$lib/activity/contract';
	import { al } from '$lib/activity/labels';
	import { appState } from '$lib/state/state.svelte';
	let cancelError = $state(false);
	let pending = $state<string | null>(null);
	const active = $derived(
		activityState.snapshot.tasks
			.filter((t) => !terminal(t.status))
			.sort((a, b) => a.created_at - b.created_at || a.id.localeCompare(b.id))
	);
	const history = $derived(activityState.snapshot.tasks.filter((t) => terminal(t.status)));
	const time = (value: number) => new Date(value * 1000).toLocaleString();
	const bytes = (value: number) =>
		`${(value / 1048576).toLocaleString(undefined, { maximumFractionDigits: 1 })} MiB`;
	const targetName = (task: ActivityTask) =>
		appState.instances.find((i) => i.uuid === task.instance_id)?.name;
	async function cancel(task: ActivityTask) {
		pending = task.id;
		cancelError = false;
		try {
			await cancelActivity(task.id);
		} catch {
			cancelError = true;
		} finally {
			pending = null;
			await resyncActivities();
		}
	}
</script>

<div class="activity-center">
	<header>
		<div class="eyebrow"><Activity size={15} /> TFL CLIENT</div>
		<h1>{al('title')}</h1>
		<p>{al('subtitle')}</p>
	</header>
	{#if activityState.error}<div class="notice" role="status">
			<AlertTriangle size={18} /><span>{al('syncError')}</span><button
				onclick={() => resyncActivities()}>{al('retry')}</button
			>
		</div>{/if}
	{#if activityState.snapshot.storage_warning}<p class="notice" role="status">
			{al('storage')}
		</p>{/if}
	{#if cancelError}<p class="notice" role="alert">{al('cancelError')}</p>{/if}
	{#if activityState.loading}<p role="status" class="empty">{al('loading')}</p>
	{:else if activityState.error && activityState.snapshot.tasks.length === 0}
		<p class="history-empty">{al('waitingSync')}</p>
	{:else}
		<section aria-label={al('active')}>
			<div class="section-heading">
				<h2>{al('active')}</h2>
				<span class="counter">{active.length}</span>
			</div>
			{#if active.length === 0}<div class="empty">
					<CheckCircle2 size={28} />
					<h3>{al('empty')}</h3>
					<p>{al('emptyHint')}</p>
				</div>{/if}
			{#each active as task (task.id)}{@render card(task)}{/each}
		</section>
		<section aria-label={al('history')}>
			<div class="section-heading">
				<h2>{al('history')}</h2>
				<Clock size={16} />
			</div>
			{#if history.length === 0}<p class="history-empty">{al('noHistory')}</p>{/if}
			{#each history as task (task.id)}{@render card(task)}{/each}
		</section>
	{/if}
</div>

{#snippet card(task: ActivityTask)}
	<article class="task" class:failed={task.status === 'failed' || task.status === 'interrupted'}>
		<div class="task-top">
			<span class="task-icon"><Package size={20} /></span>
			<div class="task-title">
				<h3>
					{al(task.target)}{targetName(task) ? ` · ${targetName(task)}` : ''}
				</h3>
				<span>{al(task.source)} · {time(task.created_at)}</span>
			</div>
			<span class="status" class:success={task.status === 'succeeded'}>{al(task.status)}</span>
		</div>
		{#if !terminal(task.status)}
			<p class="stage">{al(task.stage)}</p>
			{#if task.progress}<progress
					max={task.progress.total}
					value={task.progress.current}
					aria-label={al('downloading')}
				></progress>
				<p class="progress-label">
					{bytes(task.progress.current)} / {bytes(task.progress.total)} · {al('bytes')}
				</p>
			{:else}<div class="indeterminate" role="progressbar" aria-label={al('unknown')}></div>
				<p class="progress-label">{al('unknown')}</p>{/if}
			{#if task.cancellable}<button
					disabled={pending === task.id || task.status === 'cancel_requested'}
					onclick={() => cancel(task)}>{al('cancel')}</button
				>{:else}<p class="cancel-hint">{al('cannotCancel')}</p>{/if}
		{/if}
		{#if task.error}<p class="failure">
				{al(task.error === 'interrupted' ? 'interruptedHint' : 'install_failed')}
			</p>{/if}
		<details>
			<summary>{al('details')}</summary>
			<dl>
				{#if task.project_id}<dt>Modrinth</dt>
					<dd>{task.project_id}</dd>{/if}
				<dt>{al('id')}</dt>
				<dd>{task.id}</dd>
				<dt>{al('revision')}</dt>
				<dd>{task.revision}</dd>
				<dt>{al('updated')}</dt>
				<dd>{time(task.updated_at)}</dd>
			</dl>
		</details>
	</article>
{/snippet}

<style>
	.activity-center {
		max-width: 960px;
		margin: 0 auto;
		padding: clamp(20px, 4vw, 44px);
		color: var(--text-primary);
	}
	header {
		margin-bottom: 28px;
	}
	.eyebrow {
		display: flex;
		gap: 8px;
		align-items: center;
		font-size: 11px;
		letter-spacing: 2px;
		color: var(--accent);
		font-weight: 700;
	}
	h1 {
		font-size: clamp(26px, 4vw, 36px);
		margin: 12px 0 8px;
	}
	header p,
	.empty p {
		color: var(--text-muted);
		line-height: 1.6;
	}
	section {
		margin-top: 26px;
	}
	.section-heading {
		display: flex;
		align-items: center;
		gap: 10px;
		color: var(--text-muted);
		margin-bottom: 12px;
	}
	h2 {
		font-size: 14px;
		color: var(--text-primary);
		margin: 0;
	}
	.counter {
		padding: 2px 7px;
		background: var(--surface-elevated);
		border-radius: var(--border-radius-sm);
		font-size: 12px;
	}
	.task {
		background: var(--surface-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		padding: 18px;
		margin-bottom: 10px;
		min-width: 0;
	}
	.task-top {
		display: flex;
		align-items: center;
		gap: 12px;
		flex-wrap: wrap;
	}
	.task-icon {
		display: grid;
		place-items: center;
		width: 38px;
		height: 38px;
		background: color-mix(in srgb, var(--accent) 12%, transparent);
		color: var(--accent);
		border-radius: var(--border-radius-sm);
	}
	.task-title {
		flex: 1;
		min-width: 160px;
	}
	h3 {
		font-size: 14px;
		margin: 0 0 5px;
	}
	.task-title span {
		font-size: 11px;
		color: var(--text-muted);
	}
	.status {
		font-size: 12px;
		padding: 5px 9px;
		background: var(--surface-elevated);
		border-radius: var(--border-radius-sm);
	}
	.success {
		color: var(--text-primary);
		background: color-mix(in srgb, var(--color-success) 14%, var(--surface-elevated));
	}
	.failed .status {
		color: var(--text-primary);
		background: color-mix(in srgb, var(--color-warning, #f59e0b) 14%, var(--surface-elevated));
	}
	.stage {
		font-size: 13px;
		margin: 18px 0 10px;
	}
	progress {
		width: 100%;
		height: 6px;
		accent-color: var(--accent);
	}
	.indeterminate {
		height: 6px;
		background: repeating-linear-gradient(90deg, var(--accent) 0 12px, transparent 12px 20px);
		opacity: 0.5;
		border-radius: var(--border-radius-sm);
	}
	.progress-label,
	.cancel-hint {
		font-size: 11px;
		color: var(--text-muted);
		line-height: 1.5;
		margin-top: 8px;
	}
	.failure {
		font-size: 13px;
		line-height: 1.6;
		color: var(--text-secondary);
	}
	details {
		margin-top: 14px;
		border-top: 1px solid var(--border);
		padding-top: 10px;
		font-size: 11px;
		color: var(--text-muted);
	}
	summary {
		cursor: pointer;
		width: fit-content;
	}
	dl {
		display: grid;
		grid-template-columns: 100px minmax(0, 1fr);
		gap: 8px;
		margin-bottom: 0;
	}
	dd {
		margin: 0;
		overflow-wrap: anywhere;
	}
	dt {
		font-weight: 600;
	}
	.empty {
		display: flex;
		align-items: center;
		flex-direction: column;
		text-align: center;
		padding: 32px 20px;
		background: var(--surface-card);
		border: 1px dashed var(--border);
		border-radius: var(--border-radius);
	}
	.empty :global(svg) {
		color: var(--accent);
		margin-bottom: 12px;
	}
	.empty p {
		font-size: 13px;
		max-width: 360px;
		margin: 4px 0;
	}
	.history-empty {
		font-size: 13px;
		color: var(--text-muted);
		padding: 14px 0;
	}
	.notice {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
		padding: 14px;
		margin-bottom: 12px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		color: var(--text-primary);
		background: var(--surface-card);
		font-size: 13px;
		line-height: 1.6;
	}
	button {
		padding: 8px 12px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--surface-elevated);
		color: var(--text-primary);
		cursor: pointer;
	}
	button:disabled {
		opacity: 0.5;
		cursor: default;
	}
	button:focus-visible,
	summary:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 4px;
	}
	:global([data-visual='pixel']) .task {
		box-shadow: 3px 3px 0 var(--border);
	}
	@media (prefers-reduced-motion: reduce) {
		* {
			animation: none !important;
			transition: none !important;
		}
	}
</style>
