// Real frontend, simulated IPC. Run against Vite; never claims native/game verification.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {})
});
const output = process.env.VISUAL_OUTPUT || 'artifacts/visual-refresh';
await mkdir(output, { recursive: true });
const errors = [],
	results = [];
async function open(options = {}) {
	const page = await browser.newPage({
		viewport: options.viewport || { width: 1280, height: 720 },
		reducedMotion: options.reduced ? 'reduce' : 'no-preference',
		deviceScaleFactor: options.dpr || 1
	});
	page.on('pageerror', (e) => errors.push(e.message));
	await page.addInitScript(installFixture, options);
	await page.addInitScript(
		({ style, theme, surface }) => {
			if (!localStorage.getItem('tfl-visual-refresh-v1'))
				localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
			localStorage.setItem('tfl-surface', surface);
			localStorage.setItem('tfl-theme', theme);
		},
		{
			style: options.style || 'modern',
			theme: options.theme || 'dark',
			surface: options.surface || 'obsidian'
		}
	);
	await page.goto(process.env.VISUAL_URL || 'http://localhost:1422');
	await page.locator('.hero,.empty-state').first().waitFor();
	await page.waitForTimeout(500);
	return page;
}
async function shot(p, name) {
	await p.waitForTimeout(180);
	await p.screenshot({ path: `${output}/${name}.png` });
	results.push(name);
}
async function noOverflow(p) {
	assert.ok(
		await p.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
		'document must not overflow horizontally'
	);
}
try {
	for (const style of ['modern', 'pixel'])
		for (const quality of ['Lite', 'Balanced', 'Experience']) {
			const p = await open({ style, quality });
			await noOverflow(p);
			await shot(p, `home-${style}-${quality.toLowerCase()}`);
			if (quality === 'Lite') assert.equal(await p.locator('canvas').count(), 0);
			await p.close();
		}
	for (const [width, height] of [
		[800, 600],
		[1100, 700],
		[1366, 768],
		[1920, 1080]
	]) {
		const p = await open({ viewport: { width, height }, dpr: width === 1366 ? 2 : 1 });
		await noOverflow(p);
		await shot(p, `home-${width}x${height}`);
		await p.getByRole('button', { name: 'Abrir Horizontes', exact: true }).click();
		await noOverflow(p);
		await shot(p, `detail-${width}x${height}`);
		await p.close();
	}
	const p = await open();
	await p.getByRole('button', { name: 'Biblioteca', exact: true }).first().click();
	assert.equal(await p.locator('.vr-library-row').count(), 4);
	await shot(p, 'library');
	await p.getByRole('button', { name: 'Favorita: Horizontes', exact: true }).click();
	await p.reload();
	await p.getByRole('button', { name: 'Biblioteca', exact: true }).first().click();
	assert.equal(
		await p
			.getByRole('button', { name: 'Favorita: Horizontes', exact: true })
			.getAttribute('aria-pressed'),
		'true'
	);
	await p.getByRole('button', { name: 'Favoritas', exact: true }).click();
	assert.equal(await p.locator('.vr-library-row').count(), 1);
	await p.getByRole('button', { name: 'Todas', exact: true }).click();
	await p.getByRole('button', { name: 'Cuadrícula', exact: true }).click();
	await shot(p, 'library-grid');
	await p.getByRole('button', { name: 'Lista', exact: true }).click();
	await p.getByRole('button', { name: 'Más acciones: Horizontes', exact: true }).click();
	assert.ok(await p.getByRole('menu').isVisible());
	await p.keyboard.press('Escape');
	await p.getByRole('button', { name: 'Explorar', exact: true }).click();
	await shot(p, 'discover');
	await p.getByRole('button', { name: 'TFL Friends', exact: true }).click();
	await shot(p, 'server-dashboard');
	await p.getByRole('button', { name: 'Ajustes', exact: true }).click();
	await p.getByRole('button', { name: 'Apariencia', exact: true }).click();
	await shot(p, 'settings');
	for (let i = 0; i < 50; i++) await p.keyboard.press('Tab');
	assert.ok(
		await p.evaluate(() => !!document.activeElement.closest('[role=dialog]')),
		'modal contains keyboard focus'
	);
	await p.getByRole('button', { name: /^Pixel/ }).click();
	assert.equal(await p.locator('html').getAttribute('data-visual'), 'pixel');
	await shot(p, 'settings-pixel');
	await p.keyboard.press('Escape');
	assert.equal(await p.getByRole('dialog').count(), 0);
	await p.evaluate(() =>
		window.__fixtureEmit('DownloadFailed', { task: 'Fixture download', error: 'Offline fixture' })
	);
	await p.getByRole('button', { name: 'Notificaciones', exact: true }).click();
	assert.ok((await p.getByRole('dialog').innerText()).includes('Offline fixture'));
	await shot(p, 'notification-error');
	await p.keyboard.press('Escape');
	await p.getByRole('button', { name: 'Crear instancia', exact: true }).first().click();
	await shot(p, 'create-dialog');
	await p.keyboard.press('Escape');
	await p.getByRole('button', { name: 'Buscar instancias', exact: true }).click();
	await shot(p, 'command-palette');
	await p.keyboard.press('Escape');
	await p.close();
	const studio = await open({ premium: true });
	await studio.getByRole('button', { name: 'Skin y capa', exact: true }).click();
	await shot(studio, 'skin-studio-no-skin');
	await studio.keyboard.press('Escape');
	await studio.close();
	const light = await open({ theme: 'light', language: 'en' });
	assert.ok((await light.locator('body').innerText()).includes('Library'));
	await shot(light, 'home-light-en');
	await light.close();
	const oled = await open({
		surface: 'oled',
		style: 'pixel',
		quality: 'Experience',
		reduced: true
	});
	assert.equal(await oled.locator('html').getAttribute('data-system-reduce-motion'), '');
	await shot(oled, 'home-oled-reduced');
	await oled.close();
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/smoke-results.json`,
		JSON.stringify({ fixture: true, results, errors }, null, 2)
	);
	console.log(
		`PASS: ${results.length} screenshots, navigation, context menu, dialog focus, notifications, responsive layouts; no page errors.`
	);
} finally {
	await browser.close();
}
