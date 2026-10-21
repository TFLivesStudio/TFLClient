<script lang="ts">
	import { onMount } from 'svelte';
	import MetricCards from './MetricCards.svelte';

	import { appendSample, sampleStats, type Sample, type ProcessStats } from '$lib/console/model';
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

	let visible = $state(true);
	const activeMetrics = $derived(p.order.filter((m) => p.visible[m]));

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
</script>

<MetricCards {history} {current} {p} onMove={moveMetric} onDrop={dropMetric} />
