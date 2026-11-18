<script lang="ts">
	import {
		Cpu,
		MemoryStick,
		HardDrive,
		Network,
		Activity,
		Monitor,
		GripVertical,
		Settings2,
		ArrowUp,
		ArrowDown
	} from 'lucide-svelte';
	import { tr } from '$lib/i18n/experience';
	import { w } from '$lib/i18n/workbench';
	import { experience, type ExperienceMode } from '$lib/state/experience.svelte';
	import { reorderHandle } from '$lib/console/reorder';
	import {
		chartPath,
		monitorOf,
		processOf,
		type Sample,
		type ConsoleReading,
		type Metric,
		type ConsolePreferences
	} from '$lib/console/model';
	import { byteSize, metricSnapshot } from '$lib/console/metrics';
	import { metricLabel } from '$lib/console/labels';
	let {
		history,
		current = null,
		p,
		selected = null,
		onMove,
		onDrop,
		onSelect,
		mode,
		running = true
	}: {
		history: Sample[];
		current?: ConsoleReading | null;
		p: ConsolePreferences;
		selected?: Metric | null;
		onMove: (metric: Metric, delta: number) => void;
		onDrop: (from: Metric, to: Metric) => void;
		onSelect?: (metric: Metric) => void;
		running?: boolean;
		mode?: ExperienceMode;
	} = $props();
	let dragged = $state<Metric | null>(null);
	const advanced = $derived((mode ?? experience.mode) === 'advanced');
	const icons = {
		cpu: Cpu,
		ram: MemoryStick,
		storage: HardDrive,
		network: Network,
		disk: Activity,
		gpu: Monitor
	};
	const activeMetrics = $derived(p.order.filter((metric) => p.visible[metric]));
	const end = $derived(history.at(-1)?.time ?? Date.now());
	const monitor = $derived(monitorOf(current));
	const process = $derived(processOf(current));
	function name(metric: Metric) {
		return metricLabel(metric);
	}
	function scope(value: string) {
		return value === 'process_tree'
			? tr('Árbol de instancia', 'Instance process tree')
			: value === 'process'
				? tr('Proceso de instancia', 'Instance process')
				: value === 'machine_interfaces'
					? tr('Interfaces del equipo', 'Computer interfaces')
					: value === 'instance_volume'
						? tr('Volumen de instancia', 'Instance volume')
						: value === 'machine_disks'
							? tr('Discos del equipo', 'Computer disks')
							: value === 'machine_adapter'
								? tr('GPU del equipo', 'Computer GPU')
								: tr('No disponible', 'Unavailable');
	}
	function display(metric: Metric, value: number | null) {
		return value === null
			? '—'
			: metric === 'cpu' || metric === 'gpu'
				? `${value.toFixed(1)}%`
				: metric === 'ram'
					? byteSize(value * 1048576)
					: byteSize(value, metric === 'network' || metric === 'disk');
	}
	function maximum(metric: Metric) {
		const snapshot = metricSnapshot(current, metric, p.cpuMode);
		if (snapshot.maximum && snapshot.maximum > 0) return snapshot.maximum;
		const values = history
			.map((sample) =>
				metric === 'cpu' ? (p.cpuMode === 'cores' ? sample.cpu : sample.total) : sample[metric]
			)
			.filter((value): value is number => typeof value === 'number' && Number.isFinite(value));
		return Math.max(1, ...values) * 1.12;
	}
	function plot(metric: Metric) {
		const snapshot = metricSnapshot(current, metric, p.cpuMode);
		if (
			!running &&
			(metric === 'cpu' ||
				metric === 'ram' ||
				(metric === 'disk' && snapshot.scope === 'process_tree'))
		)
			return '';
		return chartPath(history, metric, p.cpuMode, end, p.metricSeconds[metric], maximum(metric));
	}
	function denominator(metric: Metric, maximum: number | null) {
		if (metric === 'cpu')
			return maximum === null
				? tr('Capacidad no disponible', 'Capacity unavailable')
				: `${tr('de', 'of')} ${maximum.toFixed(0)}% · ${p.cpuMode === 'cores' ? tr('suma por núcleo', 'sum across cores') : tr('normalizada al equipo', 'normalized to computer')}`;
		if (metric === 'ram')
			return maximum === null ? 'RSS' : `${tr('de', 'of')} ${byteSize(maximum * 1048576)} · RSS`;
		if (metric === 'storage')
			return maximum === null
				? tr('Capacidad no disponible', 'Capacity unavailable')
				: `${tr('de', 'of')} ${byteSize(maximum)}`;
		if (metric === 'gpu') {
			const gpu = monitor?.gpus?.[0];
			return gpu
				? `${gpu.name} · VRAM ${byteSize(gpu.vram_used_bytes)} / ${byteSize(gpu.vram_total_bytes)}`
				: tr('Sin proveedor compatible', 'No supported provider');
		}
		return tr('Entrada + salida por segundo', 'Input + output per second');
	}
