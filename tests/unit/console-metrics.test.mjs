import { test } from 'node:test';
import assert from 'node:assert/strict';
import { byteSize, metricSnapshot } from '../../src/lib/console/metrics.ts';
import { parseConsole, sampleStats, appendSample } from '../../src/lib/console/model.ts';
const snapshot = {
	process: {
		cpu_core_percent: 350,
		cpu_percent: 43.75,
		logical_cores: 8,
		memory_mb: 2304,
		disk_bytes_delta: 8192,
		sample_interval_ms: 2000,
		sampled_at_ms: 2000,
		ready: true,
		process_count: 3,
		scope: 'process_tree'
	},
	running: true,
	sampled_at_ms: 2000,
	interval_ms: 2000,
	memory_total_mb: 16384,
	network: {
		scope: 'machine_interfaces',
		provider: 'test',
		received_bytes_per_sec: 256,
		transmitted_bytes_per_sec: 64,
		total_received_bytes: 65536,
		total_transmitted_bytes: 8192,
		interval_ms: 2000,
		total_period: 'interface_uptime'
	},
	storage: {
		scope: 'instance_volume',
		provider: 'test',
		used_bytes: 2 ** 30,
		total_bytes: 8 * 2 ** 30
	},
	io: {
		scope: 'machine_disks',
		provider: 'test',
		read_bytes_per_sec: 2048,
		write_bytes_per_sec: 1024,
		total_read_bytes: null,
		total_write_bytes: null,
		total_period: 'unavailable'
	},
	gpus: [
		{
			name: 'Synthetic GPU',
			scope: 'machine_adapter',
			provider: 'test',
			utilization_percent: 20,
			vram_used_bytes: 2 ** 30,
			vram_total_bytes: 8 * 2 ** 30
		}
	]
};
test('console separates process tree CPU/RSS, filesystem occupancy, machine traffic and physical disk I/O', () => {
	const cpu = metricSnapshot(snapshot, 'cpu', 'cores');
	assert.equal(cpu.value, 350);
	assert.equal(cpu.maximum, 800);
	assert.equal(cpu.percent, 43.75);
	assert.equal(cpu.scope, 'process_tree');
	assert.equal(metricSnapshot(snapshot, 'cpu', 'total').value, 43.75);
	assert.equal(metricSnapshot(snapshot, 'storage', 'cores').value, 2 ** 30);
	assert.equal(metricSnapshot(snapshot, 'network', 'cores').value, 320);
	assert.equal(metricSnapshot(snapshot, 'network', 'cores').scope, 'machine_interfaces');
	const io = metricSnapshot(snapshot, 'disk', 'cores');
	assert.equal(io.value, 3072);
	assert.equal(io.scope, 'machine_disks');
	assert.equal(io.totalIncoming, null, 'provider without totals must not invent zero');
	assert.equal(metricSnapshot(snapshot, 'ram', 'cores').maximum, 16384);
	assert.equal(metricSnapshot(snapshot, 'gpu', 'cores').scope, 'machine_adapter');
});
test('stopping clears process values but retains the explicitly measured computer snapshot', () => {
	const stopped = { ...snapshot, running: false, process: null };
	assert.equal(sampleStats(stopped).cpu, null);
	assert.equal(sampleStats(stopped).ram, null);
	assert.equal(sampleStats(stopped).network, 320);
	assert.equal(sampleStats(stopped).storage, 2 ** 30);
	assert.equal(metricSnapshot(stopped, 'cpu', 'cores').value, null);
});
test('missing, zero and fractional rates remain distinct and use binary units', () => {
	assert.equal(byteSize(null, true), '—');
	assert.equal(byteSize(0, true), '0 B/s');
	assert.equal(byteSize(0.25, true), '0.25 B/s');
	assert.equal(byteSize(1024, true), '1.00 KiB/s');
	assert.equal(byteSize(2 ** 30), '1.00 GiB');
	assert.equal(byteSize(Infinity), '—');
	const missing = {
		...snapshot,
		network: { ...snapshot.network, received_bytes_per_sec: null },
		gpus: []
	};
	assert.equal(sampleStats(missing).network, null);
	assert.equal(metricSnapshot(missing, 'gpu', 'cores').value, null);
	const zero = {
		...snapshot,
		network: { ...snapshot.network, received_bytes_per_sec: 0, transmitted_bytes_per_sec: 0 }
	};
	assert.equal(sampleStats(zero).network, 0);
});
test('legacy three-metric preferences retain explicit choices while migrating six independent metrics', () => {
	const preferences = parseConsole({
		version: 1,
		order: ['disk', 'ram', 'cpu'],
		visible: { cpu: false, ram: true, disk: false },
		colors: { disk: '#123456' },
		seconds: 180,
		cpuMode: 'total',
		metricStyles: { ram: 'gauge' }
	});
	assert.equal(preferences.version, 2);
	assert.deepEqual(preferences.order, ['disk', 'ram', 'cpu', 'storage', 'network', 'gpu']);
	assert.equal(preferences.visible.cpu, false);
	assert.equal(preferences.visible.disk, false);
	assert.equal(preferences.colors.disk, '#123456');
	assert.equal(preferences.metricStyles.ram, 'gauge');
	assert.equal(preferences.cpuMode, 'total');
	for (const key of ['storage', 'network', 'gpu'])
		assert.equal(preferences.metricSeconds[key], 180);
	assert.deepEqual(parseConsole(JSON.parse(JSON.stringify(preferences))), preferences);
});
test('new metric history remains bounded and missing samples make real gaps', () => {
	let history = [];
	for (let i = 0; i < 1000; i++)
		history = appendSample(history, sampleStats({ ...snapshot, sampled_at_ms: i * 2000 }));
	assert.ok(history.length <= 92);
	assert.equal(history.at(-1).network, 320);
	assert.equal(history.at(-1).disk, 3072 / 1048576);
});
