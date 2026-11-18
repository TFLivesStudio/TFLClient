import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

function luminance(hex) {
	const rgb = [1, 3, 5].map((index) => Number.parseInt(hex.slice(index, index + 2), 16) / 255);
	const linear = rgb.map((value) =>
		value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4
	);
	return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
}

function contrast(a, b) {
	const values = [luminance(a), luminance(b)].sort((x, y) => y - x);
	return (values[0] + 0.05) / (values[1] + 0.05);
}

test('built-in accent labels meet normal text contrast at rest and on hover', async () => {
	const css = await readFile(new URL('../../src/styles/global.css', import.meta.url), 'utf8');
	const rules = [...css.matchAll(/([^{}]+)\{([^{}]*\n\s*--accent:\s*#[a-f\d]{6};[^{}]*)\}/gi)];
	assert.ok(rules.length >= 14, 'includes both theme defaults and every built-in accent');
	for (const [, selector, body] of rules) {
		const color = (key) => new RegExp(`--${key}:\\s*(#[a-f\\d]{6});`, 'i').exec(body)?.[1];
		const label = color('accent-text');
		for (const key of ['accent', 'accent-hover']) {
			const background = color(key);
			assert.ok(label && background, `${selector.trim()} has the required tokens`);
			assert.ok(
				contrast(label, background) >= 4.5,
				`${selector.trim()} ${key} ${label} on ${background}`
			);
		}
	}
});
