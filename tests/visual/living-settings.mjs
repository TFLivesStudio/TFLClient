// Actual compiled UI, browser-only IPC fixture; no native launch/account certification.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const output = process.env.LIVING_SETTINGS_OUTPUT || 'artifacts/living-settings';
await mkdir(output, { recursive: true });
const errors = [];
const checks = [];
async function open({
	scope = 'hero',
	style = 'pixel',
	quality = 'Balanced',
	instances = {}
} = {}) {
	const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
	const page = await context.newPage();
	page.on('pageerror', (error) => errors.push(error.message));
	await page.addInitScript(installFixture, { quality });
	await page.addInitScript(
		({ scope, style, instances }) => {
			localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
			localStorage.setItem(
				'tfl-scenes-v1',
				JSON.stringify({ version: 1, global: { preset: 'midnight-forest', scope }, instances })
			);
		},
		{ scope, style, instances }
	);
	await page.goto(process.env.VISUAL_URL || 'http://localhost:1433');
	await page.locator('.hero').waitFor();
	return page;
}
async function settings(page) {
	await page.getByRole('button', { name: 'Ajustes', exact: true }).click();
	await page
		.locator('.panel-tabs')
		.getByRole('button', { name: 'Fondos y escenas', exact: true })
		.click();
}
async function placement(page, expected) {
	await page.waitForFunction(
		(scope) => document.querySelectorAll(`[data-scene-placement="${scope}"]`).length === 1,
		expected
	);
	assert.equal(
		await page.locator('[data-scene-placement]').count(),
		1,
		'one active rendering owner'
	);
	assert.equal(await page.locator(`[data-scene-placement="${expected}"]`).count(), 1);
}
try {
	for (const style of ['pixel', 'modern']) {
		const page = await open({ scope: 'window', style });
		await placement(page, 'window');
		await page.screenshot({ path: `${output}/${style}-window.png` });
		await settings(page);
		const stored = await page.evaluate(() => localStorage.getItem('tfl-scenes-v1'));
		await page.getByRole('button', { name: /Dithered Sunset/ }).click();
		assert.equal(
			await page.evaluate(() => localStorage.getItem('tfl-scenes-v1')),
			stored,
			'preview is not persisted'
		);
		await page
			.locator('.pending-scene')
			.getByRole('button', { name: 'Cancelar', exact: true })
			.click();
		assert.equal(
			await page.getByLabel('Mostrar escenario en').count(),
			0,
			'ambience is always global'
		);
		await page.getByRole('button', { name: /Aurora Valley/ }).click();
		await page
			.locator('.pending-scene')
			.getByRole('button', { name: 'Aplicar escena', exact: true })
			.click();
		await page.keyboard.press('Escape');
		await placement(page, 'window');
		assert.equal(await page.locator('.hero [data-landscape]').count(), 0);
		assert.equal(
			await page
				.locator('[data-scene-placement="window"] [data-landscape="aurora-valley"]')
				.count(),
			1
		);
		await page.screenshot({ path: `${output}/${style}-hero.png` });
		checks.push(`${style}: global background, preview cancel, apply, separate editorial Hero`);
		await page.context().close();
	}
	const instance = await open({
		instances: { horizons: { preset: 'dithered-sunset', scope: 'window', motion: 'off' } }
	});
	await placement(instance, 'window');
	assert.equal(
		await instance
			.locator('[data-scene-placement="window"] [data-landscape="midnight-forest"]')
			.count(),
		1
	);
	await settings(instance);
	await instance.getByLabel('Recuperar ambientación anterior').selectOption('horizons');
	await instance.getByRole('button', { name: 'Cargar ambientación', exact: true }).click();
	await instance
		.locator('.pending-scene')
		.getByRole('button', { name: 'Aplicar escena', exact: true })
		.click();
	await instance.keyboard.press('Escape');
	await instance
		.locator('[data-scene-placement="window"] [data-landscape="dithered-sunset"]')
		.waitFor();
	assert.equal(
		await instance.locator('[data-scene-placement] canvas').count(),
		0,
		'restored archived static scene remains static'
	);
	checks.push('instance archive is retained and explicitly restored as a global static background');
	await instance.context().close();
	const lite = await open({ scope: 'window', quality: 'Lite' });
	await placement(lite, 'window');
	assert.equal(
		await lite.locator('[data-scene-placement] canvas,[data-scene-placement] video').count(),
		0
	);
	checks.push('Lite window background has no canvas or video');
	await lite.context().close();
	assert.deepEqual(errors, []);
	await writeFile(`${output}/results.json`, JSON.stringify({ checks, errors }, null, 2));
	console.log(JSON.stringify({ checks, errors }, null, 2));
} finally {
	await browser.close();
}
