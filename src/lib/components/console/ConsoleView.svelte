<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { Settings2, Copy, X } from 'lucide-svelte';
	import { w } from '$lib/i18n/workbench';
	import { severity, type ProcessStats, type Severity } from '$lib/console/model';
	import { consolePrefs as p, watchConsole } from '$lib/console/preferences.svelte';
	import ProcessCharts from './ProcessCharts.svelte';
	import ConsoleSettings from './ConsoleSettings.svelte';
	import { modalFocus } from '$lib/visual/modalFocus';
	let {
		instanceName,
		readStats,
		running = true,
		lines,
		empty
	}: {
		instanceName: string;
		readStats: () => Promise<ProcessStats | null>;
		running?: boolean;
		lines: { stream: string; line: string }[];
		empty: string;
	} = $props();
	let query = $state(''),
		level = $state('all'),
		customize = $state(false),
		copyState = $state<'copy' | 'copied' | 'copyFailed'>('copy');
	let logEl: HTMLDivElement | undefined;
	let follow = $derived(p.follow);
	let copyTimer: ReturnType<typeof setTimeout> | undefined;
	const shown = $derived(
		lines
			.map((entry) => ({ ...entry, severity: severity(entry.line, entry.stream) }))
			.filter(
				(entry) =>
					(level === 'all' || entry.severity === level) &&
					(!query || entry.line.toLowerCase().includes(query.toLowerCase()))
			)
	);
	onMount(() => {
		const clean = watchConsole();
		return () => {
			clean();
			clearTimeout(copyTimer);
		};
	});
	$effect(() => {
		void shown.length;
		if (follow)
			void tick().then(() => {
				if (logEl && follow) logEl.scrollTop = logEl.scrollHeight;
			});
	});
	async function copy() {
		try {
			await writeText(shown.map((e) => e.line).join('\n'));
			copyState = 'copied';
		} catch {
			copyState = 'copyFailed';
		}
		clearTimeout(copyTimer);
		copyTimer = setTimeout(() => (copyState = 'copy'), 2000);
	}
	function scroll() {
		if (logEl) follow = logEl.scrollHeight - logEl.scrollTop - logEl.clientHeight < 40;
	}
	const levels: Severity[] = ['info', 'warn', 'error', 'debug'];
</script>

