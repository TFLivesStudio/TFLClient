import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	DEFAULT_SCENE,
	DEFAULT_SCENES,
	parseScene,
	parseScenes,
	sceneCapabilities,
	dayPhase
} from '../../src/lib/scenes/model.ts';
import { SETTINGS, PAGES, searchSettings, setting } from '../../src/lib/settings/catalog.ts';
import { parseConsole } from '../../src/lib/console/model.ts';

test('scene preferences reject resource URLs, invalid IDs, prototype keys and unbounded work', () => {
	const parsed = parseScene({
		preset: 'arbitrary',
		mediaId: 'file:///etc/passwd',
		poster: 'url(secret)',
		fps: 99999,
		motion: 'yes',
		gamePolicy: 'run'
	});
	assert.deepEqual(parsed, DEFAULT_SCENE);
	assert.equal(
		parseScene({ preset: 'forest', mediaId: 'scene-123456abcdef', poster: 'mc-13', fps: 15 })
			.mediaId,
		'scene-123456abcdef'
	);
	const input = JSON.parse(
		'{"global":{"preset":"pixel"},"instances":{"__proto__":{"preset":"forest"},"ok":{"preset":"cave"}}}'
	);
	const prefs = parseScenes(input);
	assert.equal(prefs.instances.ok.preset, 'cave');
	assert.equal(Object.hasOwn(prefs.instances, '__proto__'), false);
	assert.deepEqual(parseScenes({ version: 999 }), DEFAULT_SCENES);
	assert.deepEqual(parseScenes(JSON.parse(JSON.stringify(prefs))), prefs);
});
test('scene motion policy gives accessibility, focus, quality and game budget priority', () => {
	const config = { ...DEFAULT_SCENE, preset: 'forest', shader: true, balancedVideo: true, fps: 60 };
	const base = { quality: 'experience', reduced: false, active: true, gameRunning: false, config };
	assert.equal(sceneCapabilities(base).shader, true);
	for (const change of [
		{ quality: 'lite' },
		{ reduced: true },
		{ active: false },
		{ gameRunning: true }
	]) {
		const cap = sceneCapabilities({ ...base, ...change });
		assert.equal(cap.video, false);
		assert.equal(cap.shader, false);
		assert.equal(cap.parallax, false);
		assert.equal(cap.fps, 0);
	}
	const balanced = sceneCapabilities({ ...base, quality: 'balanced' });
	assert.equal(balanced.shader, false);
	assert.equal(balanced.fps, 24);
	assert.equal(balanced.maxParticles, 8);
	assert.equal(
		sceneCapabilities({ ...base, quality: 'balanced', config: { ...config, balancedVideo: false } })
			.video,
		false
	);
	assert.equal(
		sceneCapabilities({ ...base, gameRunning: true, config: { ...config, gamePolicy: 'keep' } })
			.motion,
		true
	);
	assert.equal(sceneCapabilities({ ...base, config: { ...config, motion: 'off' } }).video, false);
});
test('time-of-day boundaries use device-local hours without location', () => {
	assert.equal(dayPhase(0), 'night');
	assert.equal(dayPhase(6), 'morning');
	assert.equal(dayPhase(11), 'day');
	assert.equal(dayPhase(17), 'sunset');
	assert.equal(dayPhase(20), 'night');
});
test('canonical settings support beginner/intermediate/expert deep tasks in both languages', () => {
	assert.equal(new Set(SETTINGS.map((s) => s.id)).size, SETTINGS.length);
	for (const s of SETTINGS) {
		assert.ok(PAGES.some((p) => p.id === s.page));
		assert.equal(setting(s.id), s);
	}
	for (const [query, id] of [
		['IDIOMA', 'language'],
		['color personalizado', 'accent'],
		['vídeo', 'scene-video'],
		['memoria', 'ram'],
		['microsoft', 'account-microsoft'],
		['jvm', 'jvm'],
		['variables entorno', 'environment'],
		['weather', 'scene-atmosphere'],
		['nucleos', 'console-metrics'],
		['actualizaciones', 'updates']
	])
		assert.ok(
			searchSettings(query, 'es').some((s) => s.id === id),
			`${query} => ${id}`
		);
	assert.equal(searchSettings('nothing matches qxz', 'en').length, 0);
	assert.equal(searchSettings('   ', 'es').length, 0);
});
test('beta 4 circular charts migrate and independent per-metric choices survive validation', () => {
	const old = parseConsole({ chartStyle: 'gauge', seconds: 180 });
	assert.deepEqual(old.metricStyles, { cpu: 'gauge', ram: 'gauge', disk: 'gauge' });
	assert.equal(old.metricSeconds.ram, 180);
	const current = parseConsole({
		...old,
		metricStyles: { cpu: 'lines', ram: 'gauge', disk: 'url(x)' },
		metricSeconds: { cpu: 30, ram: 60, disk: Infinity }
	});
	assert.equal(current.metricStyles.cpu, 'lines');
	assert.equal(current.metricStyles.disk, 'gauge');
	assert.equal(current.metricSeconds.cpu, 30);
	assert.equal(current.metricSeconds.disk, 180);
	assert.deepEqual(parseConsole(JSON.parse(JSON.stringify(current))), current);
});
test('shared frame scheduler owns one RAF and releases it when the last client stops', async () => {
	let pending = new Map(),
		id = 0;
	const oldRaf = globalThis.requestAnimationFrame,
		oldCancel = globalThis.cancelAnimationFrame;
	globalThis.requestAnimationFrame = (cb) => {
		pending.set(++id, cb);
		return id;
	};
	globalThis.cancelAnimationFrame = (key) => pending.delete(key);
	try {
		const { subscribeFrame, frameSubscriberCount } =
			await import('../../src/lib/scenes/frameClock.ts');
		let count = 0;
		const start = performance.now();
		const offA = subscribeFrame(() => count++, 24),
			offB = subscribeFrame(() => count++, 15);
		assert.equal(pending.size, 1);
		assert.equal(frameSubscriberCount(), 2);
		for (let i = 1; i <= 60; i++) {
			const callbacks = [...pending.values()];
			pending.clear();
			for (const cb of callbacks) cb(start + (i * 1000) / 60);
		}
		assert.ok(count >= 30 && count <= 40, count);
		offA();
		assert.equal(pending.size, 1);
		offB();
		assert.equal(pending.size, 0);
		assert.equal(frameSubscriberCount(), 0);
	} finally {
		globalThis.requestAnimationFrame = oldRaf;
		globalThis.cancelAnimationFrame = oldCancel;
	}
});

test('duplicate installs coalesce per instance and kind; a failure can be retried', async () => {
	const { coalesceInstall } = await import('../../src/lib/visual/installGuard.ts');
	let calls = 0,
		release;
	const wait = new Promise((r) => (release = r));
	const first = coalesceInstall('one/mod/project', async () => {
		calls++;
		await wait;
	});
	const duplicate = coalesceInstall('one/mod/project', async () => {
		calls++;
	});
	const other = coalesceInstall('two/mod/project', async () => {
		calls++;
	});
	await other;
	assert.equal(calls, 2);
	release();
	await Promise.all([first, duplicate]);
	assert.equal(calls, 2);
	await assert.rejects(
		coalesceInstall('one/mod/failure', async () => {
			throw Error('retry');
		})
	);
	await coalesceInstall('one/mod/failure', async () => {
		calls++;
	});
	assert.equal(calls, 3);
});
