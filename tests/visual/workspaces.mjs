import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
const output = process.env.VISUAL_OUTPUT || 'artifacts/visual-refresh';
await mkdir(output, { recursive: true });
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const b = await chromium.launch({
	headless: true,
	...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {})
});
const p = await b.newPage({ viewport: { width: 800, height: 600 } });
const errors = [];
p.on('pageerror', (e) => errors.push(e.message));
p.setDefaultTimeout(10000);
await p.addInitScript(installFixture, { premium: true });
const shot = async (name) => {
	await p.waitForTimeout(200);
	await p.screenshot({ path: `${output}/${name}.png` });
};
try {
	await p.goto(process.env.VISUAL_URL || 'http://localhost:1422');
	await p.getByRole('button', { name: 'Abrir Horizontes', exact: true }).click();
	await p.locator('.tabs').getByRole('button', { name: 'Mundos y backups', exact: true }).click();
	await shot('backups-800x600');
	await p.locator('.tabs').getByRole('button', { name: 'Mods', exact: true }).click();
	await shot('mods-800x600');
	await p.getByRole('button', { name: 'Biblioteca', exact: true }).click();
	await p.getByRole('button', { name: 'Nueva colección', exact: true }).click();
	await p.getByRole('textbox', { name: 'Nombre de la colección' }).fill('Survival');
	await p.getByRole('button', { name: 'Añadir', exact: true }).click();
	assert.ok(await p.getByRole('button', { name: 'Survival', exact: true }).isVisible());
	await p.getByRole('button', { name: 'Eliminar colección: Survival', exact: true }).click();
	assert.equal(await p.locator('.vr-library-row').count(), 4);
	await p.getByRole('button', { name: 'TFL Friends', exact: true }).click();
	await p.locator('.tabs').getByRole('button', { name: 'Consola', exact: true }).click();
	await shot('server-console-800x600');
	await p.getByRole('button', { name: 'Ajustes', exact: true }).click();
	await p.getByRole('button', { name: 'Apariencia', exact: true }).click();
	await shot('settings-800x600');
	await p.keyboard.press('Escape');
	await p.getByRole('button', { name: 'Skin y capa', exact: true }).click();
	await shot('studio-800x600');
	const rect = await p.getByRole('dialog').boundingBox();
	assert.ok(
		rect.x >= 0 && rect.y >= 0 && rect.x + rect.width <= 800 && rect.y + rect.height <= 600
	);
	await p.keyboard.press('Escape');
	assert.deepEqual(errors, []);
	console.log(
		'PASS: compact backups/mods/console/settings/studio and collection add/remove, no page errors.'
	);
} finally {
	await b.close();
}
