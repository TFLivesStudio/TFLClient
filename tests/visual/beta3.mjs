// Compiled production UI with deterministic IPC. This is not native WebView evidence.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({ headless: true });
const url = process.env.VISUAL_URL || 'http://127.0.0.1:1431';
const output = 'artifacts/beta3';
await mkdir(output, { recursive: true });
const results = [],
	errors = [];
async function open(style, theme, viewport = { width: 1100, height: 700 }, dpr = 1, extra = {}) {
	const context = await browser.newContext({ viewport, deviceScaleFactor: dpr });
	const page = await context.newPage();
	page.setDefaultTimeout(30000);
	page.on('pageerror', (error) => errors.push(error.message));
	await page.addInitScript(installFixture, { theme, ...extra });
	await page.addInitScript(
		({ style }) => {
			localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
			localStorage.setItem(
				'tfl-scenes-v1',
				JSON.stringify({
					version: 2,
					global: {
						preset: 'aurora-valley',
						motion: 'off',
						adaptTime: false
					},
					instances: {}
				})
			);
		},
		{ style }
	);
	await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 60000 });
	await page
		.locator(extra.empty ? '.empty-state' : '.hero')
		.first()
		.waitFor();
	await page.locator('.pixel-landscape img').waitFor();
	await page.waitForFunction(() =>
		[...document.querySelectorAll('.pixel-landscape img')].every(
			(i) => i.complete && i.naturalWidth
		)
	);
	return { page, context };
}
async function shot(page, name) {
	assert.equal(
		await page.evaluate(() => document.documentElement.scrollWidth > innerWidth),
		false,
		`${name}: document overflow`
	);
	await page.screenshot({ path: `${output}/${name}.png` });
	results.push(name);
}
async function readingContrast(page, selector) {
	return page
		.locator(selector)
		.first()
		.evaluate((element) => {
			function lum(color) {
				return color
					.map((c) => {
						const v = c / 255;
						return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
					})
					.reduce((s, v, i) => s + v * [0.2126, 0.7152, 0.0722][i], 0);
			}
			const sample = document.createElement('canvas').getContext('2d');
			const resolve = (color) => {
				sample.clearRect(0, 0, 1, 1);
				sample.fillStyle = color;
				sample.fillRect(0, 0, 1, 1);
				return [...sample.getImageData(0, 0, 1, 1).data];
			};
			// A panel's inner layout can be transparent; its Surface ancestor owns
			// the opaque reading background. Compose the actual ancestor colors.
			const layers = [];
			for (let node = element; node; node = node.parentElement) {
				const color = resolve(getComputedStyle(node).backgroundColor);
				layers.unshift(color);
				if (color[3] === 255) break;
			}
			if (!layers.length || layers[0][3] !== 255) throw new Error('No stable reading surface');
			let background = [0, 0, 0];
			for (const color of layers)
				background = background.map(
					(c, i) => (color[i] * color[3]) / 255 + c * (1 - color[3] / 255)
				);
			return ['--text-primary', '--text-secondary', '--text-muted'].map((token) => {
				// Resolve custom-property color-mix through the computed foreground of a real node.
				const probe = document.createElement('span');
				probe.style.color = `var(${token})`;
				element.append(probe);
				const foreground = resolve(getComputedStyle(probe).color).slice(0, 3);
				probe.remove();
				const a = lum(foreground),
					b = lum(background);
				return { token, ratio: (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05) };
			});
		});
}
try {
	for (const style of ['pixel', 'modern'])
		for (const theme of ['light', 'dark']) {
			const { page, context } = await open(style, theme);
			await shot(page, `home-${style}-${theme}`);
			for (const selector of ['.sidebar', '.home-greeting', '.panel']) {
				for (const sample of await readingContrast(page, selector))
					assert.ok(
						sample.ratio >= 4.5,
						`${style}/${theme} ${selector} ${sample.token} ${sample.ratio}`
					);
			}
			await page
				.locator('.sidebar')
				.getByRole('button', { name: 'Biblioteca', exact: true })
				.click();
			await page.locator('.library-view').waitFor();
			await shot(page, `library-${style}-${theme}`);
			await page.locator('.library-search input').fill('no-such-world');
			await page.locator('.library-empty').waitFor();
			await shot(page, `empty-${style}-${theme}`);
			await page.locator('.sidebar').getByRole('button', { name: 'Explorar', exact: true }).click();
			await page.locator('.discover-view').waitFor();
			await shot(page, `discover-${style}-${theme}`);
			await page
				.locator('.sidebar')
				.getByRole('button', { name: 'Horizontes', exact: true })
				.click();
			await page.locator('.instance-detail').waitFor();
			await shot(page, `detail-${style}-${theme}`);
			for (const sample of await readingContrast(page, '.detail-workspace'))
				assert.ok(sample.ratio >= 4.5, `${style}/${theme} detail ${sample.token} ${sample.ratio}`);
			await page.getByRole('button', { name: 'Ajustes', exact: true }).click();
			await page
				.locator('.panel-tabs')
				.getByRole('button', { name: 'Fondos y escenas', exact: true })
				.click();
			await page.locator('.scene-settings').waitFor();
			await shot(page, `settings-${style}-${theme}`);
			await page.keyboard.press('Escape');
			await context.close();
		}
	for (const viewport of [
		{ width: 800, height: 600 },
		{ width: 1600, height: 1000 }
	]) {
		const { page, context } = await open('pixel', 'light', viewport);
		await shot(page, `size-${viewport.width}x${viewport.height}`);
		await context.close();
	}
	for (const dpr of [1.25, 1.5, 2]) {
		const { page, context } = await open('pixel', 'dark', undefined, dpr);
		const geometry = await page.locator('.pixel-landscape img').evaluate((i) => {
			const s = getComputedStyle(i);
			return {
				width: parseFloat(s.width),
				height: parseFloat(s.height),
				left: parseFloat(s.left),
				top: parseFloat(s.top),
				rendering: s.imageRendering
			};
		});
		assert.equal(geometry.width % 384, 0);
		assert.equal(geometry.height % 216, 0);
		assert.ok(Number.isInteger(geometry.left) && Number.isInteger(geometry.top));
		assert.equal(geometry.rendering, 'pixelated');
		await shot(page, `dpr-${dpr}`);
		await context.close();
	}
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{
				browser: await browser.version(),
				scenario:
					'compiled UI, simulated IPC; DPR emulation is not Windows display scaling certification',
				screenshots: results,
				errors
			},
			null,
			2
		)
	);
	console.log(
		`PASS: ${results.length} UI cases, stable-surface contrast, responsive bounds and logical pixel scale`
	);
} finally {
	await browser.close();
}
