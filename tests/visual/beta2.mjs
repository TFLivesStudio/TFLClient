// Production UI + deterministic IPC fixtures; native telemetry is covered separately by Rust tests.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const url = process.env.VISUAL_URL || 'http://localhost:1424';
const output = 'artifacts/beta2';
await mkdir(output, { recursive: true });
const errors = [],
	results = [];
async function open(options = {}) {
	const context = await browser.newContext({
		viewport: options.viewport || { width: 1280, height: 800 }
	});
	const page = await context.newPage();
	page.setDefaultTimeout(12000);
	page.on('pageerror', (e) => errors.push(e.message));
	await page.addInitScript(installFixture, options);
	await page.addInitScript(
		({ style }) => localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style })),
		{ style: options.style || 'pixel' }
	);
	await page.goto(url);
	await page
		.locator(options.logWindow ? '.console-view' : '.hero,.empty-state')
		.first()
		.waitFor();
	return page;
}
async function shot(p, name) {
	await p.screenshot({ path: `${output}/${name}.png` });
	results.push(name);
}
async function bounds(p) {
	assert.ok(
		await p.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
		'horizontal overflow'
	);
}
try {
	for (const style of ['modern', 'pixel']) {
		const p = await open({ style, premium: true, skin: true, username: 'MinecraftPlayer16' });
		await p.locator('.skin-head').waitFor();
		assert.equal(await p.locator('.user-name').textContent(), 'MinecraftPlayer16');
		assert.ok(await p.locator('.user-name').evaluate((n) => n.scrollWidth <= n.clientWidth));
		await shot(p, `sidebar-expanded-${style}`);
		await p.getByRole('button', { name: 'Ajustes', exact: true }).click();
		for (const tab of [
			'General',
			'Apariencia',
			'Minecraft y Java',
			'Cuentas',
			'Fondos y escenas',
			'Consola y monitoreo',
			'Actualizaciones'
		]) {
			await p.locator('.panel-tabs').getByRole('button', { name: tab, exact: true }).click();
			assert.equal(await p.locator('.settings-page-heading h3').textContent(), tab);
			await bounds(p);
			if (tab === 'Apariencia') {
				assert.equal(await p.locator('.ram-row').count(), 0);
				await shot(p, `appearance-${style}`);
			}
			if (tab === 'Minecraft y Java') {
				assert.equal(await p.locator('.ram-row').count(), 1);
				assert.ok(await p.locator('.java-paths label').count());
			}
			if (tab === 'Consola y monitoreo') await shot(p, `console-settings-${style}`);
		}
		const search = p.getByRole('searchbox', { name: 'Buscar ajustes' });
		await search.fill('RAM');
		assert.equal(await p.locator('.settings-result').count(), 2);
		await search.fill('idioma');
		assert.equal(await p.locator('.settings-result').count(), 1);
		await search.fill('');
		await p.keyboard.press('Escape');
		await p.getByRole('button', { name: 'Explorar', exact: true }).click();
		await p.locator('.context-list button').first().click();
		await p.locator('.result-row').first().waitFor();
		assert.equal(await p.locator('.result-row').count(), 20);
		await p.getByRole('button', { name: 'Siguiente', exact: true }).click();
		await p.getByText('Fixture project 21', { exact: true }).waitFor();
		assert.ok(
			await p.evaluate(() =>
				window.__fixtureCommands.some((c) => c.cmd === 'search_mods' && c.args.offset === 20)
			)
		);
		await p.locator('.result-title-btn').first().click();
		await p.getByText('Full project details', { exact: true }).waitFor();
		await shot(p, `discovery-detail-${style}`);
		await p.keyboard.press('Escape');
		assert.equal(await p.getByRole('dialog').count(), 0);

		await p.getByRole('button', { name: 'Horizontes', exact: true }).click();
		await p.locator('.tabs').getByRole('button', { name: 'Mods', exact: true }).click();
		await p
			.locator('.content-manager .subtabs')
			.getByRole('button', { name: 'Descargar', exact: true })
			.click();
		await p.locator('.result-row').first().waitFor();
		await p.getByRole('button', { name: 'Siguiente', exact: true }).click();
		await p.getByText('Fixture project 21', { exact: true }).waitFor();
		await p.getByLabel('Ordenar por', { exact: true }).selectOption('updated');
		await p.getByText('Fixture project 1', { exact: true }).waitFor();
		await p.locator('.content-manager .search-row input').fill('sodium');
		await p.waitForFunction(() =>
			window.__fixtureCommands.some(
				(c) =>
					c.cmd === 'search_mods' &&
					c.args.query === 'sodium' &&
					c.args.index === 'updated' &&
					c.args.offset === 0
			)
		);
		await p.locator('.tabs').getByRole('button', { name: 'Modpacks', exact: true }).click();
		await p.locator('.result-row').first().waitFor();
		await p.getByRole('button', { name: 'Siguiente', exact: true }).click();
		await p.getByText('Fixture project 21', { exact: true }).waitFor();
		assert.ok(
			await p.evaluate(() =>
				window.__fixtureCommands.some((c) => c.cmd === 'search_modpacks' && c.args.offset === 20)
			)
		);
		await p.getByRole('button', { name: 'Explorar', exact: true }).click();
		await p.setViewportSize({ width: 800, height: 600 });
		await p.locator('.sidebar.collapsed').waitFor();
		await p.waitForFunction(
			() => Math.abs(document.querySelector('.sidebar').getBoundingClientRect().width - 72) < 1
		);
		await bounds(p);
		const sizes = await p
			.locator(
				'.sidebar .home-btn,.sidebar .instance-item,.sidebar .tfl-icon-btn,.sidebar .user-profile'
			)
			.evaluateAll((nodes) =>
				nodes
					.filter((n) => n.getClientRects().length)
					.map((n) => ({
						width: n.getBoundingClientRect().width,
						height: n.getBoundingClientRect().height
					}))
			);
		assert.ok(
			sizes.every((s) => s.width === 40 && s.height === 40),
			JSON.stringify(sizes)
		);
		const footer = await p.locator('.user-chip').boundingBox();
		assert.ok(
			footer.y + footer.height >= 580 && footer.y + footer.height <= 600,
			JSON.stringify(footer)
		);
		await shot(p, `sidebar-compact-${style}`);
		assert.ok(
			(await p.locator('.instance-list').boundingBox()).height >= 40,
			'instance list retains a full control at 800x600'
		);
		await p.getByRole('button', { name: 'TFL Friends', exact: true }).click();
		await p.locator('.tabs').getByRole('button', { name: 'Consola', exact: true }).waitFor();
		await p.close();
	}
	const log = await open({
		logWindow: true,
		telemetry: true,
		viewport: { width: 800, height: 600 }
	});
	await log.waitForFunction(() =>
		document.querySelector('.chart-card strong')?.textContent.includes('%')
	);
	await log.evaluate(() => {
		window.__fixtureEmit('InstanceLogLine', {
			instance: 'Horizontes',
			stream: 'stderr',
			line: '[Server thread/WARN]: Slow tick'
		});
		window.__fixtureEmit('InstanceLogLine', {
			instance: 'Horizontes',
			stream: 'stdout',
			line: '[main/ERROR]: Broken fixture'
		});
		window.__fixtureEmit('InstanceLogLine', {
			instance: 'Horizontes',
			stream: 'stderr',
			line: '[main/INFO]: Started'
		});
	});
	await log.locator('.line.warn').waitFor();
	assert.equal(await log.locator('.line.error').count(), 1);
	assert.equal(await log.locator('.line.info').count(), 1);
	const cpu = log.locator('.chart-card').filter({ hasText: 'CPU' });
	assert.ok(parseFloat(await cpu.locator('strong').textContent()) > 100);
	await log.waitForTimeout(2100);
	assert.ok((await cpu.locator('.trace').getAttribute('d')).includes('L'));
	const colors = await log
		.locator('.line.warn,.line.error')
		.evaluateAll((ns) => ns.map((n) => getComputedStyle(n).color));
	assert.notEqual(colors[0], colors[1]);
	await shot(log, 'console-history-800x600');
	await log.getByRole('button', { name: 'Personalizar consola', exact: true }).click();
	const settings = log.locator('.config-dialog');
	await settings.getByRole('button', { name: 'Lateral', exact: true }).click();
	await settings.locator('.designer-preview').scrollIntoViewIfNeeded();
	const cards = settings.locator('.designer-preview .chart-card');
	const second = await cards.nth(1).boundingBox();
	const handle = await cards.first().locator('.drag-handle').boundingBox();
	await log.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2);
	await log.mouse.down();
	await log.mouse.move(second.x + second.width / 2, second.y + second.height / 2, { steps: 8 });
	await log.mouse.up();
	assert.equal(await cards.first().getAttribute('data-console-metric'), 'ram');
	await settings.locator('.inspector input[type=color]').fill('#12ab34');
	await settings.getByRole('button', { name: 'Aplicar', exact: true }).click();
	await log.keyboard.press('Escape');
	assert.equal(await log.locator('.config-dialog').count(), 0);
	assert.ok(await log.locator('.console-layout.side').count());
	assert.equal(await log.locator('.chart-card').first().getAttribute('data-console-metric'), 'ram');
	// Another webview changing localStorage must update this console without recreating history.
	const sibling = await log.context().newPage();
	await sibling.route('**/__console-pref-test', (route) =>
		route.fulfill({
			contentType: 'text/html',
			body: '<html><body>Preference sync test</body></html>'
		})
	);
	await sibling.goto(url + '/__console-pref-test');
	await sibling.evaluate(() => {
		const p = JSON.parse(localStorage.getItem('tfl-console-v1'));
		p.dock = 'bottom';
		localStorage.setItem('tfl-console-v1', JSON.stringify(p));
	});
	await log.locator('.console-layout.bottom').waitFor();
	await sibling.close();
	const count = await log.evaluate(
		() => window.__fixtureCommands.filter((c) => c.cmd === 'get_running_instance_stats').length
	);
	await log.evaluate(() =>
		window.__fixtureEmit('InstanceExited', { instance: 'Horizontes', code: 0 })
	);
	await log.waitForTimeout(1200);
	assert.equal(
		await log.evaluate(
			() => window.__fixtureCommands.filter((c) => c.cmd === 'get_running_instance_stats').length
		),
		count
	);
	assert.ok(
		(await cpu.locator('.trace').getAttribute('d')).includes('L'),
		'history retained after exit'
	);
	await shot(log, 'console-customized-stopped');
	await log.reload();
	await log.locator('.console-layout.bottom').waitFor();
	assert.equal(await log.locator('.chart-card').first().getAttribute('data-console-metric'), 'ram');
	await log.close();
	// Server uses a separate PID endpoint; unmount stops polling.
	const server = await open({ telemetry: true });
	await server.getByRole('button', { name: 'TFL Friends', exact: true }).click();
	await server.locator('.tabs').getByRole('button', { name: 'Consola', exact: true }).click();
	await server.evaluate(() =>
		window.__fixtureEmit('InstanceStatusChanged', { name: 'TFL Friends', status: 'running' })
	);
	await server.waitForFunction(() =>
		window.__fixtureCommands.some((c) => c.cmd === 'get_running_server_stats')
	);
	await server.getByRole('button', { name: 'Biblioteca', exact: true }).click();
	await server.waitForTimeout(100);
	const calls = await server.evaluate(
		() => window.__fixtureCommands.filter((c) => c.cmd === 'get_running_server_stats').length
	);
	await server.waitForTimeout(1200);
	assert.equal(
		await server.evaluate(
			() => window.__fixtureCommands.filter((c) => c.cmd === 'get_running_server_stats').length
		),
		calls
	);
	await server.close();
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify({ fixture: true, results, errors }, null, 2)
	);
	console.log(
		`PASS beta2: ${results.length} screenshots; settings IA, merged catalogue, sidebar, historical CPU >100, severity, pointer reorder, cross-window preferences, cleanup.`
	);
} finally {
	await browser.close();
}