</script>

{#if activeMetrics.length}
	<section
		data-metric-list
		class="process-charts"
		class:advanced
		style:--chart-columns={Math.min(p.columns, activeMetrics.length)}
		aria-label={w('charts')}
	>
		{#each activeMetrics as metric (metric)}
			{@const Icon = icons[metric]}
			{@const snapshot = metricSnapshot(current, metric, p.cpuMode)}
			<article
				class="chart-card"
				class:selected={selected === metric}
				class:measured={snapshot.value !== null}
				class:active={snapshot.value !== null && snapshot.value > 0}
				data-console-metric={metric}
				style:--metric-color={p.colors[metric]}
				draggable={advanced}
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
				<header>
					<span class="metric-label"
						><span class="metric-icon"><Icon size={14} /></span>{name(metric)}</span
					>
					<div
						class="metric-ring"
						role="img"
						aria-label={snapshot.percent === null
							? tr('Porcentaje no disponible', 'Percentage unavailable')
							: `${snapshot.percent.toFixed(0)}%`}
					>
						<svg viewBox="0 0 32 32" aria-hidden="true"
							><circle cx="16" cy="16" r="13" class="ring-track" /><circle
								cx="16"
								cy="16"
								r="13"
								class="ring-value"
								style:stroke-dasharray={`${snapshot.percent ?? 0} 100`}
								pathLength="100"
							/></svg
						><small>{snapshot.percent === null ? '—' : `${Math.round(snapshot.percent)}%`}</small>
					</div>
				</header>
				<strong class="metric-value">{display(metric, snapshot.value)}</strong>
				<p class="metric-denominator">{denominator(metric, snapshot.maximum)}</p>
				{#if metric === 'network' || metric === 'disk'}<p class="directions">
						↓ {byteSize(snapshot.incoming, true)} <span>↑ {byteSize(snapshot.outgoing, true)}</span>
					</p>{/if}
				<p class="metric-scope">
					{scope(snapshot.scope)}{#if snapshot.value === null}
						· {tr('No disponible', 'Unavailable')}{/if}
				</p>
				{#if advanced}<div class="metric-diagnostics">
						<span title={snapshot.provider}
							>{snapshot.provider || tr('Proveedor no disponible', 'Provider unavailable')}</span
						>
						{#if metric === 'network' || metric === 'disk'}<span
								>Σ ↓ {byteSize(snapshot.totalIncoming)} · ↑ {byteSize(snapshot.totalOutgoing)}</span
							><span
								>{metric === 'network'
									? tr('Totales desde inicio de interfaces', 'Totals since interface startup')
									: monitor?.io?.total_period === 'unavailable'
										? tr(
												'Totales no disponibles en este proveedor',
												'Totals unavailable from this provider'
											)
										: tr('Totales de procesos presentes', 'Totals of current processes')}</span
							>{/if}
						{#if metric === 'cpu' || metric === 'ram'}<span
								>{process?.process_count ?? '—'}
								{tr(
									'procesos · memoria residente, no heap',
									'processes · resident memory, not heap'
								)}</span
							>{/if}
						{#if metric === 'gpu' && monitor?.gpus?.length > 1}{#each monitor.gpus.slice(1) as gpu (gpu.name)}<span
									>{gpu.name}: {gpu.utilization_percent === null
										? '—'
										: `${gpu.utilization_percent}%`} · VRAM {byteSize(gpu.vram_used_bytes)}</span
								>{/each}{/if}
					</div>{/if}
				<div class="plot">
					<svg
						viewBox="0 0 400 100"
						preserveAspectRatio="none"
						role="img"
						aria-label={`${name(metric)} · ${p.metricSeconds[metric]} s`}
						><path d="M0,98H400" class="baseline" /><path d={plot(metric)} class="trace" /></svg
					>
				</div>
				{#if advanced}<footer>
						<small>−{p.metricSeconds[metric]}s → 0s</small><span class="moves">
							{#if onSelect}<button
									type="button"
									aria-label={`${name(metric)} · ${w('settings')}`}
									aria-pressed={selected === metric}
									onclick={() => onSelect?.(metric)}><Settings2 size={12} /></button
								>{/if}
							<button
								type="button"
								class="drag-handle"
								aria-label={`${w('dragHint')} · ${name(metric)}`}
								use:reorderHandle={{ metric, drop: onDrop }}><GripVertical size={12} /></button
							>
							<button
								type="button"
								aria-label={`${w('up')} · ${name(metric)}`}
								disabled={p.order.indexOf(metric) === 0}
								onclick={() => onMove(metric, -1)}><ArrowUp size={12} /></button
							>
							<button
								type="button"
								aria-label={`${w('down')} · ${name(metric)}`}
								disabled={p.order.indexOf(metric) === p.order.length - 1}
								onclick={() => onMove(metric, 1)}><ArrowDown size={12} /></button
							>
						</span>
					</footer>{/if}
			</article>
		{/each}
	</section>
{/if}

<style>
	.process-charts {
		display: grid;
		grid-template-columns: repeat(var(--chart-columns), minmax(0, 1fr));
		gap: 10px;
		padding: 10px;
		min-width: 0;
		max-height: 42vh;
		overflow: auto;
		flex-shrink: 0;
	}
	.process-charts:not(.advanced) {
		grid-template-columns: repeat(3, minmax(0, 1fr));
	}
	.chart-card {
		position: relative;
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 146px;
		padding: 12px 13px 0;
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		overflow: hidden;
		color: var(--text-primary);
		background: linear-gradient(
			145deg,
			color-mix(in srgb, var(--surface-card) 96%, transparent),
			var(--surface-card)
		);
	}
	.chart-card.selected,
	.chart-card:global([data-drop-target]) {
		outline: 2px solid var(--accent);
		outline-offset: -2px;
	}
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		min-height: 28px;
	}
	.metric-label {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 11px;
		font-weight: 700;
		min-width: 0;
	}
	.metric-icon {
		display: grid;
		place-items: center;
		width: 26px;
		height: 26px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		color: var(--text-secondary);
		background: var(--surface-interactive);
	}
	.metric-ring {
		position: relative;
		width: 30px;
		height: 30px;
		flex-shrink: 0;
	}
	.metric-ring svg {
		width: 100%;
		height: 100%;
		transform: rotate(-90deg);
		fill: none;
		stroke-width: 2.7;
	}
	.ring-track {
		stroke: var(--border);
	}
	.ring-value {
		stroke: var(--metric-color);
	}
	.metric-ring small {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		color: var(--text-secondary);
		font-size: 7px;
		font-weight: 700;
	}
	.metric-value {
		display: block;
		margin-top: 8px;
		font-size: clamp(17px, 1.6vw, 23px);
		line-height: 1.2;
		letter-spacing: -0.035em;
		font-variant-numeric: tabular-nums;
	}
	.active .metric-value {
		color: color-mix(in srgb, var(--metric-color) 72%, var(--text-primary));
	}
	.metric-denominator,
	.directions,
	.metric-scope {
		margin: 3px 0 0;
		font-size: 9px;
		line-height: 1.4;
		color: var(--text-secondary);
		overflow-wrap: anywhere;
	}
	.directions {
		font-variant-numeric: tabular-nums;
	}
	.directions span {
		margin-left: 8px;
	}
	.metric-scope {
		margin-top: 5px;
	}
	.metric-diagnostics {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin-top: 7px;
		font-size: 9px;
		line-height: 1.35;
		color: var(--text-secondary);
	}
	.metric-diagnostics span {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.plot {
		height: 22px;
		margin: auto -13px 0;
		padding-top: 7px;
	}
	.plot svg {
		width: 100%;
		height: 100%;
		overflow: visible;
	}
	.baseline {
		fill: none;
		stroke: var(--text-secondary);
		opacity: 0.4;
		stroke-width: 1;
		vector-effect: non-scaling-stroke;
	}
	.trace {
		fill: none;
		stroke: var(--metric-color);
		opacity: 0.9;
		stroke-width: 1.2;
		vector-effect: non-scaling-stroke;
		stroke-linejoin: round;
	}
	footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
		padding: 4px 0;
		color: var(--text-secondary);
	}
	footer small {
		font-size: 9px;
	}
	.moves {
		display: flex;
		gap: 1px;
	}
	.moves button {
		display: inline-grid;
		place-items: center;
		padding: 3px;
		background: transparent;
		border: 0;
		color: var(--text-secondary);
	}
	.moves button:disabled {
		opacity: 0.35;
	}
	.drag-handle {
		touch-action: none;
		cursor: grab;
	}
	.advanced .chart-card {
		min-height: 192px;
	}
	:global(html[data-visual='pixel']) .chart-card {
		box-shadow: 2px 2px 0 #0003;
		border-width: 2px;
	}
	@media (max-width: 700px) {
		.process-charts,
		.process-charts:not(.advanced) {
			grid-template-columns: repeat(min(var(--chart-columns), 2), minmax(0, 1fr));
		}
	}
	@media (max-height: 650px) {
		.chart-card {
			min-height: 138px;
		}
		.process-charts {
			max-height: 37vh;
		}
	}
</style>
