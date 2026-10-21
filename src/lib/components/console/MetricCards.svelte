<script lang="ts">
	import { reorderHandle } from '$lib/console/reorder';
	import {
		Cpu,
		MemoryStick,
		HardDrive,
		ArrowUp,
		ArrowDown,
		GripVertical,
		Settings2
	} from 'lucide-svelte';
	import GaugeRing from '$lib/components/ui/GaugeRing.svelte';
	import { w } from '$lib/i18n/workbench';
	import {
		chartPath,
		type Sample,
		type ProcessStats,
		type Metric,
		type ConsolePreferences
	} from '$lib/console/model';
	let {
		history,
		current = null,
		p,
		selected = null,
		onMove,
		onDrop,
		onSelect
	}: {
		history: Sample[];
		current?: ProcessStats | null;
		p: ConsolePreferences;
		selected?: Metric | null;
		onMove: (m: Metric, delta: number) => void;
		onDrop: (from: Metric, to: Metric) => void;
		onSelect?: (m: Metric) => void;
	} = $props();
	let dragged = $state<Metric | null>(null);
	const icons = { cpu: Cpu, ram: MemoryStick, disk: HardDrive };
	const activeMetrics = $derived(p.order.filter((m) => p.visible[m]));
	const end = $derived(history.at(-1)?.time ?? Date.now());
	function values(metric: Metric) {
		return history
			.filter((s) => s.time >= end - p.metricSeconds[metric] * 1000)
			.map((s) => (metric === 'cpu' ? (p.cpuMode === 'cores' ? s.cpu : s.total) : s[metric]))
			.filter((v): v is number => v !== null);
	}
	function ceiling(metric: Metric) {
		return metric === 'cpu'
			? Math.max(100, Math.ceil(Math.max(0, ...values(metric)) / 100) * 100)
			: metric === 'ram'
				? Math.max(1024, Math.ceil(Math.max(0, ...values(metric)) / 1024) * 1024)
				: Math.max(1, Math.ceil(Math.max(0, ...values(metric))));
	}
	function latest(metric: Metric) {
		const s = history.at(-1);
		return s ? (metric === 'cpu' ? (p.cpuMode === 'cores' ? s.cpu : s.total) : s[metric]) : null;
	}
	/** Porcentaje 0-100 para el anillo: valor actual contra el mismo techo de
	 * escala que usa la gráfica de líneas (CPU total = 100 % fijo). */
	function gaugePercent(metric: Metric, max: number) {
		const v = latest(metric);
		if (v === null) return 0;
		return (v / (metric === 'cpu' && p.cpuMode === 'total' ? 100 : max)) * 100;
	}
	function unit(metric: Metric) {
		return metric === 'cpu' ? '%' : metric === 'ram' ? ' MB' : ' MB/s';
	}
	function display(metric: Metric) {
		const s = history.at(-1);
		const v = s ? (metric === 'cpu' ? (p.cpuMode === 'cores' ? s.cpu : s.total) : s[metric]) : null;
		return v === null
			? '—'
			: `${v.toFixed(metric === 'ram' ? 0 : 1)} ${metric === 'cpu' ? '%' : metric === 'ram' ? 'MB' : 'MB/s'}`;
	}
</script>

