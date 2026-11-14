// Production UI with isolated IPC. Future Fabric release is a presentation fixture, not an install claim.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const output = process.env.SELECTION_OUTPUT || 'artifacts/selection-experience';
await mkdir(output, { recursive: true });
const checks = [],
	errors = [];
try {
	for (const style of ['modern', 'pixel']) {
		const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
		page.on('pageerror', (e) => errors.push(e.message));
		await page.addInitScript(installFixture, { selectionFixtures: true });
		await page.addInitScript(
			(style) => localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style })),
			style
		);
		await page.goto(process.env.VISUAL_URL || 'http://127.0.0.1:4184');
		await page.locator('.hero-name').waitFor();
		for (const [name, managed, shader, release] of [
			['Horizontes', false, true, null],
			['Personal Ornithe', false, true, null],
			['TFL Client 1.8.9', true, true, '0.1.0-preview.1'],
			['TFL Client 26.3', true, false, '0.0.0-test.1']
		]) {
			await page.getByRole('button', { name: 'Biblioteca', exact: true }).first().click();
			await page.locator('.instance-open').filter({ hasText: name }).click();
			const detail = page.locator('.instance-detail');
			await detail.waitFor();
			const tabs = await detail.locator('.tab-btn').allTextContents();
			assert.equal(
				tabs.some((t) => t.trim() === 'Mods'),
				!managed,
				`${name}: mods`
			);
			assert.equal(
				tabs.some((t) => t.trim() === 'Modpacks'),
				!managed,
				`${name}: modpacks`
			);
			assert.equal(
				tabs.some((t) => t.trim() === 'Shaders'),
				shader,
				`${name}: capability`
			);
			assert.ok(tabs.some((t) => /recursos/i.test(t)));
			assert.ok(tabs.some((t) => /capturas/i.test(t)));
			assert.ok(tabs.some((t) => /copias|backups/i.test(t)));
			if (managed) {
				const panel = detail.locator('.managed-client');
				await panel.waitFor();
				assert.ok((await panel.innerText()).includes(release));
				assert.doesNotMatch(
					await detail.locator('.hero').innerText(),
					/ornithe|fabric|\.jar|\d+ mods/i
				);
				assert.ok(tabs.some((t) => /mundos/i.test(t)));
				assert.match(
					await panel.getByRole('link').getAttribute('href'),
					new RegExp('releases/tag/selection-.*-v' + release.replaceAll('.', '\\.'))
				);
				await panel.getByRole('button', { name: 'Buscar actualizaciones', exact: true }).click();
				await panel.getByRole('status').filter({ hasText: 'al día' }).waitFor();
				await panel.getByRole('button', { name: 'Verificar', exact: true }).click();
				await panel.getByRole('status').filter({ hasText: 'Integridad verificada' }).waitFor();
				await panel.getByRole('button', { name: 'Reparar', exact: true }).click();
				await panel.getByRole('status').filter({ hasText: 'Tus mundos' }).waitFor();
				const commands = await page.evaluate(() => window.__fixtureCommands);
				for (const cmd of [
					'check_selection_update',
					'verify_selection_integrity',
					'repair_selection_instance'
				])
					assert.ok(
						commands.some((c) => c.cmd === cmd && c.args.instanceName === name),
						cmd
					);
				assert.equal(await detail.getByRole('button', { name: /exportar|duplicar/i }).count(), 0);
			} else assert.equal(await detail.locator('.managed-client').count(), 0);
			await page.screenshot({ path: `${output}/${style}-${name.replaceAll(' ', '-')}.png` });
			checks.push({ style, name, managed, shader, tabs: tabs.map((t) => t.trim()) });
		}
		await page.close();
	}
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify({ checks, errors, fixture: true }, null, 2)
	);
	console.log(`PASS Selection experience: ${checks.length} client/style cases`);
} finally {
	await browser.close();
}
