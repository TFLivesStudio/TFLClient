import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	DEFAULT_SCENE,
	SCENE_IDS,
	SCENES,
	parseScene,
	parseScenes,
	sceneCapabilities
} from '../../src/lib/scenes/model.ts';

test('v1 scenes migrate additively without changing saved backgrounds, videos or instance overrides', () => {
	const old = {
		version: 1,
		global: { preset: 'pixel', mediaId: 'builtin-aurora', motion: 'off', poster: 'mc-4' },
		instances: { minecraft: { preset: 'forest', fps: 15, gamePolicy: 'pause' } }
	};
	const migrated = parseScenes(old);
	assert.equal(migrated.version, 1);
	assert.equal(migrated.global.preset, 'pixel');
	assert.equal(migrated.global.mediaId, 'builtin-aurora');
	assert.equal(migrated.global.poster, 'mc-4');
	assert.equal(migrated.global.motion, 'off');
	assert.equal(migrated.global.scope, 'hero');
	assert.equal(migrated.global.dithering, true);
	assert.equal(migrated.instances.minecraft.preset, 'forest');
	assert.equal(migrated.instances.minecraft.fps, 15);
	assert.equal(migrated.instances.minecraft.gamePolicy, 'pause');
	assert.deepEqual(parseScenes(JSON.parse(JSON.stringify(migrated))), migrated);
});

test('new landscape settings round trip and reject URLs, malformed scope and nonfinite depth', () => {
	for (const preset of ['midnight-forest', 'dithered-sunset', 'aurora-valley']) {
		assert.ok(SCENE_IDS.includes(preset));
		assert.ok(SCENES.some((scene) => scene.id === preset));
		const scene = parseScene({ preset, scope: 'window', parallax: 0, dithering: false });
		assert.equal(scene.preset, preset);
		assert.equal(scene.scope, 'window');
		assert.equal(scene.parallax, 0);
		assert.equal(scene.dithering, false);
		assert.deepEqual(parseScene(JSON.parse(JSON.stringify(scene))), scene);
	}
	for (const parallax of [NaN, Infinity, '1', null]) {
		assert.equal(parseScene({ parallax }).parallax, DEFAULT_SCENE.parallax);
	}
	assert.equal(parseScene({ parallax: -4 }).parallax, 0);
	assert.equal(parseScene({ parallax: 500 }).parallax, 1);
	assert.equal(parseScene({ scope: 'file:///private' }).scope, 'hero');
	assert.equal(parseScene({ preset: 'https://example.com/theme' }).preset, 'legacy');
});

test('zero parallax does not disable animation and performance constraints take precedence', () => {
	const config = { ...DEFAULT_SCENE, preset: 'midnight-forest', parallax: 0, fps: 60 };
	const base = { quality: 'experience', active: true, reduced: false, gameRunning: false, config };
	const cap = sceneCapabilities(base);
	assert.equal(cap.motion, true);
	assert.equal(cap.parallax, false);
	assert.equal(cap.depth, 0);
	assert.equal(cap.fps, 60);
	assert.equal(sceneCapabilities({ ...base, quality: 'balanced' }).fps, 24);
	for (const override of [
		{ quality: 'lite' },
		{ reduced: true },
		{ active: false },
		{ gameRunning: true }
	]) {
		const paused = sceneCapabilities({ ...base, ...override });
		assert.equal(paused.fps, 0);
		assert.equal(paused.parallax, false);
		assert.equal(paused.video, false);
	}
	assert.equal(
		sceneCapabilities({ ...base, config: { ...config, parallax: 1, motion: 'full' } }).depth,
		1
	);
});
