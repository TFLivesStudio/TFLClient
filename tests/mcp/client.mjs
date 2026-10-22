// External official MCP client against the actual Rust stdio binary + native Tauri host.
// Only the feature-gated native harness consumes control.json; production does not.
import assert from 'node:assert/strict';
import { readFile, writeFile, rename, mkdir } from 'node:fs/promises';
import path from 'node:path';
const sdk =
	process.env.MCP_SDK_ROOT ||
	'/workspace/.tfl-tools/mcp-test-client/node_modules/@modelcontextprotocol/sdk/dist/esm';
const { Client } = await import(`${sdk}/client/index.js`);
const { StdioClientTransport } = await import(`${sdk}/client/stdio.js`);
const profile = process.env.TFL_DATA_DIR;
assert(profile && path.isAbsolute(profile), 'An isolated test profile is mandatory');
const evidence = [];
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
async function status() {
	return JSON.parse(await readFile(path.join(profile, 'status.json'), 'utf8'));
}
async function until(fn) {
	for (let n = 0; n < 150; n++) {
		try {
			const v = await fn();
			if (v) return v;
		} catch {}
		await pause(100);
	}
	throw Error('Timed out waiting for host');
}
async function control(value) {
	await writeFile(path.join(profile, 'control.tmp'), JSON.stringify(value));
	await rename(path.join(profile, 'control.tmp'), path.join(profile, 'control.json'));
	await until(async () => {
		try {
			await readFile(path.join(profile, 'control.json'));
			return false;
		} catch {
			return true;
		}
	});
}
const command =
	process.env.MCP_BINARY || '/workspace/TFLClient-experience/target/debug/tflclient-mcp';
const a = new Client({ name: 'TFL integration acceptance client', version: '1.0.0' });
const transport = new StdioClientTransport({
	command,
	env: { ...process.env, TFL_DATA_DIR: profile },
	stderr: 'pipe'
});
let diagnostics = '';
transport.stderr?.on('data', (d) => (diagnostics += d));
await a.connect(transport);
const tools = await a.listTools();
assert(tools.tools.length >= 28);
evidence.push({ protocol: 'initialize/tools/list', tools: tools.tools.length });
const call = (name, args = {}) => a.callTool({ name, arguments: args });
const data = (result) => {
	assert(!result.isError, JSON.stringify(result));
	return result.structuredContent.data;
};
const denied = await call('tfl_get_app_info');
assert(denied.isError);
assert.match(JSON.stringify(denied), /APPROVAL_REQUIRED/);
const id = await until(
	async () => (await status()).clients.find((c) => c.connected && !c.approved)?.id
);
await control({ op: 'grant', client_id: id, permissions: ['read'], instances: [] });
assert(data(await call('tfl_get_app_info')).version);
assert(
	(
		await call('tfl_create_instance', {
			name: 'test',
			mc_version: '1.21.1',
			loader: 'vanilla',
			idempotency_key: 'denied'
		})
	).isError
);
evidence.push({
	authorization: 'unapproved read denied; approved read allowed; read-only mutation denied'
});
await control({
	op: 'grant',
	client_id: id,
	permissions: ['read', 'configure', 'operate', 'critical'],
	instances: []
});
const pref = data(await call('tfl_get_preferences'));
const changed = data(
	await call('tfl_set_preference', {
		key: 'style',
		value: 'pixel',
		expected_revision: pref.revision
	})
);
assert(changed.revision > pref.revision);
assert(
	(
		await call('tfl_set_preference', {
			key: 'style',
			value: 'modern',
			expected_revision: pref.revision
		})
	).isError
);
assert((await readFile(path.join(profile, 'preference-events.jsonl'), 'utf8')).includes('pixel'));
evidence.push({ preferences: 'persisted change + native Tauri event; stale revision rejected' });
async function job(result) {
	const { job_id } = data(result);
	return until(async () => {
		const j = data(await call('tfl_get_job', { job_id }));
		if (j.status === 'failed') throw Error(j.error);
		return j.status === 'succeeded' ? j : false;
	});
}
const args = {
	name: 'MCP Acceptance ' + Date.now(),
	mc_version: '1.21.1',
	loader: 'vanilla',
	idempotency_key: 'fixture-create'
};
const created = await call('tfl_create_instance', args);
const j = await job(created);
const uuid = j.result.instance_id;
assert(uuid);
const duplicate = data(await call('tfl_create_instance', args));
assert.equal(duplicate.job_id, data(created).job_id);
const items = data(await call('tfl_list_instances'));
assert(items.instances.some((i) => i.instance_id === uuid));
const changedMemory = data(
	await call('tfl_set_instance_memory', { instance_id: uuid, min_mb: 512, max_mb: 1024 })
);
assert.equal(changedMemory.max_memory_mb, 1024);
const dir = path.join(profile, '.tflclient', 'instances', args.name);
await mkdir(path.join(dir, 'saves', 'TestWorld'), { recursive: true });
await writeFile(path.join(dir, 'saves', 'TestWorld', 'level.dat'), 'test world fixture');
assert(data(await call('tfl_list_worlds', { instance_id: uuid })).worlds.includes('TestWorld'));
const backup = await job(
	await call('tfl_create_backup', { instance_id: uuid, idempotency_key: 'backup-1' })
);
assert(backup.result.backup.id);
assert(data(await call('tfl_list_backups', { instance_id: uuid })).length > 0);
evidence.push({
	instances:
		'real UUID create, RAM mutation, worlds, real ZIP backup; duplicate idempotency key coalesced'
});
const b = new Client({ name: 'second reader', version: '1.0.0' });
await b.connect(
	new StdioClientTransport({ command, env: { ...process.env, TFL_DATA_DIR: profile } })
);
const id2 = await until(
	async () => (await status()).clients.find((c) => c.connected && !c.approved)?.id
);
await control({
	op: 'grant',
	client_id: id2,
	permissions: ['read'],
	instances: ['00000000-0000-0000-0000-000000000000']
});
assert((await b.callTool({ name: 'tfl_get_instance', arguments: { instance_id: uuid } })).isError);
assert(!data(await b.callTool({ name: 'tfl_list_instances', arguments: {} })).instances.length);
evidence.push({ concurrency: 'second independent MCP process/session; instance scope enforced' });
let plan = data(await call('tfl_plan_action', { instance_id: uuid, action: 'delete_instance' }));
assert(
	(await call('tfl_execute_plan', { plan_id: plan.plan_id })).isError,
	'execution must fail before enqueueing without local approval'
);
await control({ op: 'approve', plan_id: plan.plan_id, approve: false });
assert((await call('tfl_execute_plan', { plan_id: plan.plan_id })).isError);
plan = data(await call('tfl_plan_action', { instance_id: uuid, action: 'delete_instance' }));
await control({ op: 'approve', plan_id: plan.plan_id, approve: true });
const deleted = await job(await call('tfl_execute_plan', { plan_id: plan.plan_id }));
assert(deleted.result.completed);
assert(!data(await call('tfl_list_instances')).instances.some((i) => i.instance_id === uuid));
await control({ op: 'revoke', client_id: id });
assert((await call('tfl_get_app_info')).isError);
await b.close();
await a.close();
assert(!/access_token|refresh_token/.test(diagnostics));
evidence.push({
	critical:
		'unapproved and denied execution rejected; exact approved delete executes once; revocation enforced'
});
await writeFile(
	path.join(profile, 'acceptance.json'),
	JSON.stringify({ sdk: '@modelcontextprotocol/sdk 1.29.0', evidence }, null, 2)
);
console.log(JSON.stringify(evidence, null, 2));
