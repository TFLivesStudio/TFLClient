import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	resolveCapabilities,
	parsePreferences,
	artworkUrl,
	accentText
} from '../../src/lib/visual/policy.ts';
const base = {
	quality: 'experience',
	reducedMotion: false,
	systemReducedMotion: false,
	noBlur: false,
	noInfiniteFx: false,
	active: true
};
test('Lite prevents automatic 3D, blur, ambient, parallax and motion', () => {
	const c = resolveCapabilities({ ...base, quality: 'lite' });
	for (const k of ['motion', 'viewer', 'blur', 'ambient', 'parallax', 'animateViewer'])
		assert.equal(c[k], false, k);
	assert.equal(c.duration, 0);
});
test('both reduced motion sources and inactive windows override Experience', () => {
	for (const patch of [{ reducedMotion: true }, { systemReducedMotion: true }, { active: false }]) {
		const c = resolveCapabilities({ ...base, ...patch });
		for (const k of ['motion', 'ambient', 'parallax', 'animateViewer'])
			assert.equal(c[k], false, k);
	}
});
test('independent toggles disable their costly capabilities', () => {
	const c = resolveCapabilities({ ...base, noBlur: true, noInfiniteFx: true });
	assert.equal(c.blur, false);
	assert.equal(c.ambient, false);
	assert.equal(c.animateViewer, false);
	assert.equal(c.parallax, true);
});
test('persisted metadata is bounded and rejects arbitrary resource URLs', () => {
	const p = parsePreferences({
		style: 'wrong',
		customAccent: 'url(https://invalid)',
		groups: [' Friends ', 'Friends', ''],
		instances: JSON.parse(
			'{"__proto__":{"favorite":true},"id":{"favorite":true,"group":"missing","artwork":"https://invalid"}}'
		)
	});
	assert.equal(p.style, 'modern');
	assert.equal(p.customAccent, '');
	assert.deepEqual(p.groups, ['Friends']);
	assert.equal(Object.hasOwn(p.instances, '__proto__'), false);
	assert.deepEqual(p.instances.id, { favorite: true, group: '', artwork: 'auto' });
	assert.equal(artworkUrl('../secret'), null);
	assert.equal(artworkUrl('mc-13'), '/wallpapers/mc-wallpaper-13.jpg');
	assert.equal(artworkUrl('mc-14'), null);
});
test('valid preferences round trip and custom accents use legible foreground', () => {
	const p = parsePreferences({
		style: 'pixel',
		customAccent: '#ff7a2e',
		groups: ['Survival'],
		instances: { a: { favorite: true, group: 'Survival', artwork: 'mc-3' } }
	});
	assert.deepEqual(parsePreferences(JSON.parse(JSON.stringify(p))), p);
	assert.equal(accentText('#ffffff'), '#000000');
	assert.equal(accentText('#000000'), '#ffffff');
});
