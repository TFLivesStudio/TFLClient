import type { Sample, ConsoleMetrics } from './model';
export const DEMO_END = 180000;
export const DEMO_HISTORY: Sample[] = Array.from({ length: 181 }, (_, i) => ({
	time: i * 1000,
	cpu: 180 + Math.sin(i / 8) * 125 + Math.cos(i / 3) * 35,
	total: (180 + Math.sin(i / 8) * 125 + Math.cos(i / 3) * 35) / 8,
	ram: 2100 + Math.sin(i / 16) * 140 + i * 2,
	disk: Math.max(0, Math.sin(i / 11) * 4),
	storage: 128 * 1073741824,
	network: Math.max(0, Math.sin(i / 7) * 65000),
	gpu: 35 + Math.sin(i / 13) * 15
}));
export const DEMO_CURRENT: ConsoleMetrics = {
	process: {
		cpu_percent: 37.5,
		cpu_core_percent: 300,
		logical_cores: 8,
		memory_mb: 2560,
		disk_bytes_delta: 1048576,
		sampled_at_ms: DEMO_END,
		sample_interval_ms: 2000,
		ready: true,
		process_count: 2,
		scope: 'process_tree'
	},
	running: true,
	sampled_at_ms: DEMO_END,
	interval_ms: 2000,
	memory_total_mb: 16384,
	network: {
		scope: 'machine_interfaces',
		provider: 'Demonstration data',
		received_bytes_per_sec: 24000,
		transmitted_bytes_per_sec: 6000,
		total_received_bytes: 4800000,
		total_transmitted_bytes: 1200000,
		interval_ms: 2000,
		total_period: 'interface_uptime'
	},
	storage: {
		scope: 'instance_volume',
		provider: 'Demonstration data',
		used_bytes: 128 * 1073741824,
		total_bytes: 512 * 1073741824
	},
	io: {
		scope: 'process_tree',
		provider: 'Demonstration data',
		read_bytes_per_sec: 524288,
		write_bytes_per_sec: 131072,
		total_read_bytes: 10485760,
		total_write_bytes: 2621440,
		total_period: 'current_process_tree_lifetime'
	},
	gpus: [
		{
			name: 'Example GPU',
			scope: 'machine_adapter',
			provider: 'Demonstration data',
			utilization_percent: 40,
			vram_used_bytes: 1073741824,
			vram_total_bytes: 8589934592
		}
	]
};
export const DEMO_LINES = [
	{ severity: 'info', line: '[12:04:01 INFO] Starting Minecraft…' },
	{ severity: 'debug', line: '[12:04:02 DEBUG] Runtime ready' },
	{ severity: 'warn', line: '[12:04:03 WARN] Resource pack uses an older format' },
	{ severity: 'error', line: '[12:04:04 ERROR] Example connection failure' },
	{ severity: 'info', line: '[12:04:05 INFO] World loaded' }
] as const;
