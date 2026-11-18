<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { ExperienceMode } from '$lib/state/experience.svelte';
	import MetricCards from './MetricCards.svelte';
	import {
		appendSample,
		sampleStats,
		monitorOf,
		type Sample,
		type ConsoleReading
	} from '$lib/console/model';
	import { tr } from '$lib/i18n/experience';
	import {
		consolePrefs as p,
		watchConsole,
		moveMetric,
		dropMetric
	} from '$lib/console/preferences.svelte';
	let {
		instanceName,
		readStats,
		mode,
		onMode,
		running = true
	}: {
		instanceName: string;
		readStats: () => Promise<ConsoleReading | null>;
		running?: boolean;
		mode?: ExperienceMode;
		onMode?: (mode: ExperienceMode) => void;
	} = $props();
	let history = $state<Sample[]>([]),
		current = $state<ConsoleReading | null>(null);
	let visible = $state(true),
		requestState = $state<'loading' | 'ready' | 'error'>('loading'),
		refreshNonce = $state(0);
	const activeMetrics = $derived(p.order.filter((metric) => p.visible[metric]));
	onMount(() => {
		const update = () => (visible = !document.hidden);
		update();
		document.addEventListener('visibilitychange', update);
		const clean = watchConsole();
		let disposed = false,
			stopSettings: UnlistenFn | undefined;
		void listen('settings-changed', () => {
			refreshNonce++;
		})
			.then((stop) => {
				if (disposed) stop();
				else stopSettings = stop;
			})
			.catch(() => {});
		return () => {
			disposed = true;
			stopSettings?.();
			clean();
			document.removeEventListener('visibilitychange', update);
		};
	});
	$effect(() => {
		void instanceName;
		history = [];
		current = null;
	});
	$effect(() => {
		if (!visible) return;
		const live = running,
			read = readStats;
		const needMetrics = activeMetrics.length > 0;
		void instanceName;
		void refreshNonce;
		current = null;
		requestState = 'loading';
		let disposed = false,
			timer: ReturnType<typeof setTimeout> | undefined;
		const poll = async () => {
			let result: ConsoleReading | null = null,
				failed = false;
			try {
				result = await read();
			} catch {
				failed = true;
			}
			if (disposed) return;
			current = result;
			const observedMode = monitorOf(result)?.ui_mode;
			if (observedMode === 'simple' || observedMode === 'advanced') onMode?.(observedMode);
			requestState = failed ? 'error' : 'ready';
			history = appendSample(history, sampleStats(result));
			// One measured computer snapshot while stopped. No stale process activity.
			if (live && needMetrics) timer = setTimeout(poll, 2000);
		};
		void poll();
		return () => {
			disposed = true;
			clearTimeout(timer);
		};
	});
</script>

<div class="monitor-caption" role="status">
	<span
		>{requestState === 'loading'
			? tr('Consultando métricas…', 'Reading metrics…')
			: requestState === 'error'
				? tr('No se pudieron consultar las métricas.', 'Metrics could not be read.')
				: running
					? tr('Monitoreo · 2 s', 'Monitoring · 2 s')
					: tr(
							'Instancia detenida · instantánea del equipo',
							'Instance stopped · computer snapshot'
						)}</span
	>
	{#if requestState === 'error'}<button type="button" onclick={() => refreshNonce++}
			>{tr('Reintentar', 'Retry')}</button
		>{/if}
	{#if current}<time datetime={new Date(current.sampled_at_ms).toISOString()}
			>{new Date(current.sampled_at_ms).toLocaleTimeString()}</time
		>{/if}
</div>
<MetricCards {history} {current} {p} {running} {mode} onMove={moveMetric} onDrop={dropMetric} />

<style>
	.monitor-caption {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 6px 10px 0;
		color: var(--text-secondary);
		font-size: 10px;
	}
	.monitor-caption button {
		padding: 3px 6px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--surface-interactive);
		color: var(--text-primary);
		font-size: inherit;
	}
</style>
