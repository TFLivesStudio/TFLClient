// Synthetic skin/cape textures and IPC; exercises real skinview3d/WebGL, not native GPU certification.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined,
	args: ['--enable-unsafe-swiftshader']
});
const output = process.env.CHARACTER_OUTPUT || 'artifacts/character-interaction';
await mkdir(output, { recursive: true });
const errors = [],
	checks = [];
async function open(quality = 'Experience', reduced = false, cape = true) {
	const page = await browser.newPage({
		viewport: { width: 1280, height: 720 },
		reducedMotion: reduced ? 'reduce' : 'no-preference'
	});
	page.on('pageerror', (e) => errors.push(e.message));
	await page.addInitScript(installFixture, { premium: true, skin: true, quality });
	await page.addInitScript(
		({ cape }) => {
			const texture = document.createElement('canvas');
			texture.width = 64;
			texture.height = 32;
			const ctx = texture.getContext('2d');
			ctx.fillStyle = '#236e9a';
			ctx.fillRect(0, 0, 64, 32);
			ctx.fillStyle = '#e9be65';
			ctx.fillRect(2, 2, 8, 14);
			ctx.fillStyle = '#73e2b3';
			ctx.fillRect(14, 2, 8, 14);
			const capeUrl = texture.toDataURL();
			const invoke = window.__TAURI_INTERNALS__.invoke;
			window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
				const result = await invoke(cmd, args);
				if (cmd === 'get_mojang_profile' && cape)
					result.capes = [{ id: 'synthetic-cape', state: 'ACTIVE', url: capeUrl }];
				return result;
			};
			window.__characterProbe = { draws: 0, contexts: 0, pending: new Set() };
			const request = window.requestAnimationFrame.bind(window),
				cancel = window.cancelAnimationFrame.bind(window);
			window.requestAnimationFrame = (cb) => {
				const id = request((time) => {
					window.__characterProbe.pending.delete(id);
					cb(time);
				});
				window.__characterProbe.pending.add(id);
				return id;
			};
			window.cancelAnimationFrame = (id) => {
				window.__characterProbe.pending.delete(id);
				cancel(id);
			};
			const original = HTMLCanvasElement.prototype.getContext,
				seen = new WeakSet();
			HTMLCanvasElement.prototype.getContext = function (type, ...args) {
				const gl = original.call(this, type, ...args);
				if (gl && type.startsWith('webgl') && !seen.has(gl)) {
					seen.add(gl);
					window.__characterProbe.contexts++;
					for (const name of ['drawElements', 'drawArrays']) {
						const fn = gl[name].bind(gl);
						gl[name] = (...args) => {
							window.__characterProbe.draws++;
							return fn(...args);
						};
					}
				}
				return gl;
			};
		},
		{ cape }
	);
	await page.goto(process.env.VISUAL_URL || 'http://127.0.0.1:4184');
	await page.locator('.hero').waitFor();
	return page;
}
try {
	for (const quality of ['Balanced', 'Experience']) {
		const page = await open(quality);
		const canvas = page.locator('.hero .skin-view canvas');
		await canvas.waitFor();
		await page.waitForFunction(() => window.__characterProbe.draws > 0);
		await page.waitForTimeout(350);
		await page.screenshot({ path: `${output}/${quality.toLowerCase()}-front.png` });
		await page.getByRole('button', { name: 'Saludar', exact: true }).click();
		await page.waitForTimeout(600);
		await page.screenshot({ path: `${output}/${quality.toLowerCase()}-wave.png` });
		await canvas.focus();
		assert.equal(await canvas.evaluate((n) => document.activeElement === n), true);
		await page.keyboard.press('ArrowRight');
		await page.keyboard.press('Enter');
		const rect = await canvas.boundingBox();
		await page.mouse.move(rect.x + rect.width * 0.35, rect.y + rect.height * 0.5);
		await page.mouse.down();
		await page.mouse.move(rect.x + rect.width * 0.95, rect.y + rect.height * 0.5, { steps: 12 });
		await page.mouse.up();
		await page.waitForTimeout(180);
		await page.screenshot({ path: `${output}/${quality.toLowerCase()}-cape-inertia.png` });
		await page.getByRole('button', { name: 'Vista frontal', exact: true }).click();
		await page.waitForTimeout(400);
		await page.getByRole('button', { name: 'Skin y capa', exact: true }).click();
		await page.getByRole('dialog').waitFor();
		await page.waitForTimeout(400);
		// The covered Hero retains its DOM canvas but disposes its renderer; dialog's optional viewer follows its own policy.
		assert.equal(await page.locator('.hero .character-actions').count(), 0);
		await page.keyboard.press('Escape');
		await page.waitForFunction(() => document.querySelector('.hero .character-actions'));
		const before = await page.evaluate(() => window.__characterProbe.contexts);
		assert.ok(before >= 2, 'Hero renderer recreated after dialog');
		await page.locator('.hero .skin-view canvas').focus();
		await page.keyboard.press('Home');
		await page.screenshot({ path: `${output}/${quality.toLowerCase()}-restored.png` });
		checks.push(
			`${quality}: skin+cape, keyboard, wave, drag, front reset, overlay disposal/recreation`
		);
		await page.close();
	}
	for (const [quality, reduced] of [
		['Lite', false],
		['Experience', true]
	]) {
		const page = await open(quality, reduced, false);
		if (quality === 'Lite') {
			await page.locator('.pixel-person').waitFor();
			assert.equal(await page.evaluate(() => window.__characterProbe.contexts), 0);
			await page.getByRole('button', { name: 'Activar vista 3D', exact: true }).click();
		}
		await page.waitForFunction(() => window.__characterProbe.draws > 0);
		await page.waitForTimeout(450);
		const a = await page.evaluate(() => ({
			draws: window.__characterProbe.draws,
			frames: window.__characterProbe.pending.size
		}));
		await page.waitForTimeout(600);
		assert.equal(
			await page.evaluate(() => window.__characterProbe.draws),
			a.draws,
			'static/reduced 3D does not render continuously'
		);
		assert.equal(a.frames, 0, 'static/reduced schedules no animation frame');
		await page.locator('.hero .skin-view canvas').focus();
		await page.keyboard.press('ArrowRight');
		assert.ok(
			(await page.evaluate(() => window.__characterProbe.draws)) > a.draws,
			'explicit rotation still renders on demand'
		);
		assert.equal(
			await page.getByRole('button', { name: 'Saludar', exact: true }).isDisabled(),
			true
		);
		await page.screenshot({ path: `${output}/${quality.toLowerCase()}-static.png` });
		checks.push(
			`${quality}${reduced ? ' reduced' : ''}: zero continuous rendering/RAF; explicit rotation works`
		);
		await page.close();
	}
	const paused = await open('Experience', false, true);
	await paused.waitForFunction(() => window.__characterProbe.draws > 0);
	await paused.evaluate(() => {
		window.__fixtureRunning = [{ name: 'Horizontes', account_uuid: 'fixture-account', pid: 1 }];
		window.__fixtureEmit('InstanceStatusChanged', { name: 'Horizontes', running: true });
	});
	await paused.waitForTimeout(500);
	const stoppedDraws = await paused.evaluate(() => window.__characterProbe.draws);
	await paused.waitForTimeout(500);
	assert.equal(
		await paused.evaluate(() => window.__characterProbe.draws),
		stoppedDraws,
		'game reduce policy suspends character/cape animation'
	);
	assert.equal(await paused.evaluate(() => window.__characterProbe.pending.size), 0);
	await paused.evaluate(() => {
		window.__fixtureRunning = [];
		window.__fixtureEmit('InstanceStatusChanged', { name: 'Horizontes', running: false });
	});
	await paused.waitForFunction((draws) => window.__characterProbe.draws > draws, stoppedDraws);
	await paused
		.locator('.hero .skin-view canvas')
		.evaluate((n) => n.dispatchEvent(new Event('webglcontextlost', { cancelable: true })));
	await paused.locator('.hero .pixel-person').waitFor();
	await paused.waitForTimeout(200);
	assert.equal(
		await paused.evaluate(() => window.__characterProbe.pending.size),
		0,
		'context loss tears down frame subscriber and exposes static skin'
	);
	checks.push(
		'running Minecraft suspends animation; stopping resumes; context loss releases RAF and shows static fallback'
	);
	await paused.close();
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{
				browser: await browser.version(),
				scenario:
					'Chromium with synthetic skin/cape and IPC; not native performance/online Minecraft certification',
				checks
			},
			null,
			2
		)
	);
	console.log('PASS character interaction: ' + checks.length + ' profile/lifecycle scenarios');
} finally {
	await browser.close();
}
