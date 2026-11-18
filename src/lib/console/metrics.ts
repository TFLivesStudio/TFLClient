import type { ConsoleReading, Metric } from './model';
export const validNumber = (value: unknown): number | null =>
	typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : null;
export function byteSize(value: number | null | undefined, rate = false): string {
	const n = validNumber(value);
	if (n === null) return '—';
	const units = rate ? ['B/s', 'KiB/s', 'MiB/s', 'GiB/s'] : ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
	const index = Math.max(
		0,
		Math.min(units.length - 1, n > 0 ? Math.floor(Math.log(n) / Math.log(1024)) : 0)
	);
	const scaled = n / 1024 ** index;
	return `${scaled.toFixed(index === 0 ? (rate && scaled > 0 && scaled < 10 ? 2 : 0) : scaled < 10 ? 2 : 1)} ${units[index]}`;
}
export function metricSnapshot(
	reading: ConsoleReading | null,
	metric: Metric,
	mode: 'cores' | 'total'
) {
	const monitor = reading && 'process' in reading ? reading : null;
	const process = reading && 'process' in reading ? reading.process : reading;
	const sum = (a: unknown, b: unknown) =>
		validNumber(a) !== null && validNumber(b) !== null ? Number(a) + Number(b) : null;
	let value: number | null = null,
		maximum: number | null = null;
	let scope = process?.scope ?? 'process',
		provider = process ? 'sysinfo / OS process counters' : '';
	let incoming: number | null = null,
		outgoing: number | null = null,
		totalIncoming: number | null = null,
		totalOutgoing: number | null = null;
	if (metric === 'cpu') {
		value = process?.ready
			? validNumber(mode === 'cores' ? process.cpu_core_percent : process.cpu_percent)
			: null;
		maximum = process ? (mode === 'cores' ? Math.max(1, process.logical_cores) * 100 : 100) : null;
	} else if (metric === 'ram') {
		value = process ? validNumber(process.memory_mb) : null;
		maximum = validNumber(monitor?.memory_total_mb);
	} else if (metric === 'storage') {
		value = validNumber(monitor?.storage?.used_bytes);
		maximum = validNumber(monitor?.storage?.total_bytes);
		scope = monitor?.storage?.scope ?? 'instance_volume';
		provider = monitor?.storage?.provider ?? '';
	} else if (metric === 'network') {
		incoming = validNumber(monitor?.network?.received_bytes_per_sec);
		outgoing = validNumber(monitor?.network?.transmitted_bytes_per_sec);
		value = sum(incoming, outgoing);
		totalIncoming = validNumber(monitor?.network?.total_received_bytes);
		totalOutgoing = validNumber(monitor?.network?.total_transmitted_bytes);
		scope = monitor?.network?.scope ?? 'machine_interfaces';
		provider = monitor?.network?.provider ?? '';
	} else if (metric === 'disk') {
		if (monitor) {
			incoming = validNumber(monitor.io?.read_bytes_per_sec);
			outgoing = validNumber(monitor.io?.write_bytes_per_sec);
			value = sum(incoming, outgoing);
			totalIncoming = validNumber(monitor.io?.total_read_bytes);
			totalOutgoing = validNumber(monitor.io?.total_write_bytes);
			scope = monitor.io?.scope ?? 'unavailable';
			provider = monitor.io?.provider ?? '';
		} else if (process?.ready && process.sample_interval_ms > 0) {
			value = validNumber((process.disk_bytes_delta * 1000) / process.sample_interval_ms);
		}
	} else {
		value = validNumber(monitor?.gpus?.[0]?.utilization_percent);
		maximum = 100;
		scope = monitor?.gpus?.[0]?.scope ?? 'machine_adapter';
		provider = monitor?.gpus?.[0]?.provider ?? '';
	}
	const percent =
		value !== null && maximum !== null && maximum > 0
			? Math.max(0, Math.min(100, (value / maximum) * 100))
			: null;
	return {
		value,
		maximum,
		percent,
		scope,
		provider,
		incoming,
		outgoing,
		totalIncoming,
		totalOutgoing
	};
}