<div class="console-view" style:--log-size={`${p.fontSize}px`}>
	<div class="toolbar">
		<input
			type="search"
			bind:value={query}
			placeholder={w('filter')}
			aria-label={w('filter')}
		/><select bind:value={level} aria-label={w('log')}
			><option value="all">{w('all')}</option>{#each levels as l (l)}<option value={l}
					>{w(l)}</option
				>{/each}</select
		><button type="button" onclick={copy} title={w(copyState)} aria-label={w(copyState)}
			><Copy size={15} /></button
		><button
			type="button"
			onclick={() => (customize = true)}
			aria-label={w('settings')}
			title={w('settings')}><Settings2 size={15} /></button
		>
	</div>
	<div class="console-layout" class:side={p.dock === 'side'} class:bottom={p.dock === 'bottom'}>
		<div class="charts-slot"><ProcessCharts {instanceName} {readStats} {running} /></div>
		<div class="log-slot">
			<!-- svelte-ignore a11y_no_noninteractive_tabindex (scrollable log must be keyboard accessible) -->
			<div
				class="log"
				class:wrap={p.wrap}
				bind:this={logEl}
				onscroll={scroll}
				role="region"
				aria-label={w('log')}
				tabindex="0"
				style:--log-info={p.colors.info}
				style:--log-warn={p.colors.warn}
				style:--log-error={p.colors.error}
				style:--log-debug={p.colors.debug}
			>
				{#each shown as entry, i (i)}<div class={`line ${entry.severity}`}>
						<span class="level" aria-label={w(entry.severity)}>{entry.severity.toUpperCase()}</span
						><span>{entry.line}</span>
					</div>{/each}
				{#if !lines.length}<p class="empty">{empty}</p>{:else if !shown.length}<p class="empty">
						0 / {lines.length}
					</p>{/if}
			</div>
			<label class="follow"
				><input type="checkbox" bind:checked={follow} />{w('follow')}<small
					>{shown.length} / {lines.length}</small
				></label
			>
		</div>
	</div>
</div>
{#if customize}
	<div
		class="config-backdrop"
		role="presentation"
		onclick={(e) => {
			if (e.target === e.currentTarget) customize = false;
		}}
	>
		<div
			class="config-dialog"
			role="dialog"
			aria-modal="true"
			aria-label={w('settings')}
			tabindex="-1"
			use:modalFocus={() => (customize = false)}
			onkeydown={(e) => {
				if (e.key === 'Escape') {
					e.stopPropagation();
					customize = false;
				}
			}}
		>
			<header>
				<h2>{w('settings')}</h2>
				<button type="button" onclick={() => (customize = false)} aria-label={w('close')}
					><X size={18} /></button
				>
			</header>
			<div class="config-body"><ConsoleSettings /></div>
		</div>
	</div>
{/if}

<style>
	.console-view {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-height: 0;
		min-width: 0;
		color: var(--text-primary, #e4e6eb);
		overflow: hidden;
	}
	.toolbar {
		display: flex;
		gap: 6px;
		padding: 8px;
		border-bottom: 1px solid var(--border, #34363d);
		flex-shrink: 0;
	}
	.toolbar input {
		flex: 1;
		width: 100px;
		min-width: 0;
	}
	input,
	select,
	button {
		background: var(--bg-input, #17191f);
		color: var(--text-primary, #e4e6eb);
		border: 1px solid var(--border, #34363d);
		border-radius: var(--border-radius-sm, 4px);
		padding: 7px;
		font-size: 12px;
	}
	button {
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}
	.console-layout {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}
	.charts-slot {
		flex-shrink: 0;
		min-width: 0;
	}
	.bottom .charts-slot {
		order: 2;
	}
	.side {
		flex-direction: row;
	}
	.side .charts-slot {
		width: 36%;
		max-width: 360px;
		overflow: auto;
		max-height: 100%;
	}
	.side .charts-slot :global(.process-charts) {
		grid-template-columns: 1fr;
		max-height: none;
	}
	.log-slot {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
		min-height: 0;
	}
	.log {
		overflow: auto;
		flex: 1;
		min-height: 0;
		padding: 10px 12px;
		background: #101216;
		font-family: ui-monospace, 'SF Mono', 'Cascadia Code', monospace;
		font-size: var(--log-size);
		line-height: 1.65;
		user-select: text;
		color: var(--log-info);
	}
	.line {
		display: flex;
		align-items: baseline;
		gap: 10px;
		white-space: pre;
		min-width: max-content;
		border-left: 2px solid transparent;
		padding-left: 6px;
	}
	.wrap .line {
		white-space: pre-wrap;
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.line > span:last-child {
		min-width: 0;
	}
	.warn {
		color: var(--log-warn);
		border-color: var(--log-warn);
		background: color-mix(in srgb, var(--log-warn) 5%, transparent);
	}
	.error {
		color: var(--log-error);
		border-color: var(--log-error);
		background: color-mix(in srgb, var(--log-error) 6%, transparent);
	}
	.debug {
		color: var(--log-debug);
	}
	.level {
		font-size: 0.8em;
		min-width: 4ch;
		opacity: 0.85;
		user-select: none;
	}
	.empty {
		color: #a1a1aa;
	}
	.follow {
		display: flex;
		gap: 6px;
		align-items: center;
		font-size: 11px;
		padding: 6px 12px;
		border-top: 1px solid var(--border, #34363d);
		flex-shrink: 0;
	}
	.follow small {
		margin-left: auto;
		opacity: 0.6;
	}
	input[type='checkbox'] {
		accent-color: var(--accent);
	}
	.config-backdrop {
		position: fixed;
		inset: 0;
		background: #0009;
		display: grid;
		place-items: center;
		z-index: 160;
		padding: 20px;
	}
	.config-dialog {
		display: flex;
		flex-direction: column;
		width: min(650px, 100%);
		max-height: 90vh;
		background: var(--surface-elevated, #1b1d23);
		border: 1px solid var(--border, #34363d);
		border-radius: var(--border-radius-lg, 8px);
		min-height: 0;
		color: var(--text-primary, #e4e6eb);
	}
	.config-dialog header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 16px 20px;
		border-bottom: 1px solid var(--border);
	}
	h2 {
		font-size: 18px;
		margin: 0;
	}
	.config-body {
		overflow: auto;
		padding: 20px;
	}
	@media (max-width: 760px) {
		.side {
			flex-direction: column;
		}
		.side .charts-slot {
			width: 100%;
			max-width: none;
			max-height: 36vh;
		}
		.side .charts-slot :global(.process-charts) {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
		.toolbar {
			flex-wrap: wrap;
		}
	}
</style>
