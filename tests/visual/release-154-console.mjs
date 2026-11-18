// Capture the product console with synthetic IPC. Native providers have separate gates.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { withFixture } from './fixture.mjs';

const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const url = process.env.VISUAL_URL || 'http://127.0.0.1:4184';
const output = process.env.CONSOLE_154_OUTPUT || 'artifacts/release-154-console';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || undefined });
const checks = [];
const errors = [];
await mkdir(output, { recursive: true });

function customizeFixture({ style, theme }) {
	localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
	localStorage.setItem('tfl-theme', theme);
	const invoke = window.__TAURI_INTERNALS__.invoke;
	window.__TAURI_INTERNALS__.invoke = async (command, args) => {
		if (command === 'plugin:clipboard-manager|write_text') window.__copiedLog = args.text;
		return invoke(command, args);
	};
}

async function waitForState(page, expectedText) {
	await page.locator('.monitor-caption').getByText(expectedText).waitFor();
}

try {
	for (const style of ['modern', 'pixel']) {
		for (const [mode, theme, width, height] of [
			['simple', 'dark', 1100, 700],
			['advanced', 'light', 800, 600]
		]) {
			const context = await browser.newContext({ viewport: { width, height } });
			try {
				const page = await context.newPage();
				page.setDefaultTimeout(20000);
				page.on('pageerror', (error) => errors.push(error.message));
				await page.addInitScript(
					withFixture({ logWindow: true, telemetry: true, mode, theme }, customizeFixture, {
						style,
						theme
					})
				);
				await page.goto(url, { waitUntil: 'domcontentloaded' });
				await page.locator('.console-view').waitFor();
				await waitForState(page, /Monitoreo/);
				assert.equal(await page.locator('[data-console-metric]').count(), 6);
				const cpu = page.locator('[data-console-metric="cpu"]');
				const network = page.locator('[data-console-metric="network"]');
				const io = page.locator('[data-console-metric="disk"]');
				const gpu = page.locator('[data-console-metric="gpu"]');
				await page.waitForFunction(() => {
					const value = document.querySelector('[data-console-metric="cpu"] .metric-value');
					return Number.parseFloat(value?.textContent) > 100;
				});
				assert.match(await cpu.innerText(), /800%/);
				assert.match(await network.innerText(), /Interfaces del equipo/);
				assert.match(await io.innerText(), /Discos del equipo/);
				assert.match(await gpu.innerText(), /Synthetic GPU/);
				await page.waitForFunction(() =>
					document
						.querySelector('[data-console-metric="cpu"] .trace')
						?.getAttribute('d')
						?.includes('L')
				);
				const activeBounds = await cpu.boundingBox();
				await page.screenshot({ path: `${output}/${style}-${theme}-${mode}-active.png` });

				await page.evaluate(() => {
					window.__fixtureEmit('InstanceLogLine', {
						instance: 'Horizontes',
						stream: 'stdout',
						line: '[main/INFO]: Ready'
					});
					window.__fixtureEmit('InstanceLogLine', {
						instance: 'Horizontes',
						stream: 'stderr',
						line: '[main/WARN]: Slow tick'
					});
				});
				await page.getByText('[main/INFO]: Ready', { exact: true }).waitFor();
				await page.getByRole('combobox').selectOption('warn');
				assert.equal(await page.getByText('[main/INFO]: Ready', { exact: true }).count(), 0);
				await page.getByRole('button', { name: 'Copiar registro visible' }).click();
				assert.equal(await page.evaluate(() => window.__copiedLog), '[main/WARN]: Slow tick');
				await page.getByRole('searchbox').fill('no match');
				assert.equal(await page.getByText('[main/WARN]: Slow tick', { exact: true }).count(), 0);
				await page.getByRole('searchbox').clear();
				await page.getByRole('checkbox', { name: /Seguir las nuevas líneas/ }).uncheck();

				await page.evaluate(() => {
					window.__fixtureStatsMode = 'missing';
					window.__fixtureEmit('InstanceExited', { instance: 'Horizontes', code: 0 });
				});
				await waitForState(page, /Instancia detenida/);
				assert.equal(await cpu.locator('.metric-value').textContent(), '—');
				assert.equal(await cpu.locator('.trace').getAttribute('d'), '');
				assert.match(await gpu.innerText(), /No disponible/);
				const idleBounds = await cpu.boundingBox();
				assert.equal(idleBounds.width, activeBounds.width);
				assert.ok(Math.abs(idleBounds.height - activeBounds.height) < 2);
				await page.screenshot({ path: `${output}/${style}-${theme}-${mode}-stopped.png` });
				const count = () =>
					page.evaluate(
						() =>
							window.__fixtureCommands.filter((event) => event.cmd === 'get_console_metrics').length
					);
				const sampled = await count();
				await page.waitForTimeout(2300);
				assert.equal(await count(), sampled, 'stopped instance does not poll in the background');
				checks.push(
					`${style}/${theme}/${mode}: six metrics, multicore CPU, scope, stop, fixed card dimensions, search, levels, copy, scroll and no idle polling`
				);
			} finally {
				await context.close();
			}
		}
	}
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{
				scope:
					'Chromium with isolated simulated IPC and synthetic telemetry. Native CPU, network, disk and GPU are covered separately.',
				checks,
				errors
			},
			null,
			2
		)
	);
	console.log(`PASS 0.15.4 console: ${checks.length} browser scenarios`);
} finally {
	await browser.close();
}