{#if activeMetrics.length}
	<section
		data-metric-list
		class="process-charts"
		style:--chart-columns={Math.min(p.columns, activeMetrics.length)}
		aria-label={w('charts')}
	>
		{#each activeMetrics as metric (metric)}{@const Icon = icons[metric]}{@const max =
				ceiling(metric)}
			<article
				class="chart-card"
				class:selected={selected === metric}
				data-console-metric={metric}
				draggable="true"
				style:--metric-color={p.colors[metric]}
				ondragstart={(e) => {
					dragged = metric;
					e.dataTransfer?.setData('text/plain', metric);
				}}
				ondragend={() => (dragged = null)}
				ondragover={(e) => {
					if (dragged) e.preventDefault();
				}}
				ondrop={(e) => {
					e.preventDefault();
					if (dragged) onDrop(dragged, metric);
					dragged = null;
				}}
			>
				{#if p.metricStyles[metric] === 'gauge'}
					<div
						class="gauge-slot"
						style:--accent={p.colors[metric]}
						role="img"
						aria-label={`${w(metric)} · ${display(metric)} · ${w('scale')} 0–${max}${unit(metric)}`}
					>
						<GaugeRing
							value={gaugePercent(metric, max)}
							label={w(metric)}
							displayValue={display(metric)}
							icon={Icon}
						/>
						{#if !values(metric).length}<span class="missing-note"
								>{current && !current.ready ? w('waiting') : w('unavailable')}</span
							>{/if}
					</div>
				{:else}
					<header>
						<span><Icon size={14} />{w(metric)}</span><strong>{display(metric)}</strong>
					</header>
					<div class="plot">
						<svg
							viewBox="0 0 400 100"
							preserveAspectRatio="none"
							role="img"
							aria-label={`${w(metric)} · ${p.metricSeconds[metric]} ${w('seconds')} · ${w('scale')} 0–${max}`}
						>
							<path d="M0,10H400 M0,50H400 M0,90H400" class="grid" />
							<path
								d={chartPath(history, metric, p.cpuMode, end, p.metricSeconds[metric], max)}
								class="trace"
							/>
						</svg><small class="scale"
							>{max}{metric === 'cpu' ? '%' : metric === 'ram' ? ' MB' : ' MB/s'}</small
						>
						{#if !values(metric).length}<span class="missing"
								>{current && !current.ready ? w('waiting') : w('unavailable')}</span
							>{/if}
					</div>
				{/if}
				<footer>
					<small
						>{p.metricStyles[metric] === 'gauge'
							? `0–${max}${unit(metric)}`
							: `−${p.metricSeconds[metric]}s → 0s`}</small
					><span class="moves"
						>{#if onSelect}<button
								type="button"
								class="select-block"
								aria-label={`${w(metric)} · ${w('settings')}`}
								aria-pressed={selected === metric}
								onclick={() => onSelect?.(metric)}><Settings2 size={12} /></button
							>{/if}
						<button
							type="button"
							class="drag-handle"
							aria-label={`${w('dragHint')} · ${w(metric)}`}
							use:reorderHandle={{ metric, drop: onDrop }}><GripVertical size={12} /></button
						>
						<button
							type="button"
							aria-label={`${w('up')} · ${w(metric)}`}
							disabled={p.order.indexOf(metric) === 0}
							onclick={() => onMove(metric, -1)}><ArrowUp size={12} /></button
						><button
							type="button"
							aria-label={`${w('down')} · ${w(metric)}`}
							disabled={p.order.indexOf(metric) === 2}
							onclick={() => onMove(metric, 1)}><ArrowDown size={12} /></button
						></span
					>
				</footer>
				{#if metric === 'cpu'}<small class="unit"
						>{w(p.cpuMode === 'cores' ? 'cores' : 'total')}</small
					>{/if}
			</article>{/each}
	</section>
{/if}

<style>
	.drag-handle {
		touch-action: none;
		cursor: grab;
	}
	.chart-card:global([data-drop-target]) {
		outline: 2px solid var(--accent);
	}
	.process-charts {
		display: grid;
		grid-template-columns: repeat(var(--chart-columns), minmax(0, 1fr));
		gap: 8px;
		min-width: 0;
		flex-shrink: 0;
		padding: 8px;
		max-height: 42vh;
		overflow: auto;
	}
	.chart-card {
		min-width: 0;
		padding: 10px 12px 6px;
		border: 1px solid var(--border, #34363d);
		background: var(--surface-card, #17191f);
		color: var(--text-primary, #e4e6eb);
		border-radius: var(--border-radius-sm, 6px);
	}
	header,
	footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}
	header > span {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
	}
	strong {
		font-size: 13px;
		font-variant-numeric: tabular-nums;
	}
	header :global(svg) {
		color: var(--metric-color);
	}
	.plot {
		height: 74px;
		position: relative;
		margin-top: 6px;
	}
	.plot > svg {
		width: 100%;
		height: 100%;
		overflow: visible;
	}
	.grid {
		stroke: var(--border, #34363d);
		stroke-width: 0.5;
		fill: none;
		stroke-dasharray: 3 4;
	}
	.trace {
		stroke: var(--metric-color);
		stroke-width: 2;
		fill: none;
		vector-effect: non-scaling-stroke;
		stroke-linejoin: round;
	}
	.scale {
		position: absolute;
		right: 0;
		top: 0;
		font-size: 9px;
		opacity: 0.6;
	}
	.missing {
		position: absolute;
		inset: 0;
		display: grid;
		place-content: center;
		font-size: 11px;
		color: var(--text-secondary, #a1a1aa);
	}
	footer small,
	.unit {
		font-size: 10px;
		color: var(--text-secondary, #a1a1aa);
	}
	.moves {
		display: flex;
		gap: 2px;
	}
	button {
		border: 0;
		background: transparent;
		color: var(--text-secondary, #a1a1aa);
		padding: 3px;
		cursor: pointer;
	}
	button:disabled {
		opacity: 0.3;
		cursor: default;
	}
	.gauge-slot {
		margin-top: 6px;
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 4px;
	}
	.missing-note {
		font-size: 11px;
		color: var(--text-secondary, #a1a1aa);
	}
	.unit {
		display: block;
		margin-top: 2px;
	}
	@media (max-width: 700px) {
		.process-charts {
			grid-template-columns: repeat(min(var(--chart-columns), 2), minmax(0, 1fr));
		}
	}
	@media (max-height: 650px) {
		.plot {
			height: 48px;
		}
		.process-charts {
			max-height: 36vh;
		}
	}
	.chart-card.selected {
		outline: 2px solid var(--accent);
		outline-offset: -2px;
	}
	.process-charts {
		container-type: inline-size;
	}
	@container (max-width:420px) {
		.chart-card header {
			flex-wrap: wrap;
			gap: 4px;
		}
	}
</style>
