<script lang="ts">
	import type { Snippet } from 'svelte';
	let { dock, charts, log }: { dock: 'top' | 'bottom' | 'side'; charts: Snippet; log: Snippet } =
		$props();
</script>

<div class="console-layout" class:side={dock === 'side'} class:bottom={dock === 'bottom'}>
	<div class="charts-slot">{@render charts()}</div>
	<div class="log-slot">{@render log()}</div>
</div>

<style>
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
	@media (max-width: 550px) {
		.side .charts-slot {
			width: 40%;
		}
	}
</style>
