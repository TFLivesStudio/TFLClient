// Browser presentation/lifecycle evidence; native authorization is tested separately.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined,
	args: ['--enable-unsafe-swiftshader']
});
const output = process.env.MCP_VISUAL_OUTPUT || 'artifacts/mcp-scenes';
await mkdir(output, { recursive: true });
const checks = [],
	errors = [];
async function open({
	style = 'modern',
	theme = 'dark',
	quality = 'Experience',
	scene = 'forest',
	viewport = { width: 1280, height: 720 },
	language = 'es'
} = {}) {
	const context = await browser.newContext({ viewport });
	const page = await context.newPage();
	page.on('pageerror', (e) => errors.push(e.message));
	await page.addInitScript(installFixture, { theme, quality, language });
	await page.addInitScript(
		({ style, scene }) => {
			localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
			localStorage.setItem(
				'tfl-scenes-v1',
				JSON.stringify({
					version: 1,
					global: { preset: scene, shader: true, motion: 'subtle' },
					instances: {}
				})
			);
			const base = window.__TAURI_INTERNALS__.invoke;
			let enabled = false;
			const client = {
				id: 'client-test',
				name: 'Codex (fixture)',
				connected: true,
				approved: false,
				permissions: [],
				instances: [],
				expires: Date.now() / 1000 + 3600,
				last_activity: Date.now() / 1000
			};
			const plan = {
				id: 'plan-test',
				client_id: 'client-test',
				params: { instance_id: 'horizons', action: 'delete_instance' },
				expires: Date.now() / 1000 + 120,
				approved: false,
				decided: false
			};
			window.__TAURI_INTERNALS__.invoke = async (cmd, args = {}) => {
				const snapshot = () =>
					structuredClone({
						enabled,
						sidecar_path: 'C:\\Program Files\\TFL Client\\tflclient-mcp.exe',
						clients: enabled ? [client] : [],
						plans: enabled ? [plan] : [],
						audit: []
					});
				if (cmd === 'mcp_status') return snapshot();
				if (cmd === 'mcp_enable') {
					enabled = args.enabled;
					return snapshot();
				}
				if (cmd === 'mcp_probe') {
					if (!enabled) throw Error('MCP_DISABLED');
					return { authenticated_ipc: true, unapproved_read_denied: true };
				}
				if (cmd === 'mcp_grant') {
					client.approved = true;
					client.permissions = args.permissions;
					client.instances = args.instances;
					return;
				}
				if (cmd === 'mcp_revoke') {
					client.connected = false;
					client.approved = false;
					return;
				}
				if (cmd === 'mcp_decide') {
					plan.decided = true;
					plan.approved = args.approve;
					return;
				}
				if (cmd === 'mcp_clear_audit') return;
				return base(cmd, args);
			};
			window.__sceneContexts = 0;
			const get = HTMLCanvasElement.prototype.getContext;
			HTMLCanvasElement.prototype.getContext = function (type, ...rest) {
				if (type.startsWith('webgl')) window.__sceneContexts++;
				return get.call(this, type, ...rest);
			};
		},
		{ style, scene }
	);
	await page.goto(process.env.VISUAL_URL || 'http://localhost:1434');
	await page.locator('.hero').waitFor();
	return { page, context };
}
try {
	for (const style of ['modern', 'pixel'])
		for (const viewport of [
			{ width: 800, height: 600 },
			{ width: 1280, height: 720 },
			{ width: 1920, height: 1080 }
		]) {
			const { page, context } = await open({ style, viewport });
			await page.getByRole('button', { name: 'Ajustes', exact: true }).click();
			await page
				.locator('.panel-tabs')
				.getByRole('button', { name: 'Integraciones', exact: true })
				.click();
			await page.getByRole('button', { name: 'Activar MCP', exact: true }).click();
			await page.getByRole('button', { name: 'Autorizar esta sesión', exact: true }).click();
			await page.getByRole('button', { name: 'Probar enlace local', exact: true }).click();
			await page.getByRole('status').filter({ hasText: 'Enlace autenticado correcto' }).waitFor();
			await page.getByRole('button', { name: 'Rechazar', exact: true }).click();
			assert.equal(
				await page.locator('.integrations').evaluate((n) => n.scrollWidth <= n.clientWidth + 2),
				true
			);
			await page.screenshot({ path: `${output}/integrations-${style}-${viewport.width}.png` });
			checks.push(`integration permissions ${style} ${viewport.width}`);
			await context.close();
		}
	for (const scene of ['forest', 'embers', 'void', 'aurora', 'pixel'])
		for (const theme of ['dark', 'light', 'oled']) {
			const { page, context } = await open({
				scene,
				theme: theme === 'oled' ? 'dark' : theme,
				style: scene === 'pixel' ? 'pixel' : 'modern'
			});
			if (theme === 'oled')
				await page.evaluate(() => (document.documentElement.dataset.surface = 'oled'));
			await page.locator('.cinematic-art').waitFor();
			assert(
				await page.locator('.cinematic-art').evaluate((n) => n.complete && n.naturalWidth > 1200)
			);
			await page.screenshot({ path: `${output}/scene-${scene}-${theme}.png` });
			await page.getByRole('button', { name: 'Ajustes', exact: true }).click();
			assert.equal(await page.locator('.living-scene .atmosphere').count(), 0);
			assert.equal(await page.locator('.living-scene canvas').count(), 0);
			checks.push(`${scene} ${theme}: poster + overlay suspension`);
			await context.close();
		}
	const { page, context } = await open({ quality: 'Lite', scene: 'aurora' });
	assert.equal(
		await page
			.locator('.living-scene video,.living-scene canvas,.living-scene .atmosphere')
			.count(),
		0
	);
	checks.push('Lite: no video/WebGL/atmospheric clock');
	await context.close();
	assert.deepEqual(errors, []);
	await writeFile(`${output}/results.json`, JSON.stringify({ checks, errors }, null, 2));
	console.log(`PASS MCP/scenes: ${checks.length} checks`);
} finally {
	await browser.close();
}
