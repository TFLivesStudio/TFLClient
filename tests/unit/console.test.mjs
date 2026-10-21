import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	parseConsole,
	DEFAULT_CONSOLE,
	severity,
	sampleStats,
	appendSample,
	chartPath
} from '../../src/lib/console/model.ts';
test('console preferences survive round trip; bad values cannot break layout or inject CSS', () => {
	const p = parseConsole({
		order: ['ram', 'ram', 'bad', 'cpu'],
		visible: { cpu: false },
		colors: { warn: 'url(x)', cpu: '#112233' },
		columns: 99,
		seconds: 999,
		fontSize: 500,
		dock: 'absolute'
	});
	assert.deepEqual(p.order, ['ram', 'cpu', 'disk']);
	assert.equal(p.visible.cpu, false);
	assert.equal(p.colors.cpu, '#112233');
	assert.equal(p.colors.warn, DEFAULT_CONSOLE.colors.warn);
	assert.equal(p.columns, 2);
	assert.equal(p.seconds, 60);
	assert.equal(p.fontSize, 20);
	assert.equal(p.dock, 'top');
	assert.deepEqual(parseConsole(JSON.parse(JSON.stringify(p))), p);
	assert.deepEqual(parseConsole({ version: 99 }), DEFAULT_CONSOLE);
});
test('all supported arrangements persist without dropping metric order', () => {
	for (const dock of ['top', 'bottom', 'side'])
		for (const columns of [1, 2, 3]) {
			const p = parseConsole({
				dock,
				columns,
				order: ['disk', 'cpu', 'ram'],
				seconds: 180,
				cpuMode: 'total',
				follow: false,
				wrap: false
			});
			assert.equal(p.dock, dock);
			assert.equal(p.columns, columns);
			assert.deepEqual(p.order, ['disk', 'cpu', 'ram']);
			assert.equal(p.follow, false);
			assert.equal(p.wrap, false);
		}
});
test('Java severity takes priority over stderr and retains info/debug', () => {
	for (const line of ['[12:23:00] [Server thread/WARN]: overloaded', '[WARNING] slow', 'WARN: hi'])
		assert.equal(severity(line, 'stderr'), 'warn');
	assert.equal(severity('[main/INFO]: 0 errors', 'stderr'), 'info');
	assert.equal(severity('[ERROR] broken', 'stdout'), 'error');
	assert.equal(severity('[FATAL] broken', 'stdout'), 'error');
	assert.equal(severity('[TRACE] path', 'stdout'), 'debug');
	assert.equal(severity('at java.lang.Thread.run', 'stderr'), 'error');
	assert.equal(severity('Loaded with 0 errors', 'stdout'), 'info');
});
const stats = {
	cpu_percent: 62.5,
	cpu_core_percent: 250,
	logical_cores: 4,
	memory_mb: 3072,
	disk_bytes_delta: 2097152,
	sampled_at_ms: 10000,
	sample_interval_ms: 2000,
	ready: true
};
test('real CPU values above one core are preserved; disk uses actual elapsed time', () => {
	const s = sampleStats(stats);
	assert.equal(s.cpu, 250);
	assert.equal(s.total, 62.5);
	assert.equal(s.ram, 3072);
	assert.equal(s.disk, 1);
	assert.equal(sampleStats({ ...stats, ready: false }).cpu, null);
	assert.equal(sampleStats({ ...stats, ready: false }).ram, 3072);
	assert.equal(sampleStats(null).ram, null);
	assert.equal(sampleStats({ ...stats, cpu_core_percent: Infinity }).cpu, null);
});
test('history stays bounded, ignores duplicate/coalesced samples, and keeps missing data as gaps', () => {
	let h = [];
	for (let i = 0; i < 1000; i++) h = appendSample(h, { ...sampleStats(stats), time: i * 1000 });
	assert.ok(h.length <= 182);
	assert.equal(h.at(-1).time, 999000);
	assert.equal(appendSample(h, h.at(-1)), h);
	const samples = [
		{ ...sampleStats(stats), time: 1000 },
		{ ...sampleStats(stats), time: 2000 },
		sampleStats(null, 3000),
		{ ...sampleStats(stats), time: 4000 },
		{ ...sampleStats(stats), time: 9000 }
	];
	const path = chartPath(samples, 'cpu', 'cores', 10000, 30, 300);
	assert.equal((path.match(/M/g) || []).length, 3);
	assert.equal((path.match(/L/g) || []).length, 1);
	assert.ok(!/NaN|Infinity/.test(path));
	assert.notEqual(path, chartPath(samples, 'cpu', 'total', 10000, 30, 100));
});
test('chart style defaults to lines, accepts gauge and rejects anything else', () => {
	assert.equal(DEFAULT_CONSOLE.chartStyle, 'lines');
	assert.equal(parseConsole({}).chartStyle, 'lines');
	assert.equal(parseConsole({ chartStyle: 'gauge' }).chartStyle, 'gauge');
	assert.equal(parseConsole({ chartStyle: 'pie' }).chartStyle, 'lines');
	assert.equal(
		parseConsole(JSON.parse(JSON.stringify(parseConsole({ chartStyle: 'gauge' })))).chartStyle,
		'gauge'
	);
});
