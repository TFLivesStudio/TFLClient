import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
const output = process.env.VISUAL_OUTPUT || 'artifacts/visual-refresh';
await mkdir(output, { recursive: true });
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}),
	args: ['--enable-unsafe-swiftshader']
});
const page = await browser.newPage();
page.setDefaultTimeout(10000);
const errors = [];
page.on('pageerror', (e) => errors.push(e.message));
try {
	await page.addInitScript(installFixture, { premium: true, skin: true, quality: 'Lite' });
	await page.addInitScript(() => {
		window.__webglCreated = 0;
		const original = HTMLCanvasElement.prototype.getContext;
		HTMLCanvasElement.prototype.getContext = function (type, ...args) {
			if (type.startsWith('webgl')) window.__webglCreated++;
			return original.call(this, type, ...args);
		};
	});
	await page.goto(process.env.VISUAL_URL || 'http://localhost:1422');
	await page.locator('.pixel-person').waitFor();
	assert.equal(
		await page.evaluate(() => window.__webglCreated),
		0,
		'Lite must not initialize WebGL'
	);
	assert.equal(
		await page.evaluate(
			() =>
				performance.getEntriesByType('resource').filter((e) => e.name.includes('skinview3d')).length
		),
		0,
		'Lite must not import skinview3d'
	);
	await page.screenshot({ path: `${output}/skin-lite-static.png` });
	await page.getByRole('button', { name: 'Activar vista 3D', exact: true }).click();
	await page.waitForFunction(() => window.__webglCreated > 0);
	await page.waitForTimeout(800);
	assert.equal(await page.locator('.hero canvas').count(), 1);
	await page.getByRole('button', { name: 'Skin y capa', exact: true }).click();
	await page.getByRole('dialog').waitFor();
	await page.waitForTimeout(200);
	assert.equal(await page.locator('.hero canvas').count(), 1); // DOM retained, renderer disposed while covered.
	await page.screenshot({ path: `${output}/skin-studio-lite.png` });
	await page.keyboard.press('Escape');
	await page.waitForTimeout(300);
	assert.ok(
		await page.evaluate(() => window.__webglCreated >= 2),
		'hero renderer recreated after overlay closes'
	);
	await page.getByRole('button', { name: 'Usar vista estática', exact: true }).click();
	await page.locator('.pixel-person').waitFor();
	assert.equal(await page.locator('.hero canvas').count(), 0);
	assert.deepEqual(errors, []);
	console.log(
		'PASS: Lite static skin, lazy import, manual WebGL, overlay disposal/recreation, manual teardown. Synthetic test texture.'
	);
} finally {
	await browser.close();
}
