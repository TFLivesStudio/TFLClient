// Chromium observations only: production frontend with identical IPC fixtures, no native/game claims.
import { writeFile, mkdir } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {})
});
const output = process.env.PERFORMANCE_OUTPUT || 'artifacts/experience/performance';
await mkdir(output, { recursive: true });
const results = [];
try {
	for (const [revision, url] of [
		['baseline', process.env.BASELINE_URL || 'http://localhost:4173'],
		['refresh', process.env.VISUAL_URL || 'http://localhost:4174']
	])
		for (const quality of ['Lite', 'Balanced', 'Experience'])
			for (let run = 0; run < 3; run++) {
				const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
				await page.addInitScript(installFixture, {
					quality,
					appVersion:
						revision === 'baseline' ? process.env.BASELINE_VERSION || '0.13.1-beta.4' : undefined
				});
				const cdp = await page.context().newCDPSession(page);
				await cdp.send('Performance.enable');
				const start = performance.now();
				await page.goto(url);
				await page.getByRole('button', { name: 'Jugar', exact: true }).first().waitFor();
				const readyMs = performance.now() - start;
				await page.waitForTimeout(500);
				const first = Object.fromEntries(
					(await cdp.send('Performance.getMetrics')).metrics.map((x) => [x.name, x.value])
				);
				await page.waitForTimeout(1000);
				const last = Object.fromEntries(
					(await cdp.send('Performance.getMetrics')).metrics.map((x) => [x.name, x.value])
				);
				results.push({
					revision,
					quality,
					run,
					readyMs: Math.round(readyMs),
					heapMiB: +(last.JSHeapUsedSize / 1048576).toFixed(2),
					idleTaskMs: +((last.TaskDuration - first.TaskDuration) * 1000).toFixed(2),
					nodes: last.Nodes
				});
				if (revision === 'baseline' && quality === 'Balanced' && run === 0)
					await page.screenshot({ path: `${output}/baseline.png` });
				const settingsStart = performance.now();
				await page.getByRole('button', { name: 'Ajustes', exact: true }).click();
				await page.getByRole('dialog').waitFor();
				results.at(-1).settingsMs = Math.round(performance.now() - settingsStart);
				await page.keyboard.press('Escape');
				const detailStart = performance.now();
				await page.getByRole('button', { name: 'Horizontes', exact: true }).click();
				await page.getByRole('button', { name: 'Detalles', exact: true }).waitFor();
				results.at(-1).detailMs = Math.round(performance.now() - detailStart);
				await page.close();
			}
	await writeFile(
		`${output}/performance.json`,
		JSON.stringify({ fixture: true, browser: await browser.version(), samples: results }, null, 2)
	);
	console.log(JSON.stringify(results));
} finally {
	await browser.close();
}
