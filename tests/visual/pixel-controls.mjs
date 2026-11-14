// Actual Svelte frontend with isolated IPC fixtures; this is not native OS certification.
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const output = process.env.PIXEL_OUTPUT || 'artifacts/pixel-controls';
const appVersion = JSON.parse(
	await readFile(new URL('../../package.json', import.meta.url), 'utf8')
).version;
await mkdir(output, { recursive: true });
const errors = [],
	results = [];
async function open(options) {
	const page = await browser.newPage({
		viewport: { width: 1280, height: 720 },
		deviceScaleFactor: options.dpr || 1
	});
	page.setDefaultTimeout(12000);
	page.on('pageerror', (error) => errors.push(error.message));
	await page.addInitScript(installFixture, {
		premium: true,
		appVersion,
		quality: options.quality || 'Balanced',
		theme: options.theme === 'light' ? 'light' : 'dark',
		language: options.language || 'es'
	});
	await page.addInitScript((o) => {
		localStorage.setItem(
			'tfl-visual-refresh-v1',
			JSON.stringify({
				style: o.style,
				pixelType: o.font || 'readable',
				material: o.material || 'matte',
				customAccent: o.accent || ''
			})
		);
		localStorage.setItem('tfl-theme', o.theme === 'light' ? 'light' : 'dark');
		localStorage.setItem('tfl-surface', o.theme === 'oled' ? 'oled' : 'obsidian');
	}, options);
	await page.goto(process.env.VISUAL_URL || 'http://localhost:4174');
	await page.locator('.hero').waitFor();
	return page;
}
async function search(page, selector, label, pixel) {
	const wrapper = page.locator(selector).first(),
		input = wrapper.locator('input');
	await input.waitFor({ state: 'visible' });
	await input.focus();
	const metrics = await wrapper.evaluate((node) => {
		const input = node.querySelector('input'),
			frame = getComputedStyle(node),
			text = getComputedStyle(input);
		const r = node.getBoundingClientRect(),
			i = input.getBoundingClientRect();
		const canvas = document.createElement('canvas');
		canvas.width = canvas.height = 1;
		const context = canvas.getContext('2d');
		context.fillStyle = text.backgroundColor;
		context.fillRect(0, 0, 1, 1);
		return {
			frame: frame.borderWidth,
			border: text.borderWidth,
			shadow: text.boxShadow,
			background: text.backgroundColor,
			backgroundAlpha: context.getImageData(0, 0, 1, 1).data[3],
			focus: frame.outlineStyle,
			focusWidth: frame.outlineWidth,
			color: text.color,
			placeholder: getComputedStyle(input, '::placeholder').color,
			width: i.width,
			height: i.height,
			outerHeight: r.height,
			rect: { x: r.x, y: r.y, width: r.width, height: r.height },
			inputRect: { x: i.x, y: i.y, width: i.width, height: i.height },
			fits:
				i.left >= r.left && i.right <= r.right + 1 && i.top >= r.top && i.bottom <= r.bottom + 1,
			name: input.getAttribute('aria-label') || input.labels?.[0]?.textContent,
			font: text.fontFamily
		};
	});
	assert.ok(
		metrics.width > 20 && metrics.height >= (pixel ? 20 : 16),
		`${label}: usable input dimensions`
	);
	if (!metrics.fits) {
		console.log(label, metrics);
		await page.screenshot({ path: `${output}/failure.png` });
	}
	assert.ok(metrics.fits, `${label}: text fits within the control`);
	assert.ok(metrics.name?.trim(), `${label}: explicit accessible name`);
	if (pixel) {
		assert.equal(metrics.frame, '2px', `${label}: one outer frame`);
		assert.equal(metrics.border, '0px', `${label}: no inner frame`);
		assert.equal(metrics.shadow, 'none', `${label}: no inner bevel`);
		assert.equal(metrics.backgroundAlpha, 0, `${label}: transparent inner field`);
		assert.equal(metrics.focus, 'solid', `${label}: visible focus`);
		assert.equal(metrics.focusWidth, '2px', `${label}: focus width`);
	}
	results.push({ label, pixel, ...metrics });
}
async function shot(page, name) {
	await page.waitForTimeout(500);
	await page.screenshot({ path: `${output}/${name}.png` });
}
try {
	for (const style of ['modern', 'pixel'])
		for (const font of ['readable', 'classic']) {
			const p = await open({ style, font });
			await shot(p, `${style}-${font}-home`);
			await search(p, '.search-box', `${style}-${font}-sidebar`, style === 'pixel');
			await p.getByRole('button', { name: 'Biblioteca', exact: true }).first().click();
			await search(p, '.library-search', `${style}-${font}-library`, style === 'pixel');
			await p.locator('.library-search input').fill('Horizontes');
			assert.equal(
				await p.locator('.vr-library-row').count(),
				1,
				'Library still filters instances'
			);
			await p.locator('.library-search input').fill('');
			await shot(p, `${style}-${font}-library`);
			await p.keyboard.press('Control+k');
			await search(p, '.input-row', `${style}-${font}-palette`, style === 'pixel');
			await p.locator('.input-row input').fill('Horizontes');
			assert.ok((await p.locator('.palette .item').count()) > 0, 'Command search still filters');
			await shot(p, `${style}-${font}-palette`);
			await p.keyboard.press('Escape');
			assert.equal(await p.locator('.palette').count(), 0, 'Escape closes the palette');
			await p.getByRole('button', { name: 'Ajustes', exact: true }).click();
			await search(p, '.settings-search', `${style}-${font}-settings`, style === 'pixel');
			await p.getByRole('button', { name: 'Apariencia', exact: true }).click();
			await shot(p, `${style}-${font}-appearance`);
			await p.keyboard.press('Escape');
			await p.locator('.instance-open').filter({ hasText: 'Horizontes' }).click();
			await p
				.getByRole('button', { name: 'Cambiar \u00edcono de la instancia', exact: true })
				.click();
			await search(p, '.icon-search', `${style}-${font}-icons`, style === 'pixel');
			await shot(p, `${style}-${font}-icons`);
			await p.keyboard.press('Escape');
			await p.locator('.tabs').getByRole('button', { name: 'Mods', exact: true }).click();
			await p.getByRole('button', { name: 'Descargar', exact: true }).click();
			await search(p, '.search-row', `${style}-${font}-mods`, style === 'pixel');
			await shot(p, `${style}-${font}-mods`);
			await p.close();
		}
	// Both typefaces, themes, quality tiers, materials and DPRs; required sizes on each combination.
	for (const font of ['readable', 'classic'])
		for (const theme of ['dark', 'light', 'oled'])
			for (const quality of ['Lite', 'Balanced', 'Experience']) {
				const p = await open({
					style: 'pixel',
					font,
					theme,
					quality,
					dpr: font === 'classic' ? 2 : 1,
					material: quality === 'Lite' ? 'matte' : quality === 'Balanced' ? 'soft-glass' : 'glass',
					accent: theme === 'light' ? '#5432b6' : '#9de9c6'
				});
				const primary = await p
					.locator('.play-btn,.vr-control.primary,.tfl-btn.primary')
					.first()
					.evaluate((button) => {
						const root = getComputedStyle(document.documentElement),
							style = getComputedStyle(button);
						const canvas = document.createElement('canvas');
						canvas.width = canvas.height = 1;
						const context = canvas.getContext('2d');
						const color = (value) => {
							context.clearRect(0, 0, 1, 1);
							context.fillStyle = value;
							context.fillRect(0, 0, 1, 1);
							return [...context.getImageData(0, 0, 1, 1).data];
						};
						return {
							background: color(style.backgroundColor),
							accent: color(root.getPropertyValue('--accent')),
							text: color(style.color),
							foreground: color(root.getPropertyValue('--accent-text'))
						};
					});
				assert.deepEqual(primary.background, primary.accent, 'Pixel action follows custom accent');
				assert.deepEqual(
					primary.text,
					primary.foreground,
					'Pixel action preserves contrast-aware text'
				);
				await p.getByRole('button', { name: 'Ajustes', exact: true }).click();
				await p.getByRole('button', { name: 'Apariencia', exact: true }).click();
				for (const [width, height] of [
					[800, 600],
					[1100, 700],
					[1280, 720],
					[1366, 768],
					[1920, 1080]
				]) {
					await p.setViewportSize({ width, height });
					await search(p, '.settings-search', `${font}-${theme}-${quality}-${width}`, true);
					assert.ok(await p.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
					if (width === 800 && font === 'classic') {
						for (const name of ['Integraciones', 'Actualizaciones']) {
							const label = p.locator('.panel-tabs .ptab').filter({ hasText: name });
							const fit = await label.evaluate((button) => {
								const walker = document.createTreeWalker(button, NodeFilter.SHOW_TEXT);
								let text;
								while ((text = walker.nextNode()))
									if (text.textContent.trim()) {
										const range = document.createRange();
										range.selectNodeContents(text);
										return (
											range.getClientRects().length === 1 &&
											button.scrollWidth <= button.clientWidth
										);
									}
								return false;
							});
							assert.ok(fit, `${name}: whole translated label fits Classic navigation at 800px`);
						}
					}
					if (width === 800 || width === 1280)
						await shot(p, `${font}-${theme}-${quality}-${width}`);
				}
				await p.close();
			}
	const accessible = await open({ style: 'pixel', font: 'classic', language: 'en' });
	await accessible.getByRole('button', { name: 'Settings', exact: true }).click();
	await search(accessible, '.settings-search', 'english-classic-keyboard', true);
	await accessible.emulateMedia({ forcedColors: 'active', reducedMotion: 'reduce' });
	await search(accessible, '.settings-search', 'forced-colors-reduced-motion', true);
	await shot(accessible, 'english-forced-colors');
	await accessible.close();
	assert.deepEqual(errors, []);
	await writeFile(`${output}/results.json`, JSON.stringify({ results, errors }, null, 2));
	console.log(
		`PASS: ${results.length} computed-style, focus, accessible-name and geometry checks; library/palette behavior; Pixel matrix.`
	);
} finally {
	await browser.close();
}
