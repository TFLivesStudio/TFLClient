// Controlled Chromium observations. These are not native CPU/GPU/RSS benchmarks.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({ headless: true, args: ['--enable-unsafe-swiftshader'] });
const output = process.env.PERFORMANCE_OUTPUT || 'artifacts/refinement-performance';
await mkdir(output, { recursive: true });
const samples = [];
try {
	for (const [revision, url, appVersion] of [
		['beta1', process.env.BASELINE_URL, process.env.BASELINE_VERSION || '0.15.4-beta.1'],
		['beta2', process.env.VISUAL_URL, process.env.APP_VERSION || '0.15.4-beta.2']
	])
		for (const quality of ['Lite', 'Balanced', 'Experience'])
			for (const cape of [false, true]) {
				const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
				await page.addInitScript(installFixture, {
					premium: true,
					skin: true,
					quality,
					appVersion
				});
				await page.addInitScript((cape) => {
					const texture = document.createElement('canvas');
					texture.width = 64;
					texture.height = 32;
					const ctx = texture.getContext('2d');
					ctx.fillStyle = '#347293';
					ctx.fillRect(0, 0, 64, 32);
					const url = texture.toDataURL(),
						invoke = window.__TAURI_INTERNALS__.invoke;
					window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
						const value = await invoke(cmd, args);
						if (cmd === 'get_mojang_profile' && cape)
							value.capes = [{ id: 'fixture-cape', state: 'ACTIVE', url }];
						return value;
					};
					localStorage.setItem(
						'tfl-scenes-v1',
						JSON.stringify({
							version: 1,
							global: {
								preset: 'midnight-forest',
								scope: 'window',
								fps: 24,
								motion: 'subtle',
								shader: false
							},
							instances: {}
						})
					);
					window.__probe = { draws: 0, contexts: 0 };
					const get = HTMLCanvasElement.prototype.getContext,
						seen = new WeakSet();
					HTMLCanvasElement.prototype.getContext = function (type, ...args) {
						const value = get.call(this, type, ...args);
						if (value && type.startsWith('webgl') && !seen.has(value)) {
							seen.add(value);
							window.__probe.contexts++;
							for (const method of ['drawArrays', 'drawElements']) {
								const draw = value[method].bind(value);
								value[method] = (...args) => {
									window.__probe.draws++;
									return draw(...args);
								};
							}
						}
						return value;
					};
				}, cape);
				const cdp = await page.context().newCDPSession(page);
				await cdp.send('Performance.enable');
				const started = performance.now();
				await page.goto(url);
				await page.locator('.hero-name').waitFor();
				if (quality !== 'Lite') await page.waitForFunction(() => window.__probe.draws > 0);
				const readyMs = performance.now() - started;
				await page.waitForTimeout(800);
				const metrics = async () =>
					Object.fromEntries(
						(await cdp.send('Performance.getMetrics')).metrics.map((m) => [m.name, m.value])
					);
				const intervals = [];
				for (let n = 0; n < 2; n++) {
					const a = await metrics(),
						draws = await page.evaluate(() => window.__probe.draws);
					await page.waitForTimeout(1000);
					const b = await metrics();
					intervals.push({
						taskMs: (b.TaskDuration - a.TaskDuration) * 1000,
						jsHeapBytes: b.JSHeapUsedSize,
						drawCalls: (await page.evaluate(() => window.__probe.draws)) - draws
					});
				}
				const contexts = await page.evaluate(() => window.__probe.contexts);
				if (quality === 'Lite') {
					assert.equal(contexts, 0);
					assert.ok(intervals.every((x) => x.drawCalls === 0));
				}
				samples.push({ revision, quality, cape, readyMs, contexts, intervals });
				await page.close();
			}
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{
				browser: await browser.version(),
				environment:
					'Windows Chromium, SwiftShader, synthetic IPC/skin/cape; two one-second observations after warmup, no native or GPU certification',
				samples
			},
			null,
			2
		)
	);
	console.log(`PASS refinement performance: ${samples.length} controlled configurations`);
} finally {
	await browser.close();
}
