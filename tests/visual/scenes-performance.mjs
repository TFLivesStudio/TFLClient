// Selected-scene Chromium observations; not native CPU/GPU/RSS measurements.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined,
	args: ['--enable-unsafe-swiftshader']
});
const output = process.env.PERFORMANCE_OUTPUT || 'artifacts/mcp/performance';
await mkdir(output, { recursive: true });
const samples = [];
try {
	for (const [revision, url, version] of [
		['baseline', process.env.BASELINE_URL, '0.14.0-beta.1'],
		['refresh', process.env.VISUAL_URL, '0.14.0-beta.2']
	])
		for (const quality of ['Lite', 'Balanced', 'Experience'])
			for (const scene of ['forest', 'embers', 'void', 'aurora', 'pixel']) {
				const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
				await page.addInitScript(installFixture, { quality, appVersion: version });
				await page.addInitScript(
					(scene) =>
						localStorage.setItem(
							'tfl-scenes-v1',
							JSON.stringify({
								version: 1,
								global: { preset: scene, motion: 'subtle', shader: false },
								instances: {}
							})
						),
					scene
				);
				const cdp = await page.context().newCDPSession(page);
				await cdp.send('Performance.enable');
				await page.goto(url);
				await page.locator('.hero').waitFor();
				await page.waitForFunction(() =>
					[...document.querySelectorAll('.scene-frame img')].every(
						(n) => n.complete && n.naturalWidth
					)
				);
				const metrics = async () =>
					Object.fromEntries(
						(await cdp.send('Performance.getMetrics')).metrics.map((m) => [m.name, m.value])
					);
				const a = await metrics();
				await page.waitForTimeout(1000);
				const b = await metrics();
				const layers = await page.evaluate(() => ({
					video: document.querySelectorAll('.living-scene video').length,
					webgl: document.querySelectorAll('.living-scene canvas').length,
					atmosphere: document.querySelectorAll('.living-scene .atmosphere').length,
					images: document.querySelectorAll('.scene-frame img').length
				}));
				if (quality === 'Lite') {
					assert.equal(layers.video, 0);
					assert.equal(layers.webgl, 0);
					assert.equal(layers.atmosphere, 0);
				}
				samples.push({
					revision,
					quality,
					scene,
					task_ms_per_one_second: (b.TaskDuration - a.TaskDuration) * 1000,
					js_heap_bytes: b.JSHeapUsedSize,
					...layers
				});
				await page.close();
			}
	await writeFile(
		`${output}/scenes.json`,
		JSON.stringify(
			{
				browser: await browser.version(),
				scenario:
					'Chromium synthetic IPC; no shader/video selected; one observation per scene/profile/revision',
				samples
			},
			null,
			2
		)
	);
	console.log('PASS scene performance: ' + samples.length + ' observations');
} finally {
	await browser.close();
}
