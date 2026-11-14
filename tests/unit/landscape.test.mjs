import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	LANDSCAPE_IDS,
	BAYER_4,
	landscapeLayers,
	landscapePoster,
	skyColor,
	isLandscape
} from '../../src/lib/scenes/landscape.ts';

test('procedural landscapes use deterministic bounded original geometry', () => {
	for (const id of LANDSCAPE_IDS) {
		const layers = landscapeLayers(id);
		assert.deepEqual(layers, landscapeLayers(id));
		assert.ok(layers.length >= 6 && layers.length <= 7);
		assert.ok(new Set(layers.map((l) => l.depth)).size >= 5);
		for (const layer of layers)
			for (const shape of layer.shapes) {
				for (const values of shape.points ?? [shape.rect]) assert.ok(values.every(Number.isFinite));
			}
	}
});
test('Bayer threshold has all sixteen levels and sky is spatially stable', () => {
	assert.deepEqual(
		[...BAYER_4].sort((a, b) => a - b),
		Array.from({ length: 16 }, (_, i) => i)
	);
	for (const id of LANDSCAPE_IDS) {
		for (let x = 0; x < 4; x++) assert.equal(skyColor(id, x, 27), skyColor(id, x + 4, 27));
		assert.equal(skyColor(id, 0, 0), skyColor(id, 0, -100));
		assert.equal(skyColor(id, 0, 216), skyColor(id, 0, 999));
	}
});
test('Lite posters need no DOM, canvas or external resources and retain distinct scenes', () => {
	const posters = LANDSCAPE_IDS.map((id) => landscapePoster(id));
	assert.equal(new Set(posters).size, 3);
	for (const poster of posters) {
		assert.ok(poster.startsWith('data:image/svg+xml,'));
		const svg = decodeURIComponent(poster.split(',').slice(1).join(','));
		assert.match(svg, /shape-rendering="crispEdges"/);
		assert.doesNotMatch(svg, /<script|<image|<foreignObject|href=|https?:\/\/(?!www.w3.org)/);
		assert.ok(poster.length < 128 * 1024, 'bounded inline original art, below 128 KiB');
	}
	assert.equal(landscapePoster('https://example.com/x'), '');
	assert.equal(isLandscape('pixel'), false);
	assert.notEqual(landscapePoster('midnight-forest'), landscapePoster('midnight-forest', false));
});
