<script lang="ts">
	import { w } from '$lib/i18n/workbench';
	import type { ConsolePreferences, Severity } from '$lib/console/model';
	let {
		p,
		shown,
		total,
		empty,
		element = $bindable(),
		onScroll
	}: {
		p: ConsolePreferences;
		shown: { line: string; severity: Severity }[];
		total: number;
		empty: string;
		element?: HTMLDivElement;
		onScroll?: () => void;
	} = $props();
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex (scrollable log must be keyboard accessible) -->
<div
	class="log"
	class:wrap={p.wrap}
	bind:this={element}
	onscroll={onScroll}
	role="region"
	aria-label={w('log')}
	tabindex="0"
	style:--log-info={p.colors.info}
	style:--log-warn={p.colors.warn}
	style:--log-error={p.colors.error}
	style:--log-debug={p.colors.debug}
>
	{#each shown as entry, i (i)}<div class={`line ${entry.severity}`}>
			<span class="level" aria-label={w(entry.severity)}>{entry.severity.toUpperCase()}</span><span
				>{entry.line}</span
			>
		</div>{/each}
	{#if !total}<p class="empty">{empty}</p>{:else if !shown.length}<p class="empty">
			0 / {total}
		</p>{/if}
</div>

<style>
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
</style>
