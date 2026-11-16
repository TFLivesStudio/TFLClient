import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	LANDSCAPE_IDS,
	LANDSCAPE_PALETTES,
	BAYER_4,
	landscapeLayers,
	landscapePoster,
	skyColor,
	skyDitherLevel,
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
test('sky dithering occupies only the final 30% of a palette transition and SVG matches Canvas texels', () => {
	for (const value of [0, 0.3, 0.69, 0.7, 1, 1.7, 3.7]) assert.equal(skyDitherLevel(value), 0);
	assert.equal(skyDitherLevel(0.85), 8);
	assert.equal(skyDitherLevel(0.999), 16);
	for (const id of LANDSCAPE_IDS) {
		const svg = decodeURIComponent(landscapePoster(id).split(',').slice(1).join(','));
		const rows = new Map();
		for (const match of svg.matchAll(
			/<rect y="(\d+)" width="384" height="1" fill="(#[0-9a-f]+)"(?: mask="url\(#m(\d+)\)")?\/>/g
		)) {
			const y = Number(match[1]);
			const row = rows.get(y) ?? {};
			if (match[3] === undefined) row.base = match[2];
			else {
				row.next = match[2];
				row.level = Number(match[3]);
			}
			rows.set(y, row);
		}
		assert.equal(rows.size, 216);
		let quietRows = 0;
		for (let y = 0; y < 216; y++) {
			const value = Math.min(4, (y / 155) * 4);
			const fraction = value - Math.floor(value);
			const row = rows.get(y);
			if (fraction <= 0.7) {
				assert.equal(row.next, undefined, 'no Bayer pattern outside the final transition segment');
				quietRows++;
			}
			for (let x = 0; x < 8; x++) {
				const expected =
					row.next && BAYER_4[((y & 3) << 2) + (x & 3)] < row.level ? row.next : row.base;
				assert.equal(skyColor(id, x, y), expected, `${id} sky texel ${x},${y}`);
				assert.equal(skyColor(id, x, y, false), LANDSCAPE_PALETTES[id].sky[Math.floor(value)]);
			}
		}
		assert.ok(quietRows >= 170, 'most sky rows remain calm solid palette fields');
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
