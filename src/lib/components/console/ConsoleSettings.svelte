<script lang="ts">
	import { reorderHandle } from '$lib/console/reorder';
	import { onMount } from 'svelte';
	import { ArrowUp, ArrowDown, GripVertical, RotateCcw } from 'lucide-svelte';
	import { w } from '$lib/i18n/workbench';
	import { DEFAULT_CONSOLE, type Metric, type Severity } from '$lib/console/model';
	import {
		consolePrefs as p,
		consoleStorage,
		saveConsole,
		watchConsole,
		moveMetric,
		dropMetric
	} from '$lib/console/preferences.svelte';
	let dragged = $state<Metric | null>(null);
	const levels: Severity[] = ['info', 'warn', 'error', 'debug'];
	onMount(watchConsole);
</script>

<div class="console-settings">
	<section>
		<h3>{w('charts')}</h3>
		<p>{w('chartHint')}</p>
		<label
			>{w('position')}<select
				value={p.dock}
				onchange={(e) => saveConsole({ dock: e.currentTarget.value as typeof p.dock })}
			>
				{#each ['top', 'bottom', 'side'] as dock (dock)}<option value={dock}
						>{w(dock as 'top' | 'bottom' | 'side')}</option
					>{/each}</select
			></label
		>
		<div class="fields">
			<label
				>{w('columns')}<select
					value={p.columns}
					onchange={(e) => saveConsole({ columns: Number(e.currentTarget.value) as 1 | 2 | 3 })}
					>{#each [1, 2, 3] as n (n)}<option value={n}>{n}</option>{/each}</select
				></label
			>
			<label
				>{w('window')}<select
					value={p.seconds}
					onchange={(e) => saveConsole({ seconds: Number(e.currentTarget.value) as 30 | 60 | 180 })}
					>{#each [30, 60, 180] as n (n)}<option value={n}>{n} {w('seconds')}</option
						>{/each}</select
				></label
			>
		</div>
		<label
			>{w('cpuMode')}<select
				value={p.cpuMode}
				onchange={(e) => saveConsole({ cpuMode: e.currentTarget.value as 'cores' | 'total' })}
				><option value="cores">{w('cores')}</option><option value="total">{w('total')}</option
				></select
			></label
		>
		<p>{w('dragHint')}</p>
		<ol class="metric-order" data-metric-list>
			{#each p.order as metric, index (metric)}
				<li
					data-console-metric={metric}
					draggable="true"
					ondragstart={(e) => {
						dragged = metric;
						e.dataTransfer?.setData('text/plain', metric);
						if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
					}}
					ondragend={() => (dragged = null)}
					ondragover={(e) => {
						if (dragged) e.preventDefault();
					}}
					ondrop={(e) => {
						e.preventDefault();
						if (dragged) dropMetric(dragged, metric);
						dragged = null;
					}}
					class:dragging={dragged === metric}
				>
					<button
						type="button"
						class="drag-handle"
						aria-label={`${w('dragHint')} · ${w(metric)}`}
						use:reorderHandle={metric}><GripVertical size={16} aria-hidden="true" /></button
					><strong>{w(metric)}</strong>
					<label class="toggle"
						><input
							type="checkbox"
							checked={p.visible[metric]}
							onchange={(e) =>
								saveConsole({ visible: { ...p.visible, [metric]: e.currentTarget.checked } })}
						/>{w('show')}</label
					>
					<input
						type="color"
						value={p.colors[metric]}
						aria-label={`${w(metric)} · ${w('colors')}`}
						oninput={(e) =>
							saveConsole({ colors: { ...p.colors, [metric]: e.currentTarget.value } })}
					/>
					<button
						type="button"
						class="vr-control"
						aria-label={`${w('up')} · ${w(metric)}`}
						disabled={index === 0}
						onclick={() => moveMetric(metric, -1)}><ArrowUp size={14} /></button
					>
					<button
						type="button"
						class="vr-control"
						aria-label={`${w('down')} · ${w(metric)}`}
						disabled={index === 2}
						onclick={() => moveMetric(metric, 1)}><ArrowDown size={14} /></button
					>
				</li>{/each}
		</ol>
	</section>
	<section>
		<h3>{w('log')}</h3>
		<p>{w('logHint')}</p>
		<label
			>{w('fontSize')}<select
				value={p.fontSize}
				onchange={(e) => saveConsole({ fontSize: Number(e.currentTarget.value) })}
				>{#each [10, 11, 12, 13, 14, 15, 16, 18, 20] as size (size)}<option value={size}
						>{size} px</option
					>{/each}</select
			></label
		>
		<label class="toggle"
			><input
				type="checkbox"
				checked={p.wrap}
				onchange={(e) => saveConsole({ wrap: e.currentTarget.checked })}
			/>{w('wrap')}</label
		>
		<label class="toggle"
			><input
				type="checkbox"
				checked={p.follow}
				onchange={(e) => saveConsole({ follow: e.currentTarget.checked })}
			/>{w('follow')}</label
		>
		<div class="level-colors">
			{#each levels as level (level)}<label
					><input
						type="color"
						value={p.colors[level]}
						oninput={(e) =>
							saveConsole({ colors: { ...p.colors, [level]: e.currentTarget.value } })}
					/>{w(level)}</label
				>{/each}
		</div>
		<div class="preview" aria-label={w('preview')} style:font-size={`${p.fontSize}px`}>
			{#each levels as level (level)}<div style:color={p.colors[level]}>
					[{level.toUpperCase()}] {w(level)}
				</div>{/each}
		</div>
	</section>
	<button
		type="button"
		class="vr-control"
		onclick={() => saveConsole(structuredClone(DEFAULT_CONSOLE))}
		><RotateCcw size={15} />{w('reset')}</button
	>
	{#if consoleStorage.error}<p role="alert">{w('saveError')}</p>{/if}
</div>

<style>
	.drag-handle {
		touch-action: none;
		cursor: grab;
		background: transparent;
		border: 0;
		color: var(--text-secondary);
		padding: 4px;
	}
	li:global([data-drop-target]) {
		outline: 2px solid var(--accent);
	}
	.console-settings {
		display: grid;
		gap: 28px;
		min-width: 0;
	}
	section {
		display: grid;
		gap: 14px;
	}
	h3 {
		margin: 0;
		font-size: 1rem;
	}
	p {
		color: var(--text-secondary);
		font-size: 13px;
		line-height: 1.6;
		margin: 0;
	}
	label {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		font-size: 13px;
		min-width: 0;
	}
	select {
		max-width: 65%;
		background: var(--bg-input);
		border: 1px solid var(--border);
		color: var(--text-primary);
		padding: 8px;
		border-radius: var(--border-radius-sm);
	}
	.fields {
		display: flex;
		gap: 24px;
		flex-wrap: wrap;
	}
	.fields label {
		flex: 1;
	}
	.toggle {
		justify-content: start;
	}
	input[type='checkbox'] {
		accent-color: var(--accent);
		width: 16px;
		height: 16px;
	}
	input[type='color'] {
		width: 32px;
		min-width: 32px;
		height: 30px;
		padding: 2px;
		background: var(--bg-input);
		border: 1px solid var(--border);
		cursor: pointer;
	}
	.metric-order {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 8px;
	}
	li {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px;
		border: 1px solid var(--border);
		background: var(--surface-card);
		border-radius: var(--border-radius-sm);
	}
	li strong {
		flex: 1;
		font-size: 13px;
	}
	li .vr-control {
		padding: 5px;
		min-height: 30px;
	}
	.dragging {
		opacity: 0.5;
		border-color: var(--accent);
	}
	.level-colors {
		display: flex;
		gap: 16px;
		flex-wrap: wrap;
	}
	.preview {
		padding: 16px;
		background: #101216;
		border: 1px solid var(--border);
		font-family: monospace;
		line-height: 1.8;
		border-radius: var(--border-radius-sm);
	}
	@media (max-width: 650px) {
		li {
			gap: 5px;
			padding: 8px;
		}
		select {
			max-width: 60%;
		}
	}
</style>
