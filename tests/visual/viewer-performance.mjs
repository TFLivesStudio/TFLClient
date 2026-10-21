// Software-rendered browser probe, not a native GPU or whole-process CPU benchmark.
import { writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}),
	args: ['--enable-unsafe-swiftshader']
});
const samples = [];
try {
	for (const [revision, url] of [
		['baseline', process.env.BASELINE_URL || 'http://localhost:4173'],
		['refresh', process.env.VISUAL_URL || 'http://localhost:4174']
	])
		for (const quality of ['Lite', 'Balanced', 'Experience']) {
			const p = await browser.newPage({ viewport: { width: 1280, height: 720 } });
			p.setDefaultTimeout(10000);
			await p.addInitScript(installFixture, { quality, premium: true, skin: true });
			await p.addInitScript(() => {
				window.__contexts = 0;
				window.__frames = 0;
				const get = HTMLCanvasElement.prototype.getContext;
				HTMLCanvasElement.prototype.getContext = function (type, ...args) {
					if (type.startsWith('webgl')) window.__contexts++;
					return get.call(this, type, ...args);
				};
				const raf = requestAnimationFrame;
				window.requestAnimationFrame = (cb) =>
					raf((t) => {
						window.__frames++;
						cb(t);
					});
			});
			await p.goto(url);
			await p.getByRole('button', { name: 'Jugar', exact: true }).first().waitFor();
			await p.waitForTimeout(1500);
			const cdp = await p.context().newCDPSession(p);
			await cdp.send('Performance.enable');
			const before = Object.fromEntries(
				(await cdp.send('Performance.getMetrics')).metrics.map((x) => [x.name, x.value])
			);
			const frames = await p.evaluate(() => window.__frames);
			await p.waitForTimeout(1000);
			const after = Object.fromEntries(
				(await cdp.send('Performance.getMetrics')).metrics.map((x) => [x.name, x.value])
			);
			samples.push({
				revision,
				quality,
				...(await p.evaluate(() => ({
					contexts: window.__contexts,
					rafCallbacks: window.__frames
				}))),
				callbacksInSample: (await p.evaluate(() => window.__frames)) - frames,
				heapMiB: +(after.JSHeapUsedSize / 1048576).toFixed(2),
				taskMs: +((after.TaskDuration - before.TaskDuration) * 1000).toFixed(2),
				elapsedMs: +((after.Timestamp - before.Timestamp) * 1000).toFixed(2)
			});
			await p.close();
		}
	await writeFile(
		'artifacts/visual-refresh/viewer-performance.json',
		JSON.stringify(
			{
				fixture: true,
				syntheticTexture: true,
				softwareRenderer: true,
				browser: await browser.version(),
				samples
			},
			null,
			2
		)
	);
	console.log(samples);
} finally {
	await browser.close();
}
