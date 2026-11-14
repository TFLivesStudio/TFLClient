// Production UI + isolated IPC; real owner-approved packaged artwork.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
import { BANNERS } from '../../src/lib/visual/banners.ts';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const output = process.env.BACKGROUND_OUTPUT || 'artifacts/background-banners';
await mkdir(output, { recursive: true });
const errors = [],
	checks = [],
	observations = [];
async function open({
	style = 'modern',
	quality = 'Balanced',
	version = '1.21.1',
	artwork = 'auto',
	schema = 1,
	url = process.env.VISUAL_URL || 'http://127.0.0.1:4184'
} = {}) {
	const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
	page.on('pageerror', (e) => errors.push(e.message));
	await page.addInitScript(installFixture, { quality });
	const legacy = {
		version: schema,
		global: { preset: 'midnight-forest', scope: 'hero', motion: 'full', fps: 24, adaptTime: false },
		instances: {
			horizons: { preset: 'embers', scope: 'hero', fps: 15, particles: false },
			create: { preset: 'aurora', scope: 'window', motion: 'off' }
		}
	};
	await page.addInitScript(
		({ style, legacy, version, artwork }) => {
			localStorage.setItem('tfl-scenes-v1', JSON.stringify(legacy));
			localStorage.setItem(
				'tfl-visual-refresh-v1',
				JSON.stringify({ style, instances: { horizons: { artwork } } })
			);
			const invoke = window.__TAURI_INTERNALS__.invoke;
			window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
				const result = await invoke(cmd, args);
				if (cmd === 'get_instances')
					return result.map((i) => (i.uuid === 'horizons' ? { ...i, mc_version: version } : i));
				return result;
			};
			window.__sceneContexts = 0;
			const original = HTMLCanvasElement.prototype.getContext;
			HTMLCanvasElement.prototype.getContext = function (type, ...args) {
				if (type === '2d') window.__sceneContexts++;
				return original.call(this, type, ...args);
			};
		},
		{ style, legacy, version, artwork }
	);
	await page.goto(url);
	await page.locator('.hero-name').waitFor();
	return { page, legacy };
}
async function capture(page, name) {
	await page.waitForTimeout(180);
	await page.evaluate(() =>
		Promise.all(
			document
				.getAnimations()
				.filter((a) => a.effect?.getComputedTiming().iterations !== Infinity)
				.map((a) => a.finished.catch(() => {}))
		)
	);
	await page.screenshot({ path: `${output}/${name}.png` });
}
async function inspectBackground(page, node, contexts) {
	assert.equal(await page.locator('[data-scene-placement]').count(), 1, 'single shell-owned scene');
	assert.equal(
		await page.locator('.hero .stage,.instance-detail .stage').count(),
		0,
		'editorial Heroes contain no scene engines'
	);
	assert.ok(
		await page.evaluate(
			(node) => node === document.querySelector('[data-scene-placement] canvas'),
			node
		),
		'same canvas survives route/selection'
	);
	assert.equal(
		await page.evaluate(() => window.__sceneContexts),
		contexts,
		'navigation allocates no replacement renderer/cache'
	);
}
try {
	for (const style of ['modern', 'pixel']) {
		const { page, legacy } = await open({ style });
		await page.waitForFunction(() =>
			document.querySelector('[data-scene-placement="window"] canvas')
		);
		const migrated = await page.evaluate(() => ({
			backup: localStorage.getItem('tfl-scenes-v1-before-global-backgrounds'),
			current: JSON.parse(localStorage.getItem('tfl-scenes-v1'))
		}));
		assert.deepEqual(
			JSON.parse(migrated.backup),
			legacy,
			'untouched v1 backup preserves prior destinations/options'
		);
		assert.equal(migrated.current.version, 2);
		assert.equal(migrated.current.global.preset, 'midnight-forest');
		assert.equal(migrated.current.global.scope, 'window');
		assert.equal(migrated.current.instances.horizons.preset, 'embers');
		assert.equal(migrated.current.instances.horizons.fps, 15);
		assert.equal(migrated.current.instances.horizons.particles, false);
		const node = await page.locator('[data-scene-placement] canvas').elementHandle(),
			contexts = await page.evaluate(() => window.__sceneContexts);
		await capture(page, `${style}-home`);
		await inspectBackground(page, node, contexts);
		await page.getByRole('button', { name: 'Biblioteca', exact: true }).first().click();
		await page.locator('.vr-library-row').first().waitFor();
		await capture(page, `${style}-library`);
		await inspectBackground(page, node, contexts);
		await page.getByRole('button', { name: 'Cuadrícula', exact: true }).click();
		const card = page
			.locator('.vr-library-row')
			.filter({ has: page.getByRole('button', { name: /^Horizontes/ }) })
			.first();
		// Instance artwork must be independent from its archived embers ambience.
		const cardStyle = await card.getAttribute('style');
		assert.doesNotMatch(
			cardStyle || '',
			/scenes\/cinematic|embers|aurora/,
			'grid identity is artwork, not archived/global ambience'
		);
		await page.locator('.instance-open').filter({ hasText: 'Horizontes' }).click();
		await page.locator('.instance-detail').waitFor();
		await capture(page, `${style}-detail`);
		await inspectBackground(page, node, contexts);
		await page.getByRole('button', { name: 'Inicio', exact: true }).click();
		await page.locator('.hero-name').waitFor();
		await inspectBackground(page, node, contexts);
		await page.locator('.hero-detail').click();
		await page.locator('.instance-detail').waitFor();
		await inspectBackground(page, node, contexts);
		observations.push({
			style,
			quality: 'Balanced',
			contextsAfterNavigation: contexts,
			rendererRecreated: false,
			archivePreserved: true
		});
		checks.push(`${style}: v1 backup, global canvas survives Home/Library/Detail, no Hero engine`);
		await page.close();
	}
	for (const version of ['1.8.9', '1.16.5', '1.21.1', '26.3']) {
		const { page } = await open({ version });
		const hero = page.locator('.hero[data-banner-source]');
		assert.equal(await hero.getAttribute('data-banner-source'), 'exact');
		assert.equal(
			await hero.getAttribute('data-banner-id'),
			BANNERS.find((b) => b.exact?.includes(version)).id
		);
		assert.match(
			await hero.locator('.hero-art').evaluate((n) => getComputedStyle(n).backgroundImage),
			/banners\/optimized/
		);
		const first = await hero.getAttribute('data-banner-id');
		await page.reload();
		await page.locator('.hero-name').waitFor();
		assert.equal(
			await hero.getAttribute('data-banner-id'),
			first,
			'version artwork resolution is deterministic'
		);
		checks.push(
			`Minecraft ${version}: deterministic banner identity${BANNERS.length ? '' : ' (approved archive missing; safe fallback)'}`
		);
		await page.close();
	}
	for (const artwork of ['none', 'mc-2']) {
		const { page } = await open({ artwork });
		const hero = page.locator('.hero[data-banner-source]');
		assert.equal(await hero.getAttribute('data-banner-source'), 'custom');
		assert.equal(await hero.getAttribute('data-banner-id'), artwork);
		const background = await hero
			.locator('.hero-art')
			.evaluate((n) => getComputedStyle(n).backgroundImage);
		if (artwork === 'none') assert.equal(background, 'none');
		else assert.match(background, /mc-wallpaper-2\.jpg/);
		await page.locator('.hero-detail').click();
		await page.locator('.instance-detail').waitFor();
		const detail = page.locator('.instance-detail>.hero');
		assert.equal(
			await detail.evaluate((n) => n.classList.contains('with-art')),
			artwork !== 'none'
		);
		if (artwork === 'mc-2')
			assert.match(
				await detail.evaluate((n) => getComputedStyle(n).backgroundImage),
				/mc-wallpaper-2\.jpg/
			);
		checks.push(`explicit ${artwork}: preserved on Home and Detail`);
		await page.close();
	}
	const { page: future, legacy: futureValue } = await open({ schema: 3 });
	assert.deepEqual(
		await future.evaluate(() => JSON.parse(localStorage.getItem('tfl-scenes-v1'))),
		futureValue,
		'unknown future schema remains untouched'
	);
	await future.close();
	checks.push('unknown future schema preserved without destructive migration');
	const { page: decode } = await open();
	const decoded = await decode.evaluate(async (banners) => {
		return Promise.all(
			banners
				.flatMap((b) => [b.url, b.thumbnail])
				.map(async (url) => {
					const image = new Image();
					image.src = url;
					await image.decode();
					return { url, width: image.naturalWidth, height: image.naturalHeight };
				})
		);
	}, BANNERS);
	assert.equal(decoded.length, 50);
	assert.ok(decoded.every((i) => i.width > 0 && i.height > 0));
	observations.push({ packagedImagesDecoded: decoded.length });
	checks.push('all 25 optimized banners and 25 thumbnails decode');
	await decode.close();
	const { page: lite } = await open({ quality: 'Lite' });
	await lite.locator('[data-scene-placement] img').waitFor();
	assert.equal(await lite.locator('[data-scene-placement] canvas').count(), 0);
	assert.equal(
		await lite.evaluate(() => window.__sceneContexts),
		0,
		'Lite global scene creates no 2D context'
	);
	await capture(lite, 'modern-lite');
	await lite.close();
	checks.push('Lite: static global scene, zero drawing contexts');
	if (process.env.BASELINE_URL) {
		const { page } = await open({ url: process.env.BASELINE_URL });
		await capture(page, 'baseline-home');
		await page.getByRole('button', { name: 'Biblioteca', exact: true }).first().click();
		await capture(page, 'baseline-library');
		await page.locator('.instance-open').filter({ hasText: 'Horizontes' }).click();
		await page.locator('.instance-detail').waitFor();
		await capture(page, 'baseline-detail');
		await page.close();
	}
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{
				browser: await browser.version(),
				scenario: 'Chromium isolated IPC; no native performance certification',
				approvedArchiveAvailable: BANNERS.some((b) => b.available),
				checks,
				observations
			},
			null,
			2
		)
	);
	console.log(
		'PASS global background/banner: ' +
			checks.length +
			' checks; approved archive available=' +
			BANNERS.some((b) => b.available)
	);
} finally {
	await browser.close();
}
