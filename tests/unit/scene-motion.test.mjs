import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createLayerMotion, layerSpeed, sceneDelta } from '../../src/lib/scenes/motion.ts';
import {
	LANDSCAPE_IDS,
	LANDSCAPE_PERIOD,
	ridgeHeight,
	landscapeLayers
} from '../../src/lib/scenes/landscape.ts';
import {
	DEFAULT_SCENE,
	parseScenes,
	parseScene,
	sceneCapabilities
} from '../../src/lib/scenes/model.ts';

test('automatic terrain moves right at calibrated logical pixels/second and wraps without a temporal jump', () => {
	for (const depth of [0, 0.2, 0.4, 1])
		for (const layer of [0.15, 0.32, 0.49, 0.5, 0.75, 1]) {
			const speed = layerSpeed(depth, layer);
			const motion = createLayerMotion(LANDSCAPE_PERIOD, layer, false, LANDSCAPE_PERIOD - 0.01);
			const next = motion.advance(0.1, depth);
			assert.ok(next >= 0 && next < LANDSCAPE_PERIOD);
			assert.ok(Math.abs(next - (speed * 0.1 - 0.01)) < 1e-9);
			for (let i = 0; i < 100; i++) motion.advance(0.1, depth);
			assert.ok(Math.abs(motion.offset - ((speed * 10.1 - 0.01) % LANDSCAPE_PERIOD)) < 1e-8);
		}
	assert.ok(layerSpeed(1, 1) > layerSpeed(1, 0.15));
	assert.equal(layerSpeed(0, 1), layerSpeed(0, 0.15));
});
test('pause, resume, malformed delta and pixel quantisation remain bounded', () => {
	const motion = createLayerMotion(768, 1, true, 767.9);
	assert.equal(motion.advance(0, 1), 767);
	assert.equal(motion.advance(NaN, 1), 767);
	assert.equal(motion.advance(-1, 1), 767);
	assert.equal(motion.advance(100000, 1), 0);
	assert.equal(sceneDelta(Infinity), 0);
	const saved = motion.phase;
	const restored = createLayerMotion(768, 1, true, saved);
	assert.equal(restored.advance(0.05, 1), motion.advance(0.05, 1));
});
test('every mountain and ground layer joins spatially, including translated copies and all viewport ratios', () => {
	for (const id of LANDSCAPE_IDS) {
		for (let l = 0; l < 3; l++)
			for (let x = -9; x <= 9; x += 3)
				assert.equal(ridgeHeight(id, l, x), ridgeHeight(id, l, x + LANDSCAPE_PERIOD));
		for (const l of landscapeLayers(id).filter((_, i) => i < 3)) {
			const ridge = l.shapes[0].points;
			assert.equal(ridge[1][1], ridge.at(-2)[1]);
		}
	}
	for (const [width, height] of [
		[800, 600],
		[1100, 700],
		[1920, 1080],
		[3440, 1440]
	]) {
		const scale = Math.max(1, Math.ceil(width / 384), Math.ceil(height / 216));
		assert.ok(384 * scale >= width && 216 * scale >= height);
		assert.ok(LANDSCAPE_PERIOD * scale > width, 'two adjacent tiles cover every modular phase');
	}
});
test('all scene preview/landscape and hero paths have no pointer-derived terrain displacement', () => {
	for (const path of [
		'src/lib/scenes/SceneStage.svelte',
		'src/lib/scenes/SceneFrame.svelte',
		'src/lib/scenes/PixelLandscape.svelte',
		'src/lib/visual/motion.ts',
		'src/lib/components/home/InstanceHero.svelte'
	]) {
		const code = readFileSync(new URL(`../../${path}`, import.meta.url), 'utf8');
		assert.doesNotMatch(code, /pointermove|use:parallax|--pointer-[xy]/);
	}
});
test('saved depth stays lossless, explicit static artwork persists, policy overrides automatic motion', () => {
	const old = {
		version: 2,
		global: { ...DEFAULT_SCENE, parallax: 0.7, preset: 'forest' },
		instances: { a: { preset: 'pixel', parallax: 0.3 } }
	};
	const migrated = parseScenes(old);
	assert.equal(migrated.global.parallax, 0.7);
	assert.equal(migrated.instances.a.parallax, 0.3);
	assert.equal(parseScene({ poster: 'cinematic' }).poster, 'cinematic');
	assert.equal(
		parseScene({ preset: 'addon:org.example:valley' }).preset,
		'addon:org.example:valley'
	);
	for (const preset of ['addon:org.demo--:night--', 'addon:a.b:scene-2'])
		assert.equal(
			parseScene({ preset }).preset,
			preset,
			'same slug contract as the native installer'
		);
	for (const preset of ['addon:../../x:y', 'addon:org.example:<script>'])
		assert.equal(parseScene({ preset }).preset, 'legacy');
	for (const override of [
		{ quality: 'lite' },
		{ reduced: true },
		{ active: false },
		{ gameRunning: true }
	]) {
		const caps = sceneCapabilities({
			config: DEFAULT_SCENE,
			quality: 'experience',
			active: true,
			reduced: false,
			gameRunning: false,
			...override
		});
		assert.equal(caps.fps, 0);
	}
});
