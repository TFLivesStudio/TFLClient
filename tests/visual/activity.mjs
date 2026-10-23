// UI evidence uses simulated IPC. Real content IO is verified by the native acceptance example.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const output = process.env.ACTIVITY_OUTPUT || 'artifacts/activity';
await mkdir(output, { recursive: true });
const checks = [],
	errors = [];
try {
	for (const style of ['modern', 'pixel']) {
		const context = await browser.newContext({
			viewport: { width: 800, height: 600 },
			reducedMotion: 'reduce'
		});
		const page = await context.newPage();
		page.on('pageerror', (e) => errors.push(e.message));
		await page.addInitScript(installFixture, { quality: 'Lite' });
		await page.addInitScript((style) => {
			localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
			window.__activitySnapshot = { revision: 0, tasks: [], storage_warning: false };
			window.__activityError = false;
			const base = window.__TAURI_INTERNALS__.invoke;
			window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
				if (cmd === 'get_activity_tasks') {
					if (window.__activityError) throw Error('offline');
					return structuredClone(window.__activitySnapshot);
				}
				if (cmd === 'cancel_activity_task') throw Error('ACTIVITY_NOT_CANCELLABLE');
				return base(cmd, args);
			};
		}, style);
		await page.goto(process.env.VISUAL_URL || 'http://localhost:1437');
		await page.locator('.hero').waitFor();
		await page.getByRole('button', { name: 'Actividades', exact: true }).click();
		await page.getByRole('heading', { name: 'Todo al día' }).waitFor();
		checks.push(`${style}: empty and navigation`);
		await page.evaluate(() => {
			const task = {
				id: '86b91bfc-f924-44ce-9c9f-e28ce045ef87',
				revision: 1,
				kind: 'content.install',
				source: 'mcp',
				target: 'mod',
				instance_id: 'horizons',
				project_id: 'abcdef12',
				status: 'running',
				stage: 'downloading',
				created_at: 1792700000,
				updated_at: 1792700000,
				finished_at: null,
				progress: { current: 1048576, total: 4194304 },
				cancellable: false,
				error: null
			};
			window.__activitySnapshot = { revision: 1, tasks: [task], storage_warning: false };
			window.__fixtureActivityChanged();
		});
		await page.getByRole('heading', { name: 'Instalación de mod · Horizontes' }).waitFor();
		assert.equal(await page.locator('progress').getAttribute('value'), '1048576');
		assert.equal(await page.getByRole('button', { name: 'Solicitar cancelación' }).count(), 0);
		await page.getByText('Detalles técnicos', { exact: true }).focus();
		await page.keyboard.press('Enter');
		await page.getByText('abcdef12', { exact: true }).waitFor();
		await page.screenshot({ path: `${output}/${style}-800.png` });
		checks.push(`${style}: real-byte presentation, MCP, safe detail, keyboard, noncancellable`);
		for (let n = 0; n < 5; n++) {
			await page.getByRole('button', { name: 'Biblioteca', exact: true }).click();
			assert.equal(await page.locator('.progress-stack progress').getAttribute('value'), '1048576');
			await page.getByRole('button', { name: /Actividades/ }).click();
			assert.equal(await page.locator('.progress-stack').count(), 0);
		}
		assert.equal(await page.evaluate(() => window.__fixtureActivityListeners()), 1);
		await page.evaluate(() => {
			const s = window.__activitySnapshot;
			s.revision++;
			s.tasks[0].revision++;
			s.tasks[0].progress = null;
			s.tasks[0].stage = 'resolving';
			window.__fixtureActivityChanged();
		});
		await page.getByText('Progreso indeterminado', { exact: true }).waitFor();
		await page.evaluate(() => {
			const s = window.__activitySnapshot;
			s.revision++;
			s.tasks[0].revision++;
			s.tasks[0].status = 'failed';
			s.tasks[0].error = 'install_failed';
			s.tasks[0].finished_at = 1792700001;
			window.__fixtureActivityChanged();
		});
		await page.getByText('Falló', { exact: true }).waitFor();
		await page.getByRole('button', { name: 'Biblioteca', exact: true }).click();
		assert.equal(
			await page.locator('.progress-stack').count(),
			0,
			'failed activity cannot leave a phantom download'
		);
		await page.getByRole('button', { name: /Actividades/ }).click();
		checks.push(
			`${style}: floating bar shares real task progress and disappears on backend failure`
		);
		await page.evaluate(() => {
			window.__activityError = true;
			window.__fixtureActivityChanged();
		});
		await page.getByRole('button', { name: 'Volver a sincronizar' }).waitFor();
		await page.evaluate(() => {
			window.__activityError = false;
			window.__activitySnapshot.revision++;
			window.__activitySnapshot.tasks[0].status = 'interrupted';
			window.__activitySnapshot.tasks[0].error = 'interrupted';
			window.__activitySnapshot.storage_warning = true;
		});
		await page.getByRole('button', { name: 'Volver a sincronizar' }).click();
		await page.getByText('Interrumpida', { exact: true }).waitFor();
		await page.setViewportSize({ width: 1280, height: 720 });
		await page.screenshot({ path: `${output}/${style}-history.png` });
		for (const theme of ['light', 'dark']) {
			await page.evaluate((theme) => {
				document.documentElement.dataset.theme = theme;
				if (theme === 'dark') document.documentElement.dataset.surface = 'oled';
				else delete document.documentElement.dataset.surface;
			}, theme);
			await page.screenshot({ path: `${output}/${style}-${theme}.png` });
		}
		await page.evaluate(() => {
			const s = window.__activitySnapshot;
			s.revision++;
			s.tasks[0].status = 'cancel_requested';
			s.tasks[0].error = null;
			s.tasks[0].cancellable = true;
			window.__fixtureActivityChanged();
		});
		assert(await page.getByRole('button', { name: 'Solicitar cancelación' }).isDisabled());
		await page.evaluate(() => {
			const s = window.__activitySnapshot;
			s.revision++;
			s.tasks[0].status = 'running';
			window.__fixtureActivityChanged();
		});
		await page.getByRole('button', { name: 'Solicitar cancelación' }).click();
		await page
			.getByText('La operación ya terminó o no acepta cancelación.', { exact: true })
			.waitFor();
		await page.evaluate(() => {
			const s = window.__activitySnapshot;
			s.revision++;
			s.tasks[0].status = 'succeeded';
			s.tasks[0].cancellable = false;
			window.__fixtureActivityChanged();
		});
		await page.getByText('Completada', { exact: true }).waitFor();
		assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
		checks.push(
			`${style}: navigation retains task, one listener, indeterminate, error/recovery, interrupted history, storage warning, reduced motion`
		);
		checks.push(
			`${style}: Light/Dark/OLED, reserved cancel-requested disabled, cancellation rejection stays truthful, success history`
		);
		await context.close();
	}
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify({ ipc: 'simulated', checks, errors }, null, 2)
	);
	console.log(JSON.stringify({ passed: checks.length, checks }));
} finally {
	await browser.close();
}
