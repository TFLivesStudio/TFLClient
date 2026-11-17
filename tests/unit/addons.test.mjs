import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
	contributionId,
	enabledAddons,
	findScene,
	findTheme,
	validThemeTokens,
	contrastRatio,
	themeContrastIssues,
	advanceAddonLayer,
	readableThemeAccent,
	themeRawAliases,
	readableSemanticColors
} from '../../src/lib/addons/model.ts';

const tokens = {
	'--bg-main': '#101820',
	'--bg-sidebar': '#101820',
	'--bg-card': '#16222c',
	'--bg-input': '#101820',
	'--bg-item-active': '#22313f',
	'--text-primary': '#ffffff',
	'--text-secondary': '#c8d8e8',
	'--text-muted': '#a8bdd0',
	'--accent': '#ff7a2e',
	'--accent-text': '#101010'
};
const addon = {
	manifest: {
		id: 'example.dawn',
		themes: [{ id: 'night', name: 'Night', tokens }],
		scenes: [{ id: 'hills', name: 'Hills', layers: [] }]
	},
	enabled: true
};
test('only enabled declarative contributions resolve, safe profile fails closed', () => {
	const catalog = { safeMode: false, addons: [addon] };
	assert.equal(findScene(catalog, contributionId('example.dawn', 'hills')).addon, addon);
	assert.equal(findTheme(catalog, 'addon:example.dawn:night').theme.id, 'night');
	assert.equal(findScene(catalog, 'addon:someone.else:hills'), null);
	assert.deepEqual(enabledAddons({ ...catalog, safeMode: true }), []);
	assert.equal(findTheme({ ...catalog, safeMode: true }, 'addon:example.dawn:night'), null);
	assert.equal(
		findScene(
			{ safeMode: false, addons: [{ ...addon, enabled: false }] },
			'addon:example.dawn:hills'
		),
		null
	);
});
test('theme tokens cannot introduce CSS, URLs, arbitrary custom properties or scripts', () => {
	assert.equal(validThemeTokens(tokens), true);
	for (const value of [
		'url(https://evil.example)',
		'#ffffff;display:none',
		'var(--secret)',
		'expression(alert(1))',
		'#fff',
		'red'
	])
		assert.equal(validThemeTokens({ '--bg-main': value }), false);
	assert.equal(validThemeTokens({ '--unbounded-host-property': '#000000' }), false);
});
test('addon theme selection checks normal text on every stable surface', () => {
	assert.equal(contrastRatio('#000000', '#ffffff'), 21);
	assert.equal(contrastRatio('rgb(0, 0, 0)', 'rgb(255, 255, 255)'), 21);
	assert.equal(contrastRatio('transparent', '#ffffff'), 0);
	assert.ok(contrastRatio('#ffffff', 'rgba(0,0,0,0.9)') > 4.5);
	assert.ok(contrastRatio('#ffffff', 'rgba(0,0,0,0.2)') < 4.5);
	assert.deepEqual(themeContrastIssues(tokens, {}), []);
	assert.ok(
		themeContrastIssues({ '--text-muted': '#22313f' }, tokens).includes(
			'--text-muted / --bg-item-active'
		),
		'unreadable metadata must be rejected'
	);
	assert.ok(
		themeContrastIssues({ '--bg-card': '#ffffff' }, tokens).includes('--text-primary / --bg-card')
	);
	assert.ok(
		themeContrastIssues({ '--accent-text': '#ff7a2e' }, tokens).includes('--accent-text / --accent')
	);
});
test('declarative layer motion travels right with bounded modular phases and stable pause', () => {
	assert.equal(advanceAddonLayer(10, 0, 12, 0.5, 384), 10);
	assert.ok(advanceAddonLayer(10, 0.05, 12, 0.5, 384) > 10);
	assert.ok(advanceAddonLayer(10, 0.05, 12, 0.8, 384) > advanceAddonLayer(10, 0.05, 12, 0.2, 384));
	assert.ok(Math.abs(advanceAddonLayer(383.98, 0.1, 1, 0, 384) - 0.02) < 0.00001);
	assert.equal(advanceAddonLayer(10, 600, 12, 0.5, 384), advanceAddonLayer(10, 0.1, 12, 0.5, 384));
	assert.equal(advanceAddonLayer(10, 0.1, 0, 0.5, 384), 10);
	assert.equal(advanceAddonLayer(10, 0.1, 10, 0.5, 0), 0);
});
test('full dark addon themes derive readable accent text independently of host appearance', () => {
	const full = JSON.parse(
		readFileSync(
			new URL('../../examples/addons/orange-dunes/manifest.json', import.meta.url),
			'utf8'
		)
	).themes[0].tokens;
	assert.ok(
		contrastRatio('#a53c00', full['--bg-main']) < 4.5,
		'old light-only accent text fails on this dark addon'
	);
	const light = {
		...tokens,
		...Object.fromEntries(
			['--bg-main', '--bg-sidebar', '--bg-card', '--bg-input', '--bg-item-active'].map((key) => [
				key,
				'#ffffff'
			])
		),
		'--text-primary': '#101010'
	};
	for (const host of [tokens, light]) {
		const accent = readableThemeAccent(full, host);
		assert.equal(accent, full['--accent']);
		for (const key of ['--bg-main', '--bg-sidebar', '--bg-card', '--bg-input', '--bg-item-active'])
			assert.ok(contrastRatio(accent, full[key]) >= 4.5);
	}
	assert.equal(readableThemeAccent({ '--accent': '#ff7a2e' }, light), '#101010');
	assert.equal(
		validThemeTokens({ '--text-accent': '#101010' }),
		false,
		'derived host token is not an addon capability'
	);
});
test('theme materials derive only explicit background tokens and preserve partial host palettes', () => {
	assert.deepEqual(themeRawAliases({ '--accent': '#ff7a2e' }), {});
	assert.deepEqual(
		themeRawAliases({ '--bg-card': '#251e29', '--raw-main': '#ffffff', '--bg-main': '#17131b' }),
		{ '--raw-main': '#17131b', '--raw-card': '#251e29' }
	);
	for (const key of Object.keys(themeRawAliases(tokens)))
		assert.equal(validThemeTokens({ [key]: '#101010' }), false);
});
test('semantic alert colors follow effective surfaces and keep RGB channels coherent', () => {
	const full = JSON.parse(
		readFileSync(
			new URL('../../examples/addons/orange-dunes/manifest.json', import.meta.url),
			'utf8'
		)
	).themes[0].tokens;
	const surfaces = ['--bg-main', '--bg-sidebar', '--bg-card', '--bg-input', '--bg-item-active'].map(
		(key) => full[key]
	);
	const colors = readableSemanticColors(surfaces, full['--text-primary'], {
		'--color-error': '#b42331',
		'--color-success': '#16713b',
		'--color-warning': '#805500'
	});
	for (const key of ['--color-error', '--color-success', '--color-warning']) {
		assert.ok(surfaces.every((surface) => contrastRatio(colors[key], surface) >= 4.5));
		assert.equal(validThemeTokens({ [key]: colors[key] }), false);
	}
	assert.equal(
		colors['--color-error-rgb'],
		[1, 3, 5].map((i) => parseInt(colors['--color-error'].slice(i, i + 2), 16)).join(', ')
	);
	const fallback = readableSemanticColors(['#000000', '#ffffff'], '#767676', {});
	assert.equal(fallback['--color-error'], '#767676');
	assert.equal(fallback['--color-error-rgb'], '118, 118, 118');
});
