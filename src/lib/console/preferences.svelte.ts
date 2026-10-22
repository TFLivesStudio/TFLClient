import { persistAuthority } from '$lib/integrations/preferences';
import {
	CONSOLE_KEY,
	DEFAULT_CONSOLE,
	parseConsole,
	type ConsolePreferences,
	type Metric
} from './model';
export const consolePrefs = $state<ConsolePreferences>(structuredClone(DEFAULT_CONSOLE));
export const consoleStorage = $state({ error: false });
export function restoreConsole() {
	try {
		Object.assign(
			consolePrefs,
			parseConsole(JSON.parse(localStorage.getItem(CONSOLE_KEY) ?? '{}'))
		);
	} catch {
		Object.assign(consolePrefs, structuredClone(DEFAULT_CONSOLE));
	}
}
export function watchConsole() {
	restoreConsole();
	const handler = (e: StorageEvent) => {
		if (e.key === CONSOLE_KEY || e.key === null) restoreConsole();
	};
	window.addEventListener('storage', handler);
	return () => window.removeEventListener('storage', handler);
}
export function saveConsole(patch: Partial<ConsolePreferences>) {
	if (patch.chartStyle && !patch.metricStyles)
		patch.metricStyles = { cpu: patch.chartStyle, ram: patch.chartStyle, disk: patch.chartStyle };
	if (patch.seconds && !patch.metricSeconds)
		patch.metricSeconds = { cpu: patch.seconds, ram: patch.seconds, disk: patch.seconds };
	Object.assign(consolePrefs, parseConsole({ ...$state.snapshot(consolePrefs), ...patch }));
	try {
		localStorage.setItem(CONSOLE_KEY, JSON.stringify($state.snapshot(consolePrefs)));
		persistAuthority('console', $state.snapshot(consolePrefs));
		consoleStorage.error = false;
	} catch {
		consoleStorage.error = true;
	}
}
export function moveMetric(metric: Metric, offset: number) {
	const order = [...consolePrefs.order];
	const from = order.indexOf(metric),
		to = Math.max(0, Math.min(2, from + offset));
	order.splice(from, 1);
	order.splice(to, 0, metric);
	saveConsole({ order });
}
export function dropMetric(metric: Metric, target: Metric) {
	moveMetric(metric, consolePrefs.order.indexOf(target) - consolePrefs.order.indexOf(metric));
}
