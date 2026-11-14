// Real production frontend, simulated IPC; observations are Chromium, not native GPU certification.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({ headless: true });
const url = process.env.VISUAL_URL || 'http://localhost:4175';
const output = process.env.LANDSCAPE_OUTPUT || 'artifacts/living-landscapes';
await mkdir(output, { recursive: true });
const results = [],
	errors = [];
async function open(preset, quality = 'Balanced', extra = {}) {
	const page = await browser.newPage({
		viewport: { width: 1280, height: 720 },
		deviceScaleFactor: extra.dpr || 1,
		reducedMotion: extra.reduced ? 'reduce' : 'no-preference'
	});
	page.on('pageerror', (error) => errors.push(error.message));
	await page.addInitScript(installFixture, { quality });
	await page.addInitScript(
		({ preset, noCanvas }) => {
			localStorage.setItem(
				'tfl-scenes-v1',
				JSON.stringify({
					version: 1,
					global: { preset, motion: 'full', fps: 24, parallax: 1, adaptTime: false },
					instances: {}
				})
			);
			window.__landscapeContextCount = 0;
			window.__landscapeSurfaces = new Set();
			window.__pendingFrames = new Set();
			const request = window.requestAnimationFrame.bind(window);
			const cancel = window.cancelAnimationFrame.bind(window);
			window.requestAnimationFrame = (callback) => {
				const id = request((time) => {
					window.__pendingFrames.delete(id);
					callback(time);
				});
				window.__pendingFrames.add(id);
				return id;
			};
			window.cancelAnimationFrame = (id) => {
				window.__pendingFrames.delete(id);
				cancel(id);
			};
			const getContext = HTMLCanvasElement.prototype.getContext;
			HTMLCanvasElement.prototype.getContext = function (type, ...args) {
				if (type === '2d') {
					window.__landscapeContextCount++;
					window.__landscapeSurfaces.add(this);
					if (noCanvas) return null;
				}
				return getContext.call(this, type, ...args);
			};
		},
		{ preset, noCanvas: extra.noCanvas }
	);
	await page.goto(url);
	await page.locator('.pixel-landscape').waitFor();
	await page.waitForFunction(() =>
		[...document.querySelectorAll('.pixel-landscape img')].every(
			(img) => img.complete && img.naturalWidth
		)
	);
	return page;
}
async function selectScene(page, config) {
	await page.evaluate((config) => {
		const value = JSON.stringify({
			version: 1,
			global: { motion: 'full', fps: 24, adaptTime: false, ...config },
			instances: {}
		});
		localStorage.setItem('tfl-scenes-v1', value);
		window.dispatchEvent(new StorageEvent('storage', { key: 'tfl-scenes-v1', newValue: value }));
	}, config);
}
try {
	for (const preset of ['midnight-forest', 'dithered-sunset', 'aurora-valley']) {
		for (const quality of ['Lite', 'Balanced', 'Experience']) {
			const page = await open(preset, quality, { dpr: quality === 'Experience' ? 2 : 1 });
			const canvas = page.locator('.pixel-landscape canvas');
			if (quality === 'Lite') {
				assert.equal(await canvas.count(), 0);
				assert.equal(await page.evaluate(() => window.__landscapeContextCount), 0);
			} else {
				await page.waitForFunction(
					() => document.querySelector('.pixel-landscape')?.dataset.animated === 'true'
				);
				const first = await canvas.evaluate((n) => n.toDataURL());
				await page.waitForTimeout(700);
				assert.notEqual(
					await canvas.evaluate((n) => n.toDataURL()),
					first,
					`${preset} has actual scene animation`
				);
				const count = await page.evaluate(() => window.__landscapeContextCount);
				for (let i = 0; i < 8; i++) await page.mouse.move(20 + i * 150, 150 + i * 30);
				assert.equal(
					await page.evaluate(() => window.__landscapeContextCount),
					count,
					'parallax does not recreate caches'
				);
				const geometry = await canvas.evaluate((n) => {
					const s = getComputedStyle(n);
					return {
						x: parseFloat(s.left),
						y: parseFloat(s.top),
						sx: parseFloat(s.width) / n.width,
						sy: parseFloat(s.height) / n.height,
						width: n.width,
						height: n.height
					};
				});
				assert.equal(geometry.width, 384);
				assert.equal(geometry.height, 216);
				for (const value of [geometry.x, geometry.y, geometry.sx, geometry.sy])
					assert.ok(Number.isInteger(value));
				await canvas.evaluate((n) =>
					n.dispatchEvent(new Event('contextlost', { cancelable: true }))
				);
				assert.equal(await page.locator('.pixel-landscape').getAttribute('data-animated'), 'false');
				await canvas.evaluate((n) => n.dispatchEvent(new Event('contextrestored')));
				await page.waitForFunction(
					() => document.querySelector('.pixel-landscape')?.dataset.animated === 'true'
				);
				assert.equal(
					await page.evaluate(() => window.__landscapeContextCount),
					count * 2,
					'restore creates exactly one cache set'
				);
				const cdp = await page.context().newCDPSession(page);
				await cdp.send('Performance.enable');
				const metrics = async () =>
					Object.fromEntries(
						(await cdp.send('Performance.getMetrics')).metrics.map((m) => [m.name, m.value])
					);
				const a = await metrics();
				await page.waitForTimeout(1000);
				const b = await metrics();
				results.push({
					preset,
					quality,
					jsHeap: b.JSHeapUsedSize,
					taskMsPerSecond: (b.TaskDuration - a.TaskDuration) * 1000,
					contexts: count
				});
			}
			await page.screenshot({ path: `${output}/${preset}-${quality.toLowerCase()}.png` });
			await page.close();
		}
	}
	for (const extra of [{ reduced: true }, { noCanvas: true }]) {
		const page = await open('midnight-forest', 'Experience', extra);
		assert.equal(await page.locator('.pixel-landscape[data-animated="true"]').count(), 0);
		if (extra.reduced) assert.equal(await page.evaluate(() => window.__landscapeContextCount), 0);
		await page.close();
	}
	const lifecycle = await open('midnight-forest', 'Experience');
	for (let cycle = 0; cycle < 12; cycle++) {
		await selectScene(lifecycle, { preset: cycle % 2 ? 'aurora-valley' : 'midnight-forest' });
		await lifecycle.waitForFunction(() =>
			document.querySelector('.pixel-landscape[data-animated="true"] canvas')
		);
		const active = await lifecycle.evaluate(
			() => [...window.__landscapeSurfaces].filter((n) => n.width > 0 && n.height > 0).length
		);
		assert.equal(active, cycle % 2 ? 9 : 8, 'only mounted scene owns live cached surfaces');
		await selectScene(lifecycle, { preset: 'legacy' });
		await lifecycle.waitForFunction(() => !document.querySelector('.pixel-landscape'));
		await lifecycle.waitForTimeout(60);
		assert.equal(
			await lifecycle.evaluate(
				() => [...window.__landscapeSurfaces].filter((n) => n.width > 0 && n.height > 0).length
			),
			0,
			'unmount releases all cached canvas backing stores'
		);
		assert.equal(
			await lifecycle.evaluate(() => window.__pendingFrames.size),
			0,
			'last scene unmount leaves no queued animation frame'
		);
	}
	await selectScene(lifecycle, { preset: 'midnight-forest', mediaId: 'builtin-aurora' });
	await lifecycle.waitForFunction(() => {
		const v = document.querySelector('.living-scene video');
		return v && !v.paused && v.readyState >= 2 && v.currentTime > 0;
	});
	assert.equal(
		await lifecycle.locator('.pixel-landscape canvas').count(),
		0,
		'playing video does not run a second procedural renderer'
	);
	assert.equal(
		await lifecycle.evaluate(
			() => [...window.__landscapeSurfaces].filter((n) => n.width > 0 && n.height > 0).length
		),
		0,
		'video releases procedural backing stores'
	);
	await lifecycle
		.locator('.living-scene video')
		.evaluate((n) => n.dispatchEvent(new Event('error')));
	await lifecycle.waitForFunction(() =>
		document.querySelector('.pixel-landscape[data-animated="true"] canvas')
	);
	assert.equal(await lifecycle.locator('.living-scene video').count(), 0);
	const beforeFallback = await lifecycle
		.locator('.pixel-landscape canvas')
		.evaluate((n) => n.toDataURL());
	await lifecycle.waitForTimeout(700);
	assert.notEqual(
		await lifecycle.locator('.pixel-landscape canvas').evaluate((n) => n.toDataURL()),
		beforeFallback,
		'media failure resumes real landscape animation'
	);
	await lifecycle.close();
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{
				browser: await browser.version(),
				scenario:
					'Chromium production frontend with isolated IPC; synthetic context-loss events; no native benchmarks',
				results
			},
			null,
			2
		)
	);
	console.log(
		'PASS Living landscapes: 3 scenes × 3 quality tiers, animation, DPR, cache reuse, context restoration, static/no-Canvas fallback, 12 mount/dispose cycles with zero queued RAF, real video and media-error recovery'
	);
} finally {
	await browser.close();
}
