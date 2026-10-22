// Production UI, simulated IPC only. Does not authenticate a real Microsoft account.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const output = process.env.AUTH_OUTPUT || 'artifacts/auth';
await mkdir(output, { recursive: true });
const errors = [],
	checks = [];
function authFixture({ style, onboarding, language }) {
	localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
	const base = window.__TAURI_INTERNALS__.invoke;
	const user = {
		username: 'Alex',
		uuid: 'fixture-account',
		user_type: 'Microsoft',
		session_only: false
	};
	let operation = null;
	let count = 0;
	window.__authNext = null;
	window.__authCalls = [];
	window.__TAURI_INTERNALS__.invoke = async (cmd, args = {}) => {
		window.__authCalls.push({ cmd, args });
		if (cmd === 'get_settings') {
			const s = await base(cmd, args);
			return { ...s, onboarded: !onboarding, language };
		}
		if (cmd === 'get_user_list') return [user];
		if (cmd === 'account_session_statuses')
			return [{ uuid: user.uuid, state: user.session_only ? 'memory' : 'protected' }];
		if (cmd === 'start_microsoft_auth') {
			if (operation && !['error', 'cancelled', 'success', 'expired'].includes(operation.phase))
				throw 'AUTH_BUSY';
			operation = {
				id: `attempt-${++count}`,
				phase: 'waiting',
				method: args.method === 'auto' ? 'device' : args.method,
				domain: 'microsoft.com',
				user_code: args.method === 'auto' || args.method === 'device' ? 'ABCD-EFGH' : null,
				verification_uri: 'https://www.microsoft.com/link',
				error: null,
				user: null,
				expires_in: 300,
				test_build: false
			};
			return { ...operation };
		}
		if (cmd === 'microsoft_auth_status') {
			if (window.__authNext) {
				operation = { ...operation, ...window.__authNext };
				if (
					[
						'storage_required',
						'error',
						'expired',
						'cancelled',
						'success',
						'xbox',
						'xsts',
						'minecraft',
						'profile',
						'saving'
					].includes(operation.phase)
				) {
					operation.user_code = null;
					operation.verification_uri = null;
				}
				window.__authNext = null;
			}
			return { ...operation };
		}
		if (cmd === 'cancel_microsoft_auth') {
			operation = { ...operation, phase: 'cancelled' };
			return { ...operation };
		}
		if (cmd === 'microsoft_auth_memory_session') {
			user.session_only = true;
			operation = { ...operation, phase: 'success', user, error: null };
			return { ...operation };
		}
		if (cmd === 'microsoft_auth_retry_storage') {
			operation = { ...operation, phase: 'storage_required', error: 'SECURE_STORE_UNAVAILABLE' };
			return { ...operation };
		}
		if (cmd === 'microsoft_auth_open_device') return null;
		return base(cmd, args);
	};
}
async function open({
	style = 'modern',
	onboarding = false,
	width = 1280,
	height = 720,
	language = 'es'
} = {}) {
	const context = await browser.newContext({
		viewport: { width, height },
		reducedMotion: 'reduce'
	});
	const page = await context.newPage();
	page.setDefaultTimeout(10000);
	page.on('pageerror', (error) => errors.push(error.message));
	await page.addInitScript(installFixture, { quality: 'Lite', language });
	await page.addInitScript(authFixture, { style, onboarding, language });
	await page.goto(process.env.VISUAL_URL || 'http://localhost:1436');
	if (!onboarding) {
		await page.locator('.hero').waitFor();
		await page
			.getByRole('button', { name: language === 'en' ? 'Settings' : 'Ajustes', exact: true })
			.click();
		await page
			.locator('.panel-tabs')
			.getByRole('button', { name: language === 'en' ? 'Accounts' : 'Cuentas', exact: true })
			.click();
		await page
			.getByRole('button', {
				name: language === 'en' ? 'Sign in with Microsoft' : 'Iniciar con Microsoft',
				exact: true
			})
			.click();
	} else {
		await page.locator('.onboarding').waitFor();
		await page.getByRole('button', { name: /Microsoft/ }).click();
	}
	await page.locator('.auth-device code').waitFor();
	return page;
}
async function bounds(page) {
	assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
	const overflow = await page
		.locator('.microsoft-sign-in')
		.evaluate((n) => n.scrollWidth > n.clientWidth + 2);
	assert.equal(overflow, false);
	assert.equal(await page.locator('input[type=password], iframe').count(), 0);
}
try {
	for (const style of ['modern', 'pixel']) {
		for (const onboarding of [false, true]) {
			for (const width of [800, 1280]) {
				const page = await open({ style, onboarding, width, height: width === 800 ? 600 : 720 });
				await bounds(page);
				await page.screenshot({
					path: `${output}/${style}-${onboarding ? 'onboarding' : 'accounts'}-${width}.png`
				});
				checks.push(`${style}/${onboarding ? 'onboarding' : 'accounts'}/${width}`);
				await page.context().close();
			}
		}
	}
	const page = await open();
	await page.getByRole('button', { name: 'Abrir Microsoft', exact: true }).click();
	assert.ok(
		await page.evaluate(() =>
			window.__authCalls.some((c) => c.cmd === 'microsoft_auth_open_device')
		)
	);
	await page.getByRole('button', { name: 'Usar navegador', exact: true }).click();
	await page.getByText('Completa el inicio de sesión en tu navegador', { exact: false }).waitFor();
	assert.equal(await page.locator('.auth-device').count(), 0);
	const switched = await page.evaluate(() =>
		window.__authCalls.filter((c) =>
			['start_microsoft_auth', 'cancel_microsoft_auth'].includes(c.cmd)
		)
	);
	assert.deepEqual(
		switched.map((c) => c.cmd),
		['start_microsoft_auth', 'cancel_microsoft_auth', 'start_microsoft_auth']
	);
	checks.push('method switch cancels the previous Rust operation');
	await page.evaluate(
		() => (window.__authNext = { phase: 'error', error: 'MINECRAFT_APP_NOT_APPROVED' })
	);
	await page.getByRole('alert').filter({ hasText: '403 Invalid app registration' }).waitFor();
	await page.getByRole('button', { name: 'Reintentar', exact: true }).click();
	await page.locator('.auth-device code').waitFor();
	await page.evaluate(
		() => (window.__authNext = { phase: 'storage_required', error: 'SECURE_STORE_UNAVAILABLE' })
	);
	await page.getByRole('button', { name: 'Usar sesión temporal', exact: true }).waitFor();
	await page.screenshot({ path: `${output}/storage-consent.png` });
	assert.equal(await page.locator('input[type=password]').count(), 0);
	await page.getByRole('button', { name: 'Reintentar guardado seguro', exact: true }).click();
	await page.getByRole('button', { name: 'Usar sesión temporal', exact: true }).click();
	await page.locator('.microsoft-sign-in').waitFor({ state: 'detached' });
	await page.locator('.account-type').filter({ hasText: 'Sesión temporal' }).waitFor();
	checks.push('registration error and explicit temporary-session consent');
	await page.getByRole('button', { name: 'Renovar sesión', exact: true }).click();
	await page.locator('.auth-device code').waitFor();
	const renewal = await page.evaluate(() =>
		window.__authCalls.filter((c) => c.cmd === 'start_microsoft_auth').at(-1)
	);
	assert.equal(renewal.args.expectedUuid, 'fixture-account');
	await page
		.locator('.microsoft-sign-in')
		.getByRole('button', { name: 'Cancelar', exact: true })
		.click();
	await page.locator('.microsoft-sign-in').waitFor({ state: 'detached' });
	checks.push('renew existing account and cancel');
	await page.context().close();
	const closed = await open();
	await closed
		.locator('.settings-refresh')
		.getByRole('button', { name: 'Cerrar', exact: true })
		.click();
	await closed.locator('.settings-refresh').waitFor({ state: 'detached' });
	assert.ok(
		await closed.evaluate(() => window.__authCalls.some((c) => c.cmd === 'cancel_microsoft_auth'))
	);
	checks.push('closing settings cancels pending sign-in');
	await closed.context().close();
	const english = await open({ language: 'en' });
	await english.getByRole('button', { name: 'Open Microsoft', exact: true }).waitFor();
	await english.getByRole('button', { name: 'Use browser', exact: true }).click();
	await english.getByText('Finish signing in in your browser', { exact: false }).waitFor();
	await bounds(english);
	await english.screenshot({ path: `${output}/english-external.png` });
	await english.keyboard.press('Escape');
	await english.locator('.settings-refresh').waitFor({ state: 'detached' });
	assert.ok(
		await english.evaluate(() => window.__authCalls.some((c) => c.cmd === 'cancel_microsoft_auth'))
	);
	checks.push('English labels and keyboard cancellation');
	await english.context().close();
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify({ simulatedIPC: true, checks, errors }, null, 2)
	);
	console.log(`${checks.length} authentication UI checks passed`);
} finally {
	await browser.close();
}
