import assert from 'node:assert/strict';
import test from 'node:test';
import { connectActivity, terminal } from '../../src/lib/activity/contract.ts';
const tick = () => new Promise((resolve) => setTimeout(resolve, 0));
const snapshot = (revision) => ({ revision, tasks: [], storage_warning: false });
test('subscribe before snapshot and requery changes during a pending snapshot', async () => {
	let changed,
		resolveLoad,
		subscribed = false,
		calls = 0,
		stopped = 0;
	const seen = [];
	const connection = connectActivity(
		async (callback) => {
			changed = callback;
			subscribed = true;
			return () => stopped++;
		},
		async () => {
			assert(subscribed);
			calls++;
			if (calls === 1)
				return new Promise((resolve) => {
					resolveLoad = resolve;
				});
			return snapshot(2);
		},
		(value) => seen.push(value.revision),
		() => assert.fail('unexpected sync error')
	);
	await tick();
	changed();
	resolveLoad(snapshot(1));
	await tick();
	assert.deepEqual(seen, [1, 2]);
	assert.equal(calls, 2);
	connection.stop();
	assert.equal(stopped, 1);
});
test('late listener registration and snapshot cannot leak after disposal', async () => {
	let register,
		stopped = 0,
		loads = 0;
	const connection = connectActivity(
		() =>
			new Promise((resolve) => {
				register = resolve;
			}),
		async () => {
			loads++;
			return snapshot(1);
		},
		() => assert.fail('disposed'),
		() => assert.fail('error')
	);
	connection.stop();
	register(() => stopped++);
	await tick();
	assert.equal(stopped, 1);
	assert.equal(loads, 0);
});
test('stale revisions ignored, failure can resync and terminal states explicit', async () => {
	let value = 3,
		fails = false,
		errors = 0;
	const seen = [];
	const connection = connectActivity(
		async () => () => {},
		async () => {
			if (fails) throw Error('offline');
			return snapshot(value);
		},
		(s) => seen.push(s.revision),
		() => errors++
	);
	await tick();
	value = 2;
	await connection.sync();
	assert.deepEqual(seen, [3]);
	fails = true;
	await connection.sync();
	assert.equal(errors, 1);
	fails = false;
	value = 4;
	await connection.sync();
	assert.deepEqual(seen, [3, 4]);
	connection.stop();
	for (const status of ['succeeded', 'failed', 'cancelled', 'interrupted'])
		assert(terminal(status));
	for (const status of ['queued', 'running', 'cancel_requested']) assert(!terminal(status));
});
