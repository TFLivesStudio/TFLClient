// Linux-only process measurements, separated from browser heap/GPU claims.
import { readFile, writeFile, rename, stat } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';
import path from 'node:path';
const root = process.env.TFL_DATA_DIR,
	host = Number(process.env.MCP_HOST_PID);
assert(root && host, 'Marked isolated native test profile and its host PID required');
const sdk = process.env.MCP_SDK_ROOT;
const { Client } = await import(`${sdk}/client/index.js`);
const { StdioClientTransport } = await import(`${sdk}/client/stdio.js`);
const tickRate = Number(execFileSync('getconf', ['CLK_TCK'], { encoding: 'utf8' }));
const delay = (ms) => new Promise((r) => setTimeout(r, ms));
async function state() {
	return JSON.parse(await readFile(path.join(root, 'status.json'), 'utf8'));
}
async function control(v) {
	await writeFile(path.join(root, 'control.tmp'), JSON.stringify(v));
	await rename(path.join(root, 'control.tmp'), path.join(root, 'control.json'));
	await delay(350);
}
async function sample(pid) {
	const status = await readFile(`/proc/${pid}/status`, 'utf8'),
		raw = await readFile(`/proc/${pid}/stat`, 'utf8');
	const parts = raw.slice(raw.lastIndexOf(')') + 2).split(' ');
	return {
		rss_kib: Number(status.match(/^VmRSS:\s+(\d+)/m)[1]),
		ticks: Number(parts[11]) + Number(parts[12]),
		at: performance.now()
	};
}
async function idle(pid) {
	const a = await sample(pid);
	await delay(1500);
	const b = await sample(pid);
	return {
		rss_kib: b.rss_kib,
		cpu_one_core_percent: (100 * ((b.ticks - a.ticks) / tickRate)) / ((b.at - a.at) / 1000),
		interval_ms: b.at - a.at
	};
}
await control({ op: 'enabled', enabled: false });
assert(!(await state()).enabled);
const disabled = await idle(host);
await control({ op: 'enabled', enabled: true });
assert((await state()).enabled);
const enabled = await idle(host);
const binary = process.env.MCP_BINARY;
const transport = new StdioClientTransport({ command: binary, env: { ...process.env } });
const client = new Client({ name: 'MCP performance acceptance', version: '1.0.0' });
await client.connect(transport);
await delay(200);
const id = (await state()).clients.find((c) => c.connected && !c.approved).id;
await control({ op: 'grant', client_id: id, permissions: ['read'], instances: [] });
const helper = await idle(transport.pid);
const start = await sample(host),
	helperStart = await sample(transport.pid);
const latencies = [];
for (let n = 0; n < 12; n++) {
	const t = performance.now();
	const result = await client.callTool({ name: 'tfl_get_app_info', arguments: {} });
	assert(!result.isError);
	latencies.push(performance.now() - t);
	await delay(200);
}
const end = await sample(host),
	helperEnd = await sample(transport.pid);
const result = {
	scenario:
		'Linux debug, windowless native Tauri integration harness; includes constant test-control polling. Not full launcher UI or release build.',
	disabled,
	enabled_no_client: enabled,
	stdio_helper_idle: helper,
	query_session: {
		latencies_ms: latencies,
		host_rss_kib: end.rss_kib,
		helper_rss_kib: helperEnd.rss_kib,
		host_cpu_one_core_percent:
			(100 * ((end.ticks - start.ticks) / tickRate)) / ((end.at - start.at) / 1000),
		helper_cpu_one_core_percent:
			(100 * ((helperEnd.ticks - helperStart.ticks) / tickRate)) /
			((helperEnd.at - helperStart.at) / 1000)
	},
	helper_binary_debug_bytes: (await stat(binary)).size
};
await client.close();
await control({ op: 'enabled', enabled: false });
await writeFile(path.join(root, 'performance.json'), JSON.stringify(result, null, 2));
console.log(JSON.stringify(result, null, 2));
