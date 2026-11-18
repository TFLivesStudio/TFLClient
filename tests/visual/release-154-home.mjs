// Production UI + isolated IPC. This proves browser behavior, not native account authentication.
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { withFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const appVersion = JSON.parse(
	await readFile(new URL('../../package.json', import.meta.url), 'utf8')
).version;
const output = process.env.HOME_154_OUTPUT || 'artifacts/release-154-home';
await mkdir(output, { recursive: true });
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined,
	args: ['--enable-unsafe-swiftshader']
});
const checks = [],
	errors = [];
async function open({
	style = 'modern',
	theme = 'dark',
	mode = 'simple',
	empty = false,
	premium = false,
	skin = true,
	quality = 'Lite',
	width = 1100,
	height = 700,
	offlineService = false
} = {}) {
	const page = await browser.newPage({
		viewport: { width, height },
		reducedMotion: quality === 'Lite' ? 'reduce' : 'no-preference'
	});
	page.setDefaultTimeout(30000);
	page.setDefaultNavigationTimeout(60000);
	page.on('pageerror', (error) => errors.push(error.message));
	await page.addInitScript(
		withFixture(
			{ appVersion, theme, empty, premium, skin, quality, mode },
			({ style, offlineService }) => {
				localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
				const invoke = window.__TAURI_INTERNALS__.invoke;
				window.__TAURI_INTERNALS__.invoke = async (command, args) => {
					if (command === 'get_character_profile' && offlineService)
						throw new Error('Synthetic offline service failure');
					const result = await invoke(command, args);
					if (command === 'account_session_statuses')
						return [{ uuid: 'fixture-account', state: 'reauthenticate' }];
					return result;
				};
			},
			{ style, offlineService }
		)
	);
	await page.goto(process.env.VISUAL_URL || 'http://127.0.0.1:4184', {
		waitUntil: 'domcontentloaded'
	});
	try {
		await page.locator('.home-shell').waitFor();
	} catch (error) {
		await page.screenshot({ path: `${output}/open-failure.png` });
		await writeFile(
			`${output}/open-failure.json`,
			JSON.stringify(
				{
					errors,
					body: await page.locator('body').innerText(),
					commands: await page.evaluate(() => window.__fixtureCommands)
				},
				null,
				2
			)
		);
		throw error;
	}
	return page;
}
try {
	for (const style of ['modern', 'pixel'])
		for (const theme of ['light', 'dark'])
			for (const mode of ['simple', 'advanced'])
				for (const empty of [false, true]) {
					const page = await open({ style, theme, mode, empty });
					const home = page.locator('.home-shell');
					await page.getByRole('button', { name: 'Personalizar Inicio', exact: true }).waitFor();
					assert.equal(
						await home.locator('.home-greeting, .quick-actions, .instance-carousel').count(),
						0
					);
					assert.equal(
						await home
							.getByRole('button', { name: /Crear instancia|Explorar TFL Selection/ })
							.count(),
						0
					);
					assert.equal(await home.getByText('Otras instancias', { exact: true }).count(), 0);
					if (!empty) {
						await home.locator('.pixel-person').waitFor();
						assert.match(
							await home
								.locator('.character-figure .pixel-person')
								.evaluate((n) => getComputedStyle(n).filter),
							/grayscale\(1\)/
						);
						assert.match(await home.locator('.status-list').innerText(), /Synthetic test CPU/);
						assert.match(await home.locator('.status-list').innerText(), /Synthetic test GPU/);
						assert.equal(await home.getByText(/Java/).count(), 0);
					}
					const customize = home.getByRole('button', { name: 'Personalizar Inicio', exact: true });
					const bounds = await customize.boundingBox();
					assert.ok(
						bounds.width <= 40 && bounds.height <= 40,
						'customization is a compact icon control'
					);
					await customize.focus();
					await page.keyboard.press('Enter');
					await page.getByRole('dialog').waitFor();
					await page.keyboard.press('Escape');
					await page.getByRole('dialog').waitFor({ state: 'hidden' });
					await page.screenshot({
						path: `${output}/${style}-${theme}-${mode}-${empty ? 'empty' : 'content'}.png`,
						fullPage: true
					});
					checks.push(
						`${style}/${theme}/${mode}/${empty ? 'empty' : 'content'}: compact keyboard customization, no duplicate actions/greeting/carousel, hardware scope`
					);
					await page.close();
				}
	for (const offlineService of [false, true]) {
		const page = await open({ premium: true, quality: 'Balanced', offlineService });
		await page.locator('.hero .skin-view canvas').waitFor();
		assert.equal(
			await page.locator('.hero .account-offline').count(),
			0,
			'an expired official account is never displayed as offline'
		);
		assert.equal(
			await page.evaluate(() =>
				window.__fixtureCommands.some((c) => c.cmd === 'get_mojang_profile')
			),
			false,
			'opening Home never refreshes authentication'
		);
		if (offlineService) await page.locator('.hero .skin-view canvas').waitFor();
		await page.screenshot({
			path: `${output}/official-expired-${offlineService ? 'fallback' : 'cosmetic'}.png`
		});
		checks.push(
			`official expired, ${offlineService ? 'service unavailable' : 'cached cosmetic'}: no grayscale, no authenticated profile read`
		);
		await page.close();
	}
	for (const [width, height] of [
		[800, 600],
		[1600, 1000]
	]) {
		const page = await open({ width, height });
		const overflow = await page
			.locator('.home-shell')
			.evaluate((n) => n.scrollWidth > n.clientWidth + 1);
		assert.equal(overflow, false, `Home fits ${width}x${height}`);
		await page.screenshot({ path: `${output}/${width}x${height}.png` });
		checks.push(`${width}x${height}: no horizontal Home overflow`);
		await page.close();
	}
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{
				browser: await browser.version(),
				scope:
					'Chromium with simulated IPC and synthetic hardware/profile data. Does not verify native hardware, authentication, Windows scaling or server access.',
				checks
			},
			null,
			2
		)
	);
	console.log(`PASS release 154 Home: ${checks.length} browser scenarios`);
} finally {
	await browser.close();
}
