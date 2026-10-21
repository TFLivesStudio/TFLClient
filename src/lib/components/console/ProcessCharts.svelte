<script lang="ts">
	import { reorderHandle } from '$lib/console/reorder';
	import { onMount } from 'svelte';
	import { Cpu, MemoryStick, HardDrive, ArrowUp, ArrowDown, GripVertical } from 'lucide-svelte';
	import { w } from '$lib/i18n/workbench';
	import {
		appendSample,
		sampleStats,
		chartPath,
		type Sample,
		type ProcessStats,
		type Metric
	} from '$lib/console/model';
	import {
		consolePrefs as p,
		watchConsole,
		moveMetric,
		dropMetric
	} from '$lib/console/preferences.svelte';
	let {
		instanceName,
		readStats,
		running = true
	}: {
		instanceName: string;
		readStats: () => Promise<ProcessStats | null>;
		running?: boolean;
	} = $props();
	let history = $state<Sample[]>([]),
		current = $state<ProcessStats | null>(null);
	let dragged = $state<Metric | null>(null);
	const icons = { cpu: Cpu, ram: MemoryStick, disk: HardDrive };
	let visible = $state(true);
	const activeMetrics = $derived(p.order.filter((m) => p.visible[m]));
	const end = $derived(history.at(-1)?.time ?? Date.now());
	onMount(() => {
		const update = () => (visible = !document.hidden);
		update();
		document.addEventListener('visibilitychange', update);
		const clean = watchConsole();
		return () => {
			clean();
			document.removeEventListener('visibilitychange', update);
		};
	});
	$effect(() => {
		const name = instanceName;
		history = [];
		current = null;
		// Explicit identity reset; changing visibility/running must retain history.
		void name;
	});
	$effect(() => {
		if (!running || !visible || !activeMetrics.length) return;
		const read = readStats;
		void instanceName;
		let disposed = false,
			timer: ReturnType<typeof setTimeout> | undefined;
		const poll = async () => {
			let result: ProcessStats | null = null;
			try {
				result = await read();
			} catch {
				/* a gap, never fake a zero */
			}
			if (disposed) return;
			current = result;
			history = appendSample(history, sampleStats(result));
			timer = setTimeout(poll, 1000);
		};
		void poll();
		return () => {
			disposed = true;
			clearTimeout(timer);
		};
	});
	function values(metric: Metric) {
		return history
			.filter((s) => s.time >= end - p.seconds * 1000)
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
					if (dragged) dropMetric(dragged, metric);
					dragged = null;
				}}
			>
				<header>
					<span><Icon size={14} />{w(metric)}</span><strong>{display(metric)}</strong>
				</header>
				<div class="plot">
					<svg
						viewBox="0 0 400 100"
						preserveAspectRatio="none"
						role="img"
						aria-label={`${w(metric)} · ${p.seconds} ${w('seconds')} · ${w('scale')} 0–${max}`}
					>
						<path d="M0,10H400 M0,50H400 M0,90H400" class="grid" />
						<path d={chartPath(history, metric, p.cpuMode, end, p.seconds, max)} class="trace" />
					</svg><small class="scale"
						>{max}{metric === 'cpu' ? '%' : metric === 'ram' ? ' MB' : ' MB/s'}</small
					>
					{#if !values(metric).length}<span class="missing"
							>{current && !current.ready ? w('waiting') : w('unavailable')}</span
						>{/if}
				</div>
				<footer>
					<small>−{p.seconds}s → 0s</small><span class="moves">
						<button
							type="button"
							class="drag-handle"
							aria-label={`${w('dragHint')} · ${w(metric)}`}
							use:reorderHandle={metric}><GripVertical size={12} /></button
						>
						<button
							type="button"
							aria-label={`${w('up')} · ${w(metric)}`}
							disabled={p.order.indexOf(metric) === 0}
							onclick={() => moveMetric(metric, -1)}><ArrowUp size={12} /></button
						><button
							type="button"
							aria-label={`${w('down')} · ${w(metric)}`}
							disabled={p.order.indexOf(metric) === 2}
							onclick={() => moveMetric(metric, 1)}><ArrowDown size={12} /></button
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
</style>
