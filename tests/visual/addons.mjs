// Real production components with deterministic IPC. This is browser QA, not native installation certification.
import assert from 'node:assert/strict';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { contrastRatio } from '../../src/lib/addons/model.ts';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const output = process.env.ADDONS_OUTPUT || 'artifacts/beta3-addons';
await mkdir(output, { recursive: true });
const errors = [],
	results = [];
const fullThemeTokens = process.env.ADDONS_UI_FULL_THEME
	? JSON.parse(
			await readFile(
				new URL('../../examples/addons/orange-dunes/manifest.json', import.meta.url),
				'utf8'
			)
		).themes[0].tokens
	: null;
function addonsFixture(fullTokens) {
	const base = window.__TAURI_INTERNALS__.invoke;
	const manifest = {
		schemaVersion: 1,
		id: 'test.dunes',
		name: 'Dunas de prueba',
		author: 'Autor declarado',
		version: '1.0.0',
		description: 'Tema parcial y paisaje declarativo de prueba.',
		apiVersion: '^1.0.0',
		launcherVersion: '>=0.15.4-beta.3',
		permissions: [],
		themes: [
			{
				id: 'orange',
				name: 'Acento de prueba',
				tokens: { '--accent': '#ff7a2e', '--accent-text': '#101010' }
			}
		],
		scenes: [
			{
				id: 'dunes',
				name: 'Paisaje de prueba',
				background: '#142536',
				layers: [{ path: 'assets/dunes.png', speed: 3, opacity: 1 }]
			}
		]
	};
	const state = { safeMode: false, addons: [], errors: [], brokenAddons: [] };
	if (fullTokens) manifest.themes[0].tokens = fullTokens;
	if (fullTokens)
		manifest.themes.push({
			id: 'bad-muted',
			name: 'Metadatos sin contraste',
			tokens: { ...fullTokens, '--text-muted': fullTokens['--bg-card'] }
		});
	const digest = 'a'.repeat(64);
	let version = '1.0.0';
	let previous = null;
	window.__addonFixture = {
		state,
		corrupt() {
			state.brokenAddons = [
				{ id: 'test.dunes', error: 'Paquete de prueba dañado', rollbackAvailable: true }
			];
			state.addons = [];
		},
		setVersion(value) {
			version = value;
		},
		failNextInspect: false
	};
	const catalog = () =>
		structuredClone({
			...state,
			addons: state.addons.map((addon) => ({ ...addon, enabled: addon.enabled && !state.safeMode }))
		});
	window.__TAURI_INTERNALS__.invoke = async (cmd, args = {}) => {
		if (!cmd.startsWith('addons_')) return base(cmd, args);
		window.__fixtureCommands.push({ cmd, args });
		if (cmd === 'addons_list') return catalog();
		if (cmd === 'addons_pick_file') return '/fixture/dunes.tfl';
		if (cmd === 'addons_inspect' && window.__addonFixture.failNextInspect) {
			window.__addonFixture.failNextInspect = false;
			throw new Error('Archivo de prueba inválido');
		}
		if (cmd === 'addons_inspect')
			return {
				manifest: { ...manifest, version },
				digest,
				installedVersion: state.addons[0]?.manifest.version ?? null
			};
		if (cmd === 'addons_install') {
			const old = state.addons[0];
			previous = old ? structuredClone(old) : null;
			state.addons = [
				{
					manifest: { ...manifest, version },
					digest,
					enabled: old?.enabled ?? false,
					rollbackAvailable: !!old
				}
			];
			return catalog();
		}
		if (cmd === 'addons_set_enabled') {
			state.addons[0].enabled = args.enabled;
			return catalog();
		}
		if (cmd === 'addons_safe_mode') {
			state.safeMode = args.enabled;
			return catalog();
		}
		if (cmd === 'addons_remove') {
			state.addons = [];
			state.brokenAddons = [];
			return catalog();
		}
		if (cmd === 'addons_rollback') {
			state.brokenAddons = [];
			state.addons = [previous ?? { manifest, digest, enabled: false, rollbackAvailable: false }];
			return catalog();
		}
		if (cmd === 'addons_scene_resources')
			return {
				'assets/dunes.png':
					'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLbtAAAAABJRU5ErkJggg=='
			};
		throw new Error(`Unexpected addon command ${cmd}`);
	};
}
async function tab(page, name) {
	await page.locator('.panel-tabs').getByRole('button', { name, exact: true }).click();
}
try {
	for (const style of ['modern', 'pixel'])
		for (const theme of ['dark', 'light']) {
			if (
				process.env.ADDONS_UI_CASE &&
				!process.env.ADDONS_UI_CASE.split(',').includes(`${style}/${theme}`)
			)
				continue;
			console.log(`Addons UI ${style}/${theme}: opening production UI`);
			const context = await browser.newContext({ viewport: { width: 1100, height: 700 } });
			const page = await context.newPage();
			page.setDefaultTimeout(30000);
			page.on('pageerror', (e) => errors.push(e.message));
			await page.addInitScript(installFixture, { theme, appVersion: '0.15.4-beta.3' });
			await page.addInitScript(addonsFixture, fullThemeTokens);
			await page.addInitScript(
				(value) => localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style: value })),
				style
			);
			await page.goto(process.env.VISUAL_URL || 'http://localhost:1424', {
				waitUntil: 'domcontentloaded',
				timeout: 60000
			});
			await page.locator('.hero,.empty-state').first().waitFor();
			await page.getByRole('button', { name: 'Ajustes', exact: true }).click();
			await tab(page, 'Addons');
			await page.getByRole('button', { name: 'Seleccionar .tfl', exact: true }).click();
			const install = page.getByRole('button', { name: 'Instalar addon', exact: true });
			await install.waitFor();
			await page.locator('.inspection').first().scrollIntoViewIfNeeded();
			await page.screenshot({ path: `${output}/inspect-${style}-${theme}.png` });
			assert.equal(await install.isDisabled(), true, 'installation needs explicit consent');
			await page.locator('.consent input').check();
			await install.click();
			await page.locator('.installed-addon .enabled input').waitFor();
			await page.locator('.installed-addon').first().scrollIntoViewIfNeeded();
			await page.screenshot({ path: `${output}/installed-${style}-${theme}.png` });
			assert.equal(
				await page.locator('.installed-addon .enabled input').isChecked(),
				false,
				'installation starts disabled'
			);
			await page.locator('.installed-addon .enabled input').check();
			await tab(page, 'Apariencia');
			if (fullThemeTokens) {
				await page
					.getByRole('combobox', { name: 'Tema de addon', exact: true })
					.selectOption('addon:test.dunes:bad-muted');
				assert.equal(
					await page.evaluate(() => localStorage.getItem('tfl-addon-theme-v1')),
					'',
					'unreadable metadata theme rejected'
				);
				await page.getByRole('alert').filter({ hasText: '--text-muted' }).waitFor();
			}
			await page
				.getByRole('combobox', { name: 'Tema de addon', exact: true })
				.selectOption('addon:test.dunes:orange');
			assert.equal(
				await page.evaluate(() => localStorage.getItem('tfl-addon-theme-v1')),
				'addon:test.dunes:orange',
				'partial theme resolves real host color-mix defaults'
			);
			assert.equal(
				await page.evaluate(() => document.documentElement.style.getPropertyValue('--accent')),
				fullThemeTokens?.['--accent'] ?? '#ff7a2e'
			);
			await page.locator('.addon-theme-picker').scrollIntoViewIfNeeded();
			if (fullThemeTokens) {
				// Every visual save reapplies host defaults, including saves with no color-attribute change.
				for (const button of ['Modern', 'Pixel', 'Clásica', 'Legible']) {
					await page
						.getByRole('button', { name: button, exact: button !== 'Modern' && button !== 'Pixel' })
						.click();
					const selected = await page.evaluate(() => ({
						id: localStorage.getItem('tfl-addon-theme-v1'),
						accent: document.documentElement.style.getPropertyValue('--accent'),
						text: document.documentElement.style.getPropertyValue('--accent-text')
					}));
					assert.equal(selected.id, 'addon:test.dunes:orange');
					assert.equal(selected.accent, fullThemeTokens['--accent']);
					assert.equal(selected.text, fullThemeTokens['--accent-text']);
				}
				await page.getByRole('button', { name: 'Matte', exact: true }).click();
				assert.equal(await page.evaluate(() => document.documentElement.dataset.material), 'matte');
				assert.equal(
					await page.evaluate(() => localStorage.getItem('tfl-addon-theme-v1')),
					'addon:test.dunes:orange',
					'host material revalidation retains compatible addon'
				);
				await page.getByRole('button', { name: 'Soft Glass', exact: true }).click();
				assert.equal(
					await page.evaluate(() => document.documentElement.dataset.material),
					'soft-glass'
				);
				await page.locator('.addon-theme-picker').scrollIntoViewIfNeeded();
			}
			await page.screenshot({ path: `${output}/theme-${style}-${theme}.png` });
			if (fullThemeTokens) {
				const palette = await page.evaluate(() => {
					const css = getComputedStyle(document.documentElement);
					return {
						textAccent: css.getPropertyValue('--text-accent').trim(),
						surfaces: [
							'--bg-main',
							'--bg-sidebar',
							'--bg-card',
							'--bg-input',
							'--bg-item-active'
						].map((key) => css.getPropertyValue(key).trim())
					};
				});
				assert.ok(
					palette.surfaces.every((surface) => contrastRatio(palette.textAccent, surface) >= 4.5),
					'full addon theme keeps accent text readable on every surface'
				);
				const renderedContrast = await page.evaluate(() => {
					const canvas = document.createElement('canvas');
					canvas.width = canvas.height = 1;
					const context = canvas.getContext('2d');
					if (!context) throw new Error('Canvas required for composed contrast measurement');
					const rgba = () => {
						const [r, g, b, a] = context.getImageData(0, 0, 1, 1).data;
						return `rgba(${r},${g},${b},${a / 255})`;
					};
					return [
						'.settings-refresh .panel-header h2',
						'.settings-page-heading h3',
						'.settings-page-heading p',
						'.addon-theme-picker .field',
						'.addon-theme-picker small'
					].map((selector) => {
						const element = document.querySelector(selector);
						if (!element) throw new Error(`Missing real text surface ${selector}`);
						const chain = [];
						for (let node = element; node; node = node.parentElement) chain.push(node);
						const backgrounds = ['#000000', '#ffffff'].map((backdrop) => {
							context.fillStyle = backdrop;
							context.fillRect(0, 0, 1, 1);
							for (const node of chain.toReversed()) {
								context.fillStyle = getComputedStyle(node).backgroundColor;
								context.fillRect(0, 0, 1, 1);
							}
							return rgba();
						});
						context.clearRect(0, 0, 1, 1);
						context.fillStyle = getComputedStyle(element).color;
						context.fillRect(0, 0, 1, 1);
						return { selector, foreground: rgba(), backgrounds };
					});
				});
				for (const sample of renderedContrast)
					assert.ok(
						sample.backgrounds.every(
							(background) => contrastRatio(sample.foreground, background) >= 4.5
						),
						`actual composed text contrast fails: ${sample.selector}`
					);
				await writeFile(
					`${output}/contrast-${style}-${theme}.json`,
					JSON.stringify({ palette, renderedContrast }, null, 2)
				);
				await tab(page, 'Addons');
				await page.evaluate(() => (window.__addonFixture.failNextInspect = true));
				await page.getByRole('button', { name: 'Seleccionar .tfl', exact: true }).click();
				await page.getByRole('alert').filter({ hasText: 'Archivo de prueba inválido' }).waitFor();
				await page.locator('.addon-settings .error').scrollIntoViewIfNeeded();
				await page.screenshot({ path: `${output}/alert-${style}-${theme}.png` });
				const alertSample = await page.evaluate(() => {
					const element = document.querySelector('.addon-settings .error');
					if (!element) throw new Error('Missing real addon validation alert');
					const canvas = document.createElement('canvas');
					canvas.width = canvas.height = 1;
					const ctx = canvas.getContext('2d');
					if (!ctx) throw new Error('Canvas required');
					const rgba = () => {
						const [r, g, b, a] = ctx.getImageData(0, 0, 1, 1).data;
						return `rgba(${r},${g},${b},${a / 255})`;
					};
					const chain = [];
					for (let node = element; node; node = node.parentElement) chain.push(node);
					const backgrounds = ['#000000', '#ffffff'].map((color) => {
						ctx.fillStyle = color;
						ctx.fillRect(0, 0, 1, 1);
						for (const node of chain.toReversed()) {
							ctx.fillStyle = getComputedStyle(node).backgroundColor;
							ctx.fillRect(0, 0, 1, 1);
						}
						return rgba();
					});
					const colors = Object.fromEntries(
						['--color-error', '--color-success', '--color-warning', '--text-muted'].map((key) => {
							ctx.clearRect(0, 0, 1, 1);
							ctx.fillStyle = getComputedStyle(element).getPropertyValue(key);
							ctx.fillRect(0, 0, 1, 1);
							return [key, rgba()];
						})
					);
					ctx.clearRect(0, 0, 1, 1);
					ctx.fillStyle = getComputedStyle(element).color;
					ctx.fillRect(0, 0, 1, 1);
					return {
						backgrounds,
						colors,
						foreground: rgba(),
						errorRgb: getComputedStyle(element).getPropertyValue('--color-error-rgb').trim()
					};
				});
				for (const color of [...Object.values(alertSample.colors), alertSample.foreground])
					assert.ok(
						alertSample.backgrounds.every((background) => contrastRatio(color, background) >= 4.5),
						'real alert and semantic foregrounds remain readable'
					);
				assert.equal(
					alertSample.errorRgb,
					alertSample.colors['--color-error']
						.match(/^rgba\((\d+),(\d+),(\d+),/)
						.slice(1)
						.join(', ')
				);
				await writeFile(
					`${output}/alert-contrast-${style}-${theme}.json`,
					JSON.stringify(alertSample, null, 2)
				);
				await page
					.getByLabel('Perfil seguro: desactivar todos los addons', { exact: true })
					.check();
				assert.equal(
					await page.evaluate(() =>
						document.documentElement.style.getPropertyValue('--text-accent')
					),
					'',
					'safe profile restores previous host accent text token'
				);
				assert.ok(
					await page.evaluate(() =>
						[
							'--raw-main',
							'--raw-sidebar',
							'--raw-card',
							'--raw-input',
							'--raw-item-active',
							'--color-error',
							'--color-error-rgb',
							'--color-success',
							'--color-warning'
						].every((key) => !document.documentElement.style.getPropertyValue(key))
					),
					'safe profile restores host material aliases'
				);
				results.push(
					`${style}/${theme}: full example theme composed text+alert contrast >=4.5; unreadable metadata rejected; semantic RGB+safe-mode restoration`
				);
				await context.close();
				continue;
			}
			await tab(page, 'Fondos y escenas');
			await page.getByText('Paisaje de prueba', { exact: true }).waitFor();
			await tab(page, 'Addons');
			await page.getByLabel('Perfil seguro: desactivar todos los addons', { exact: true }).check();
			assert.equal(
				await page.evaluate(() => localStorage.getItem('tfl-addon-theme-v1')),
				'',
				'safe profile restores host theme'
			);
			await page
				.getByLabel('Perfil seguro: desactivar todos los addons', { exact: true })
				.uncheck();
			await page.evaluate(() => window.__addonFixture.setVersion('1.1.0'));
			await page.getByRole('button', { name: 'Seleccionar .tfl', exact: true }).click();
			await page.locator('.consent input').check();
			await page.getByRole('button', { name: 'Actualizar addon', exact: true }).click();
			await page.getByRole('button', { name: 'Volver a la versión anterior', exact: true }).click();
			await page.evaluate(() => window.__addonFixture.corrupt());
			await page.getByRole('button', { name: 'Actualizar lista', exact: true }).click();
			await page.getByRole('button', { name: 'Recuperar versión anterior', exact: true }).click();
			await page.locator('.installed-addon .enabled').waitFor();
			await page.getByRole('button', { name: 'Desinstalar', exact: true }).click();
			await page.getByRole('button', { name: 'Confirmar desinstalación', exact: true }).click();
			assert.equal(await page.locator('.installed-addon').count(), 0);
			assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
			results.push(
				`${style}/${theme}: inspect consent install enable partial-theme safe-profile update rollback corruption-recovery uninstall`
			);
			await context.close();
		}
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify({ native: false, results, errors }, null, 2)
	);
	console.log(JSON.stringify({ passed: results.length, results }));
} finally {
	await browser.close();
}
