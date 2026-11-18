// Browser UI evidence with isolated, explicitly simulated IPC. Native settings,
// filesystem edits and external MCP authorization are verified by separate gates.
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { withFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const output = process.env.SETTINGS_154_OUTPUT || 'artifacts/release-154-settings';
const url = process.env.VISUAL_URL || 'http://127.0.0.1:1423';
const appVersion = JSON.parse(
	await readFile(new URL('../../package.json', import.meta.url), 'utf8')
).version;
const manifest = JSON.parse(
	await readFile(
		new URL('../../examples/addons/portuguese-br/manifest.json', import.meta.url),
		'utf8'
	)
);
const flag = `data:image/png;base64,${(await readFile(new URL('../../examples/addons/portuguese-br/assets/br.png', import.meta.url))).toString('base64')}`;
await mkdir(output, { recursive: true });
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const errors = [],
	checks = [];

function settingsFixture({ manifest, flag, style }) {
	localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
	const base = window.__TAURI_INTERNALS__.invoke;
	const saved = () => JSON.parse(localStorage.getItem('release-154-settings-fixture') || '{}');
	let installed = false,
		addonEnabled = false,
		mcpEnabled = false;
	const client = {
		id: 'release-test-session',
		name: 'Test agent (self-declared)',
		connected: true,
		approved: false,
		permissions: [],
		instances: [],
		expires: Date.now() / 1000 + 3600,
		last_activity: Date.now() / 1000
	};
	const target = (client, label, id, installed = true) => ({
		id,
		client,
		label,
		path: `C:\\Profiles\\Example\\${id}.config`,
		installed,
		supported: true,
		configured: false,
		managed: false,
		revision: '0',
		reason: installed ? null : 'No se detectó el cliente instalado. Usa la configuración manual.'
	});
	let targets = [
		target('codex', 'Codex · user', 'codex-user'),
		target('codex', 'Codex · alternative profile', 'codex-profile'),
		target('opencode', 'OpenCode · user', 'opencode-user', false),
		target('claude', 'Claude Code · user', 'claude-user')
	];
	const catalog = () => ({
		safeMode: false,
		addons: installed
			? [{ manifest, digest: 'a'.repeat(64), enabled: addonEnabled, rollbackAvailable: false }]
			: [],
		brokenAddons: [],
		errors: []
	});
	const status = () => ({
		enabled: mcpEnabled,
		sidecar_path: 'C:\\Programs\\TFL Client\\tflclient-mcp.exe',
		clients: mcpEnabled ? [client] : [],
		plans: [],
		audit: []
	});
	window.__release154 = {
		events: [],
		failConfig: false,
		failSettings: false,
		clipboard: '',
		targets: () => structuredClone(targets)
	};
	window.__TAURI_INTERNALS__.invoke = async (cmd, args = {}) => {
		if (cmd === 'get_settings') return { ...(await base(cmd, args)), ...saved() };
		if (cmd === 'patch_settings') {
			if (window.__release154.failSettings) throw Error('SETTINGS_WRITE_DENIED');
			const next = await base(cmd, args);
			const allowed = Object.fromEntries(
				Object.entries(args.patch ?? {}).filter(([key]) => ['locale', 'ui_mode'].includes(key))
			);
			localStorage.setItem(
				'release-154-settings-fixture',
				JSON.stringify({ ...saved(), ...allowed })
			);
			return { ...next, ...saved() };
		}
		if (cmd.startsWith('addons_') || cmd.startsWith('mcp_'))
			window.__release154.events.push({ cmd, args: structuredClone(args) });
		if (cmd === 'addons_list') return catalog();
		if (cmd === 'addons_pick_file') return 'C:\\Fixtures\\portuguese-br.tfl';
		if (cmd === 'addons_inspect')
			return { manifest, digest: 'a'.repeat(64), installedVersion: installed ? '1.0.0' : null };
		if (cmd === 'addons_install') {
			installed = true;
			return catalog();
		}
		if (cmd === 'addons_set_enabled') {
			addonEnabled = args.enabled;
			return catalog();
		}
		if (cmd === 'addons_remove') {
			installed = false;
			addonEnabled = false;
			return catalog();
		}
		if (cmd === 'addons_resource') return flag;
		if (cmd === 'mcp_status') return structuredClone(status());
		if (cmd === 'mcp_enable') {
			mcpEnabled = args.enabled;
			return structuredClone(status());
		}
		if (cmd === 'mcp_quick_connect_targets') return structuredClone(targets);
		if (cmd === 'mcp_quick_connect_write') {
			if (window.__release154.failConfig) throw Error('MCP_CONFIG_WRITE_DENIED');
			targets = targets.map((target) =>
				target.id === args.targetId
					? {
							...target,
							configured: args.enabled,
							managed: args.enabled,
							revision: String(Number(target.revision) + 1)
						}
					: target
			);
			return structuredClone(targets);
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
		if (cmd === 'plugin:clipboard-manager|write_text') {
			window.__release154.clipboard = args.text;
			return;
		}
		return base(cmd, args);
	};
}
async function open(style, theme, width) {
	const context = await browser.newContext({
		viewport: { width, height: width === 800 ? 600 : 700 },
		reducedMotion: 'reduce'
	});
	const page = await context.newPage();
	page.setDefaultTimeout(15000);
	page.on('pageerror', (error) => errors.push(error.message));
	await page.addInitScript(
		withFixture(
			{ appVersion, theme, mode: 'simple', quality: 'Lite', language: 'es' },
			settingsFixture,
			{ manifest, flag, style }
		)
	);
	await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 60000 });
	await page.locator('.home-shell').waitFor();
	await page.getByRole('button', { name: 'Ajustes', exact: true }).click();
	return { page, context };
}
async function tab(page, name) {
	await page.locator('.panel-tabs').getByRole('button', { name, exact: true }).click();
}
async function screenshot(page, name) {
	assert.equal(
		await page.evaluate(() => document.documentElement.scrollWidth > innerWidth),
		false,
		`${name}: no document overflow`
	);
	await page.screenshot({ path: `${output}/${name}.png` });
	checks.push(name);
}
try {
	for (const style of ['modern', 'pixel'])
		for (const theme of ['light', 'dark']) {
			const { page, context } = await open(style, theme, theme === 'light' ? 800 : 1100);
			try {
				assert.equal(
					await page
						.locator('.panel-tabs')
						.getByRole('button', { name: 'Actualizaciones', exact: true })
						.count(),
					0
				);
				const pages = await page.locator('.panel-tabs button').allTextContents();
				for (const label of pages.map((text) => text.trim()).filter(Boolean)) {
					await tab(page, label);
					assert.equal(
						await page
							.getByRole('button', {
								name: /^(?:Aplicar(?: escena| cambios)?|Guardar(?: opciones de inicio| cambios| preferencias)?)$/
							})
							.count(),
						0,
						`Simple ${label}: preferences autosave`
					);
				}
				await tab(page, 'Integraciones');
				const quick = page.locator('.quick-connect');
				assert.equal(
					await quick.getByRole('switch').count(),
					4,
					'exactly four primary MCP switches'
				);
				assert.equal(
					await quick.locator('#mcp-connect-codex').isEnabled(),
					false,
					'ambiguous profile must be selected'
				);
				await quick.locator('#mcp-destination-codex').selectOption('codex-user');
				assert.match(
					await quick.innerText(),
					/codex-user\.config/,
					'destination is shown before modifying it'
				);
				assert.equal(
					await quick.locator('#mcp-connect-opencode').isEnabled(),
					false,
					'absent client is not fake connected'
				);
				await quick.locator('#mcp-local-toggle').check();
				await page.getByRole('button', { name: 'Autorizar lectura', exact: true }).waitFor();
				await page.evaluate(() => (window.__release154.failConfig = true));
				await quick.locator('#mcp-connect-codex').click();
				await quick.getByRole('alert').filter({ hasText: 'MCP_CONFIG_WRITE_DENIED' }).waitFor();
				assert.equal(
					await quick.locator('#mcp-connect-codex').isChecked(),
					false,
					'failed writes preserve committed switch state'
				);
				await page.evaluate(() => (window.__release154.failConfig = false));
				await quick.locator('#mcp-connect-codex').check();
				await page.waitForFunction(
					() =>
						window.__release154.targets().find((target) => target.id === 'codex-user').configured
				);
				assert.equal(
					await page.getByRole('button', { name: 'Autorizar lectura', exact: true }).count(),
					1,
					'configuration does not grant authorization'
				);
				await page.getByRole('button', { name: 'Autorizar lectura', exact: true }).click();
				assert.deepEqual(
					await page.evaluate(
						() =>
							window.__release154.events.find((event) => event.cmd === 'mcp_grant').args.permissions
					),
					['read']
				);
				await quick.getByRole('button', { name: /Copiar prompt/ }).click();
				const prompt = await page.evaluate(() => window.__release154.clipboard);
				assert.match(prompt, /consulta de solo lectura/);
				assert.match(prompt, /Si no puedes comprobarla/);
				assert.doesNotMatch(prompt, /C:\\|tflclient-mcp\.exe|Bearer /i);
				await screenshot(page, `${style}-${theme}-simple-integrations`);
				await tab(page, 'Addons');
				await page.getByRole('button', { name: 'Seleccionar .tfl', exact: true }).click();
				await page.locator('.inspection').waitFor();
				assert.match(await page.locator('.inspection').innerText(), /Permisos[\s\S]*Ninguno/);
				assert.equal(
					await page.locator('.inspection').getByText('SHA-256', { exact: true }).count(),
					0
				);
				await page.locator('.consent input').check();
				await page.getByRole('button', { name: 'Instalar addon', exact: true }).click();
				await page.locator('.installed-addon .enabled input').check();
				await tab(page, 'General');
				await page.evaluate(() => (window.__release154.failSettings = true));
				await page
					.locator('.language-picker')
					.getByRole('button', { name: /Português \(Brasil\)/ })
					.click();
				await page.locator('.language-picker').getByRole('alert').waitFor();
				assert.equal(await page.locator('html').getAttribute('lang'), 'es-419');
				await page.evaluate(() => (window.__release154.failSettings = false));
				await page
					.locator('.language-picker')
					.getByRole('button', { name: 'Reintentar', exact: true })
					.click();
				await page.waitForFunction(() => document.documentElement.lang === 'pt-BR');
				assert.match(await page.locator('.language-picker').innerText(), /3 idiomas disponíveis/);
				await screenshot(page, `${style}-${theme}-language-addon`);
				await tab(page, 'Backgrounds & scenes');
				assert.equal(
					await page.locator('.settings-page-heading h3').textContent(),
					'Backgrounds & scenes'
				);
				await tab(page, 'Addons');
				await page.locator('.installed-addon .enabled input').uncheck();
				await page.waitForFunction(() => document.documentElement.lang === 'en-US');
				checks.push(
					`${style}/${theme}: addon install/consent/enable/select/error/retry/disable/fallback`
				);
				const mode = page.locator('.experience-switch');
				await mode.getByRole('button', { name: 'Advanced', exact: true }).click();
				await page
					.locator('.panel-tabs')
					.getByRole('button', { name: 'Updates', exact: true })
					.waitFor();
				await screenshot(page, `${style}-${theme}-advanced`);
				await page.reload({ waitUntil: 'domcontentloaded' });
				await page.locator('.home-shell').waitFor();
				await page.getByRole('button', { name: 'Settings', exact: true }).click();
				assert.equal(
					await mode
						.getByRole('button', { name: 'Advanced', exact: true })
						.getAttribute('aria-pressed'),
					'true'
				);
				checks.push(`${style}/${theme}: Advanced mode persisted on reload`);
			} finally {
				await context.close();
			}
		}
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{ scenario: 'Chromium with isolated simulated IPC; no native certification', checks, errors },
			null,
			2
		)
	);
	console.log(`PASS 0.15.4 settings: ${checks.length} checks`);
} finally {
	await browser.close();
}
