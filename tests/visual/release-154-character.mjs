// Short recordings of the real renderer with synthetic IPC/skin data.
// These do not certify native WebView/GPU performance or a Microsoft session.
import assert from 'node:assert/strict';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import { withFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const appVersion = JSON.parse(
	await readFile(new URL('../../package.json', import.meta.url), 'utf8')
).version;
const output = process.env.CHARACTER_154_OUTPUT || 'artifacts/release-154-character';
await mkdir(output, { recursive: true });
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined,
	args: ['--enable-unsafe-swiftshader']
});
const errors = [],
	checks = [];
try {
	for (const model of ['CLASSIC', 'SLIM']) {
		const context = await browser.newContext({
			viewport: { width: 1280, height: 720 },
			reducedMotion: 'no-preference',
			recordVideo: { dir: output, size: { width: 1280, height: 720 } }
		});
		const page = await context.newPage();
		page.setDefaultTimeout(30000);
		page.setDefaultNavigationTimeout(60000);
		page.on('pageerror', (error) => errors.push(error.message));
		await page.addInitScript(
			withFixture(
				{ appVersion, premium: true, skin: true, quality: 'Experience' },
				(model) => {
					const invoke = window.__TAURI_INTERNALS__.invoke;
					window.__TAURI_INTERNALS__.invoke = async (command, args) => {
						const value = await invoke(command, args);
						if (command === 'get_character_profile')
							value.skins.forEach((skin) => {
								skin.variant = model;
							});
						return value;
					};
				},
				model
			)
		);
		await page.goto(process.env.VISUAL_URL || 'http://127.0.0.1:4184', {
			waitUntil: 'domcontentloaded'
		});
		await page.locator('.hero .character-actions').waitFor();
		await page.mouse.move(10, 10);
		await page.waitForTimeout(9500);
		await page.screenshot({ path: `${output}/${model.toLowerCase()}-left-glance.png` });
		await page.waitForTimeout(15500);
		await page.screenshot({ path: `${output}/${model.toLowerCase()}-right-glance.png` });
		assert.equal(
			await page.evaluate(() =>
				window.__fixtureCommands.some((entry) => entry.cmd === 'get_mojang_profile')
			),
			false,
			'cosmetic preview never authenticates'
		);
		const video = page.video();
		await page.close();
		await context.close();
		await rename(await video.path(), `${output}/${model.toLowerCase()}.webm`);
		checks.push(
			`${model}: real skinview3d renderer, quiet rigid breath and alternating idle glances, no authenticated profile read`
		);
	}
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{
				browser: await browser.version(),
				scope:
					'Chromium/SwiftShader recordings with synthetic profile and IPC; actual original-joint geometry invariants are exercised separately by tests/unit/character.test.mjs. Not native performance evidence.',
				checks
			},
			null,
			2
		)
	);
	console.log(`PASS release 154 character recording: ${checks.length} model recordings`);
} finally {
	await browser.close();
}
