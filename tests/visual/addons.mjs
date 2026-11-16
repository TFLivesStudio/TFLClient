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
		}
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
				await tab(page, 'Addons');
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
				results.push(
					`${style}/${theme}: full example theme readable accent text >=4.5 on five surfaces; safe-mode token restoration`
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
