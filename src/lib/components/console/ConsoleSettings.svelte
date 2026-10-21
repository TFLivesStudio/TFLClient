<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { ArrowUp, ArrowDown, Undo2, RotateCcw, Check } from 'lucide-svelte';
	import { w } from '$lib/i18n/workbench';
	import { tr } from '$lib/i18n/experience';
	import {
		DEFAULT_CONSOLE,
		parseConsole,
		type ConsolePreferences,
		type Metric,
		type Severity
	} from '$lib/console/model';
	import {
		consolePrefs as live,
		consoleStorage,
		saveConsole,
		watchConsole
	} from '$lib/console/preferences.svelte';
	import { DEMO_HISTORY, DEMO_LINES } from '$lib/console/demo';
	import MetricCards from './MetricCards.svelte';
	import ConsoleLayout from './ConsoleLayout.svelte';
	import LogBlock from './LogBlock.svelte';
	import SettingGroup from '$lib/components/settings/SettingGroup.svelte';
	let draft = $state<ConsolePreferences>(structuredClone(DEFAULT_CONSOLE)),
		selected = $state<Metric | 'log'>('cpu'),
		undo = $state<ConsolePreferences[]>([]),
		saved = $state(false);
	let hasDraft = $state(false);
	const levels: Severity[] = ['info', 'warn', 'error', 'debug'];
	const metrics: Metric[] = ['cpu', 'ram', 'disk'];
	onMount(() => {
		const stop = watchConsole();
		draft = structuredClone($state.snapshot(live));
		return stop;
	});
	$effect(() => {
		const current = JSON.stringify(live);
		if (!untrack(() => hasDraft)) draft = parseConsole(JSON.parse(current));
	});
	const dirty = $derived(JSON.stringify(draft) !== JSON.stringify(live));
	function patch(value: Partial<ConsolePreferences>) {
		if (value.chartStyle && !value.metricStyles)
			value.metricStyles = { cpu: value.chartStyle, ram: value.chartStyle, disk: value.chartStyle };
		if (value.seconds && !value.metricSeconds)
			value.metricSeconds = { cpu: value.seconds, ram: value.seconds, disk: value.seconds };
		undo = [...undo.slice(-19), structuredClone($state.snapshot(draft))];
		draft = parseConsole({ ...$state.snapshot(draft), ...value });
		hasDraft = true;
		saved = false;
	}
	function move(metric: Metric, delta: number) {
		const order = [...draft.order],
			from = order.indexOf(metric),
			to = Math.max(0, Math.min(2, from + delta));
		order.splice(from, 1);
		order.splice(to, 0, metric);
		patch({ order });
	}
	function drop(from: Metric, to: Metric) {
		move(from, draft.order.indexOf(to) - draft.order.indexOf(from));
	}
	function apply() {
		saveConsole($state.snapshot(draft));
		hasDraft = false;
		saved = !consoleStorage.error;
	}
	function cancel() {
		draft = structuredClone($state.snapshot(live));
		undo = [];
		hasDraft = false;
		saved = false;
	}
	function undoLast() {
		const previous = undo.at(-1);
		if (previous) {
			draft = previous;
			undo = undo.slice(0, -1);
			hasDraft = true;
			saved = false;
		}
	}
	function preset(id: string) {
		if (id === 'classic') patch({ dock: 'top', visible: { cpu: false, ram: false, disk: false } });
		else if (id === 'compact')
			patch({
				dock: 'top',
				columns: 3,
				chartStyle: 'gauge',
				visible: { cpu: true, ram: true, disk: true },
				fontSize: 11
			});
		else
			patch({
				dock: id as 'top' | 'bottom' | 'side',
				columns: id === 'side' ? 1 : 2,
				visible: { ...draft.visible, cpu: true, ram: true }
			});
	}
</script>

