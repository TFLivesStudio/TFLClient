import { test } from 'node:test';
import assert from 'node:assert/strict';
import { minecraftFamily, resolveBanner, BANNERS } from '../../src/lib/visual/banners.ts';
import { parsePreferences } from '../../src/lib/visual/policy.ts';
test('version families are explicit and unknown/snapshot versions never invent associations', () => {
	assert.equal(minecraftFamily('1.8.9'), '1.8');
	assert.equal(minecraftFamily('1.21.11'), '1.21');
	assert.equal(minecraftFamily('26.3.1'), '26.3');
	for (const version of ['26w01a', 'unknown', '../../1.8', 'file:///banner', '1.8.9;bad'])
		assert.equal(minecraftFamily(version), null);
});
test('explicit safe artwork wins and unknown resources use a declared bundled fallback', () => {
	assert.equal(resolveBanner('1.8.9', 'mc-2').url, '/wallpapers/mc-wallpaper-2.jpg');
	assert.equal(resolveBanner('1.8.9', 'none').url, null);
	for (const path of ['https://evil/banner.png', 'file:///private', '../../secret', 'mc-999'])
		assert.equal(resolveBanner('unknown', path).source, 'fallback');
	for (const banner of BANNERS) {
		assert.ok(banner.url.startsWith('/banners/'));
		assert.equal(banner.url.includes('..'), false);
		assert.match(banner.position, /^(?:\d+%|center)(?: (?:\d+%|center))?$/);
	}
});

test('all approved exact versions resolve, family fallback is declared, and custom approved art persists', () => {
	assert.equal(BANNERS.length, 25);
	for (const banner of BANNERS)
		for (const version of banner.exact) {
			assert.equal(resolveBanner(version).id, banner.id);
			assert.equal(resolveBanner(version).source, 'exact');
		}
	assert.equal(resolveBanner('1.16.4').id, 'banner-1.16.5');
	assert.equal(resolveBanner('1.21.8').id, 'banner-1.21.11');
	assert.equal(resolveBanner('26.3.9').id, 'banner-26.3');
	assert.equal(resolveBanner('1.7.10').source, 'fallback');
	assert.equal(resolveBanner('1.8.9', 'banner-26.1').id, 'banner-26.1');
	assert.equal(
		parsePreferences({ instances: { a: { artwork: 'banner-26.1' } } }).instances.a.artwork,
		'banner-26.1'
	);
});