<div class="console-designer">
	<SettingGroup
		id="console-layout"
		hint={tr(
			'Arrastra las asas de las tarjetas o usa sus flechas. Las posiciones respetan estos moldes. Los cambios se guardan al pulsar Aplicar.',
			'Drag card handles or use their arrows. Positions stay inside these layouts. Changes save when you press Apply.'
		)}
	>
		<div class="molds">
			{#each [['classic', tr('Clásica', 'Classic')], ['top', tr('Superior', 'Top')], ['bottom', tr('Inferior', 'Bottom')], ['side', tr('Lateral', 'Side')], ['compact', tr('Compacta', 'Compact')]] as [id, label] (id)}<button
					class="mold"
					type="button"
					onclick={() => preset(id)}
					><span class={`mold-art ${id}`}><i></i><i></i><b></b></span>{label}</button
				>{/each}
		</div>
		<div class="demo-label">
			<strong>{tr('VISTA PREVIA · DATOS DE DEMOSTRACIÓN', 'PREVIEW · DEMONSTRATION DATA')}</strong
			><span
				>{tr(
					'Sin consultar procesos ni iniciar Minecraft',
					'No process polling or Minecraft launch'
				)}</span
			>
		</div>
		<div class="designer-preview" style:--log-size={`${draft.fontSize}px`}>
			<ConsoleLayout dock={draft.dock}>
				{#snippet charts()}<MetricCards
						history={DEMO_HISTORY}
						p={draft}
						selected={selected === 'log' ? null : selected}
						onMove={move}
						onDrop={drop}
						onSelect={(m) => (selected = m)}
					/>{/snippet}
				{#snippet log()}<button
						type="button"
						class="log-inspect"
						class:selected={selected === 'log'}
						onclick={() => (selected = 'log')}
						>{tr('Seleccionar registro', 'Select log block')}</button
					><LogBlock
						p={draft}
						shown={[...DEMO_LINES]}
						total={DEMO_LINES.length}
						empty=""
					/>{/snippet}
			</ConsoleLayout>
		</div>
		<div class="actions">
			<button type="button" class="vr-control primary" disabled={!dirty} onclick={apply}
				><Check size={14} />{tr('Aplicar', 'Apply')}</button
			><button type="button" class="vr-control" disabled={!dirty} onclick={cancel}
				>{tr('Cancelar', 'Cancel')}</button
			><button type="button" class="vr-control" disabled={!undo.length} onclick={undoLast}
				><Undo2 size={14} />{tr('Deshacer', 'Undo')}</button
			><button
				type="button"
				class="vr-control"
				onclick={() => patch(structuredClone(DEFAULT_CONSOLE))}
				><RotateCcw size={14} />{tr('Restablecer', 'Reset')}</button
			>
		</div>
		{#if saved}<p role="status">
				{tr(
					'Consola guardada. Las ventanas abiertas se sincronizan automáticamente.',
					'Console saved. Open windows sync automatically.'
				)}
			</p>{/if}
	</SettingGroup>
	<SettingGroup
		id="console-metrics"
		hint={tr(
			'Selecciona una tarjeta para personalizarla. CPU por núcleo puede superar 100%; RAM muestra memoria residente, no solo el heap de Java.',
			'Select a card to customize it. Per-core CPU can exceed 100%; RAM shows resident memory, not only Java heap.'
		)}
	>
		<div class="metric-tabs">
			{#each metrics as metric (metric)}<button
					type="button"
					class="vr-control"
					aria-pressed={selected === metric}
					onclick={() => (selected = metric)}>{w(metric)}</button
				>{/each}
		</div>
		{#if selected !== 'log'}<div class="inspector">
				<h4>{w(selected)}</h4>
				<label
					>{w('chartStyle')}<select
						value={draft.metricStyles[selected]}
						onchange={(e) => {
							if (selected !== 'log')
								patch({
									metricStyles: {
										...draft.metricStyles,
										[selected]: e.currentTarget.value as 'lines' | 'gauge'
									}
								});
						}}
						><option value="lines">{w('styleLines')}</option><option value="gauge"
							>{w('styleGauge')}</option
						></select
					></label
				><label
					>{w('window')}<select
						value={draft.metricSeconds[selected]}
						onchange={(e) => {
							if (selected !== 'log')
								patch({
									metricSeconds: {
										...draft.metricSeconds,
										[selected]: Number(e.currentTarget.value) as 30 | 60 | 180
									}
								});
						}}
						>{#each [30, 60, 180] as n (n)}<option value={n}>{n} s</option>{/each}</select
					></label
				><label
					>{w('show')}<input
						type="checkbox"
						checked={draft.visible[selected]}
						onchange={(e) =>
							patch({ visible: { ...draft.visible, [selected]: e.currentTarget.checked } })}
					/></label
				><label
					>{w('colors')}<input
						type="color"
						value={draft.colors[selected]}
						oninput={(e) =>
							patch({ colors: { ...draft.colors, [selected]: e.currentTarget.value } })}
					/></label
				>
				<div class="actions">
					<button
						type="button"
						class="vr-control"
						aria-label={`${w('up')} · ${w(selected)}`}
						disabled={draft.order.indexOf(selected) === 0}
						onclick={() => {
							if (selected !== 'log') move(selected, -1);
						}}><ArrowUp size={14} />{w('up')}</button
					><button
						type="button"
						class="vr-control"
						aria-label={`${w('down')} · ${w(selected)}`}
						disabled={draft.order.indexOf(selected) === 2}
						onclick={() => {
							if (selected !== 'log') move(selected, 1);
						}}><ArrowDown size={14} />{w('down')}</button
					>
				</div>
			</div>{/if}
		<div class="fields">
			<label
				>{w('columns')}<select
					value={draft.columns}
					onchange={(e) => patch({ columns: Number(e.currentTarget.value) as 1 | 2 | 3 })}
					>{#each [1, 2, 3] as n (n)}<option value={n}>{n}</option>{/each}</select
				></label
			><label
				>{w('window')}<select
					value={draft.seconds}
					onchange={(e) => patch({ seconds: Number(e.currentTarget.value) as 30 | 60 | 180 })}
					>{#each [30, 60, 180] as n (n)}<option value={n}>{n} s</option>{/each}</select
				></label
			>
		</div>
		<label
			>{w('cpuMode')}<select
				value={draft.cpuMode}
				onchange={(e) => patch({ cpuMode: e.currentTarget.value as 'cores' | 'total' })}
				><option value="cores">{w('cores')}</option><option value="total">{w('total')}</option
				></select
			></label
		>
		<label
			>{w('chartStyle')}<select
				value={draft.chartStyle}
				onchange={(e) => patch({ chartStyle: e.currentTarget.value as 'lines' | 'gauge' })}
				><option value="lines">{w('styleLines')}</option><option value="gauge"
					>{w('styleGauge')}</option
				></select
			></label
		>
	</SettingGroup>
	<SettingGroup
		id="console-log"
		hint={tr(
			'Los niveles tienen texto y color para distinguir avisos incluso sin percibir el color.',
			'Levels use text and color to distinguish warnings without color perception.'
		)}
	>
		<div class="levels">
			{#each levels as level (level)}<label
					><span>{w(level)}</span><input
						type="color"
						value={draft.colors[level]}
						aria-label={`${w(level)} · ${w('colors')}`}
						oninput={(e) => patch({ colors: { ...draft.colors, [level]: e.currentTarget.value } })}
					/></label
				>{/each}
		</div>
		<label
			>{w('fontSize')}<input
				type="number"
				min="10"
				max="20"
				value={draft.fontSize}
				onchange={(e) => patch({ fontSize: Number(e.currentTarget.value) })}
			/></label
		><label
			>{w('wrap')}<input
				type="checkbox"
				checked={draft.wrap}
				onchange={(e) => patch({ wrap: e.currentTarget.checked })}
			/></label
		><label
			>{w('follow')}<input
				type="checkbox"
				checked={draft.follow}
				onchange={(e) => patch({ follow: e.currentTarget.checked })}
			/></label
		>
	</SettingGroup>
	{#if dirty}<div class="pending">
			<span>{tr('Cambios sin aplicar', 'Unapplied changes')}</span><button
				type="button"
				class="vr-control primary"
				onclick={apply}>{tr('Aplicar cambios', 'Apply changes')}</button
			>
		</div>{/if}
	{#if consoleStorage.error}<p class="error" role="alert">{w('saveError')}</p>{/if}
</div>

<style>
	.console-designer {
		display: grid;
		gap: 24px;
		min-width: 0;
	}
	.molds {
		display: grid;
		grid-template-columns: repeat(5, minmax(0, 1fr));
		gap: 8px;
	}
	.mold {
		border: 1px solid var(--border);
		background: var(--surface-card);
		color: var(--text-primary);
		border-radius: var(--border-radius-sm);
		padding: 8px;
		font-size: 11px;
		display: grid;
		gap: 7px;
		cursor: pointer;
	}
	.mold:hover {
		border-color: var(--accent);
	}
	.mold-art {
		height: 38px;
		background: var(--bg-main);
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 3px;
		padding: 4px;
		box-sizing: border-box;
	}
	.mold-art i {
		background: color-mix(in srgb, var(--accent) 35%, var(--surface-card));
		height: 10px;
	}
	.mold-art b {
		background: var(--surface-interactive);
		grid-column: 1/-1;
	}
	.classic i {
		display: none;
	}
	.classic b {
		grid-row: 1/3;
	}
	.bottom b {
		grid-row: 1;
	}
	.side {
		grid-template-columns: 1fr 2fr;
	}
	.side i {
		grid-column: 1;
	}
	.side b {
		grid-column: 2;
		grid-row: 1/3;
	}
	.compact {
		grid-template-columns: 1fr 1fr 1fr;
	}
	.compact b {
		grid-column: 1/-1;
	}
	.demo-label {
		display: flex;
		flex-wrap: wrap;
		gap: 5px 12px;
		justify-content: space-between;
		font-size: 9px;
		letter-spacing: 0.04em;
		color: var(--text-secondary);
		margin: 18px 0 8px;
	}
	.designer-preview {
		height: 340px;
		display: flex;
		overflow: hidden;
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		background: var(--bg-main);
	}
	.designer-preview :global(.process-charts) {
		max-height: 220px;
	}
	.log-inspect {
		border: 0;
		border-bottom: 1px solid var(--border);
		color: var(--text-secondary);
		background: var(--surface-card);
		font-size: 10px;
		padding: 6px;
		cursor: pointer;
	}
	.log-inspect.selected {
		color: var(--accent);
		box-shadow: inset 0 0 0 1px var(--accent);
	}
	.actions,
	.metric-tabs {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 12px;
	}
	.actions button {
		display: flex;
		align-items: center;
		gap: 5px;
	}
	.primary {
		background: var(--accent);
		color: var(--accent-text, #101010);
		border-color: var(--accent);
	}
	.inspector {
		margin-top: 14px;
		padding: 12px 14px;
		background: var(--surface-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
	}
	h4 {
		margin: 0 0 8px;
		font-size: 13px;
	}
	label {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		min-height: 40px;
		font-size: 12px;
	}
	select,
	input[type='number'] {
		padding: 8px;
		background: var(--bg-input);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		color: var(--text-primary);
		max-width: 60%;
	}
	input[type='number'] {
		width: 64px;
	}
	input[type='color'] {
		width: 36px;
		height: 28px;
		background: none;
		border: 1px solid var(--border);
		padding: 2px;
		border-radius: 4px;
	}
	input[type='checkbox'] {
		accent-color: var(--accent);
		width: 16px;
		height: 16px;
	}
	.fields,
	.levels {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}
	p {
		font-size: 12px;
		line-height: 1.6;
		color: var(--text-secondary);
	}
	.pending {
		position: sticky;
		bottom: 0;
		padding: 10px 12px;
		background: var(--surface-card);
		border: 1px solid var(--border);
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 12px;
		z-index: 2;
	}
	.error {
		color: var(--color-error);
	}
	@media (max-width: 900px) {
		.molds {
			grid-template-columns: repeat(3, minmax(0, 1fr));
		}
		.designer-preview {
			height: 320px;
		}
		.fields {
			grid-template-columns: 1fr;
		}
		.levels {
			gap: 0 12px;
		}
	}
	button:disabled {
		opacity: 0.4;
	}
</style>
