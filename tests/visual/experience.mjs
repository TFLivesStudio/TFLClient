// Production UI with explicit simulated IPC. Native codec/GPU and real login are separate manual checks.
import assert from 'node:assert/strict';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
import { SETTINGS, PAGES } from '../../src/lib/settings/catalog.ts';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined,
	args: ['--enable-unsafe-swiftshader']
});
const url = process.env.VISUAL_URL || 'http://localhost:1433';
const output = process.env.EXPERIENCE_OUTPUT || 'artifacts/experience';
await mkdir(output, { recursive: true });
const errors = [],
	checks = [];
const icons = [];
for (const id of (await readdir('src-tauri/resources/instance-icons')).filter((n) =>
	n.endsWith('.svg')
))
	icons.push({
		id: id.slice(0, -4),
		data_url:
			'data:image/svg+xml;base64,' +
			Buffer.from(await readFile('src-tauri/resources/instance-icons/' + id)).toString('base64')
	});
async function open(options = {}) {
	const context = await browser.newContext({
		viewport: options.viewport || { width: 1280, height: 720 },
		reducedMotion: options.reduced ? 'reduce' : 'no-preference'
	});
	const page = await context.newPage();
	page.setDefaultTimeout(12000);
	page.on('pageerror', (e) => errors.push(e.message));
	await page.addInitScript(installFixture, { ...options, iconPresets: icons });
	await page.addInitScript(
		({ style, scene, surface }) => {
			localStorage.setItem('tfl-visual-refresh-v1', JSON.stringify({ style }));
			if (surface) localStorage.setItem('tfl-surface', surface);
			if (scene && !localStorage.getItem('tfl-scenes-v1'))
				localStorage.setItem(
					'tfl-scenes-v1',
					JSON.stringify({ version: 1, global: scene, instances: {} })
				);
			window.__webglContexts = 0;
			const original = HTMLCanvasElement.prototype.getContext;
			HTMLCanvasElement.prototype.getContext = function (type, ...args) {
				if (type.startsWith('webgl')) window.__webglContexts++;
				return original.call(this, type, ...args);
			};
		},
		{ style: options.style || 'modern', scene: options.scene, surface: options.surface }
	);
	await page.goto(url);
	await page.locator('.hero,.console-view').first().waitFor();
	return page;
}
async function settings(p, label) {
	if (!(await p.locator('.settings-refresh').count()))
		await p.getByRole('button', { name: 'Ajustes', exact: true }).click();
	if (label)
		await p.locator('.panel-tabs').getByRole('button', { name: label, exact: true }).click();
}
async function shot(p, name) {
	await p.screenshot({ path: `${output}/${name}.png` });
	checks.push(name);
}
async function bounds(p) {
	const overflow = await p.evaluate(() =>
		[
			...document.querySelectorAll(
				'.settings-refresh,.panel-body,.designer-preview,.console-layout,.gallery,.theme-options,.style-options'
			)
		]
			.filter((n) => n.getClientRects().length)
			.map((n) => ({ name: n.className, width: n.clientWidth, scroll: n.scrollWidth }))
			.filter((n) => n.scroll > n.width + 2)
	);
	assert.deepEqual(overflow, []);
	assert.ok(await p.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
}
try {
	const p = await open();
	await settings(p);
	await p.evaluate(() => {
		localStorage.setItem('tfl-accent', 'violet');
		document.documentElement.setAttribute('data-accent', 'violet');
	});
	await settings(p, 'Apariencia');
	assert.equal(
		await p.getByRole('button', { name: 'Violet', exact: true }).getAttribute('aria-pressed'),
		'true'
	);
	await p.getByRole('button', { name: 'Restaurar naranja TFL', exact: true }).click();
	assert.equal(
		await p.getByRole('button', { name: 'TFL', exact: true }).getAttribute('aria-pressed'),
		'true'
	);
	const seen = [];
	for (const page of PAGES) {
		await settings(p, page.label[0]);
		for (const id of await p
			.locator('[data-setting]')
			.evaluateAll((ns) => ns.map((n) => n.dataset.setting)))
			seen.push(id);
	}
	assert.deepEqual([...seen].sort(), SETTINGS.map((s) => s.id).sort());
	await settings(p, 'Fondos y escenas');
	await p
		.locator('[data-setting="scene-gallery"]')
		.evaluate((el) => el.scrollIntoView({ block: 'start' }));
	await p.locator('[data-setting="scene-gallery"] img').evaluateAll(async (images) => {
		await Promise.all(images.map((img) => img.decode().catch(() => {})));
	});
	await shot(p, 'scene-library');
	const query = p.getByRole('searchbox', { name: 'Buscar ajustes' });
	for (const [text, id] of [
		['memoria', 'ram'],
		['color personalizado', 'accent'],
		['vídeo', 'scene-video'],
		['JVM', 'jvm'],
		['variables entorno', 'environment'],
		['nucleos', 'console-metrics']
	]) {
		await query.fill(text);
		const result = p
			.locator('.settings-result')
			.filter({ hasText: SETTINGS.find((s) => s.id === id).label[0] })
			.first();
		await result.click();
		await p.locator(`[data-setting="${id}"]:focus`).waitFor();
		assert.equal(await p.locator('[data-highlighted]').getAttribute('data-setting'), id);
		if (id === 'accent') await shot(p, 'accent-deep-link');
		if (id === 'scene-video') await shot(p, 'video-deep-link');
		await p.getByRole('button', { name: /Volver a resultados/ }).click();
		assert.equal(await query.inputValue(), text);
	}
	await query.fill('zzq not a setting');
	await p.getByRole('status').filter({ hasText: 'No encontramos' }).waitFor();
	await query.fill('');
	await settings(p, 'Minecraft y Java');
	await p.locator('.ram-row input').first().fill('8192');
	await p.locator('.ram-row input').nth(1).fill('1024');
	const before = await p.evaluate(
		() => window.__fixtureCommands.filter((c) => c.cmd === 'update_settings').length
	);
	await p.getByRole('button', { name: 'Guardar', exact: true }).click();
	await p.getByRole('alert').filter({ hasText: 'valores enteros' }).waitFor();
	assert.equal(
		await p.evaluate(
			() => window.__fixtureCommands.filter((c) => c.cmd === 'update_settings').length
		),
		before
	);
	await p.locator('.ram-row input').first().fill('2048');
	await p.locator('.ram-row input').nth(1).fill('6144');
	await p.getByRole('button', { name: 'Guardar', exact: true }).click();
	await p.getByText('Memoria guardada', { exact: true }).waitFor();
	await p.getByRole('textbox', { name: 'Variables de entorno', exact: true }).fill('BAD NAME=1');
	await p.getByRole('button', { name: 'Guardar opciones de inicio' }).click();
	await p.getByRole('alert').filter({ hasText: 'NOMBRE=valor' }).waitFor();
	await p
		.getByRole('textbox', { name: 'Variables de entorno', exact: true })
		.fill('TFL_DEMO=one=two');
	await p.getByRole('textbox', { name: 'Argumentos de Java', exact: true }).fill('-XX:+UseG1GC');
	await p.getByRole('button', { name: 'Guardar opciones de inicio' }).click();
	await p.waitForFunction(() =>
		window.__fixtureCommands.some(
			(c) => c.cmd === 'update_settings' && c.args.newSettings.env_vars.TFL_DEMO === 'one=two'
		)
	);
	checks.push('canonical-targets-deep-search-ram-java-validation');
	await settings(p, 'Consola y monitoreo');
	assert.equal(
		await p.evaluate(
			() =>
				window.__fixtureCommands.filter(
					(c) => c.cmd.startsWith('get_running_') && c.cmd.endsWith('_stats')
				).length
		),
		0
	);
	const original = await p.evaluate(() => localStorage.getItem('tfl-console-v1'));
	await p.getByRole('button', { name: 'Lateral', exact: true }).click();
	assert.equal(await p.evaluate(() => localStorage.getItem('tfl-console-v1')), original);
	await p
		.locator('.console-designer')
		.getByRole('button', { name: 'Cancelar', exact: true })
		.click();
	assert.equal(await p.locator('.designer-preview .console-layout.side').count(), 0);
	await p.getByRole('button', { name: 'Inferior', exact: true }).click();
	await p.getByRole('button', { name: 'Deshacer', exact: true }).click();
	assert.equal(await p.locator('.designer-preview .console-layout.bottom').count(), 0);
	await p.getByRole('button', { name: 'Lateral', exact: true }).click();
	await p.locator('.designer-preview').scrollIntoViewIfNeeded();
	const cards = p.locator('.designer-preview .chart-card'),
		handle = await cards.first().locator('.drag-handle').boundingBox(),
		target = await cards.nth(1).boundingBox();
	await p.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2);
	await p.mouse.down();
	await p.mouse.move(target.x + target.width / 2, target.y + target.height / 2, { steps: 8 });
	await p.mouse.up();
	assert.equal(await cards.first().getAttribute('data-console-metric'), 'ram');
	await p.locator('.inspector input[type=color]').fill('#12ab34');
	await p.locator('.inspector select').first().selectOption('gauge');
	await p.getByRole('button', { name: 'Aplicar', exact: true }).click();
	const saved = await p.evaluate(() => JSON.parse(localStorage.getItem('tfl-console-v1')));
	assert.equal(saved.order[0], 'ram');
	assert.equal(saved.metricStyles.cpu, 'gauge');
	assert.equal(saved.colors.cpu, '#12ab34');
	await p.locator('.designer-preview').evaluate((el) => el.scrollIntoView({ block: 'center' }));
	await shot(p, 'console-designer-applied');
	await p.keyboard.press('Escape');
	await p.getByRole('button', { name: 'Abrir Horizontes', exact: true }).click();
	await p.getByRole('button', { name: 'Cambiar ícono de la instancia', exact: true }).click();
	await p.getByRole('searchbox', { name: 'Buscar iconos' }).fill('pine');
	await p.locator('.icon-swatch').first().click();
	assert.equal(await p.locator('.icon-preview img').getAttribute('alt'), 'Pine');
	await shot(p, 'original-icon-preview');
	await p.getByRole('button', { name: 'Aplicar', exact: true }).click();
	await p.waitForFunction(() =>
		window.__fixtureCommands.some(
			(c) => c.cmd === 'set_instance_icon_from_preset' && c.args.presetId === 'nature-pine'
		)
	);
	await p.locator('.tabs').getByRole('button', { name: 'Mods', exact: true }).click();
	await p.getByRole('button', { name: 'Descargar', exact: true }).click();
	await p.locator('.result-row').first().waitFor();
	await p.locator('.result-title-btn').first().dblclick();
	await p.waitForFunction(() => window.__fixtureCommands.some((c) => c.cmd === 'install_mod'));
	await p.waitForTimeout(400);
	assert.equal(
		await p.evaluate(() => window.__fixtureCommands.filter((c) => c.cmd === 'install_mod').length),
		1
	);
	assert.equal(await p.getByText('Full project details', { exact: true }).count(), 0);
	await p.locator('.result-title-btn').first().dblclick();
	await p.waitForTimeout(400);
	assert.equal(
		await p.evaluate(() => window.__fixtureCommands.filter((c) => c.cmd === 'install_mod').length),
		1
	);
	await p
		.locator('.result-row')
		.first()
		.getByRole('button', { name: 'Ver versiones', exact: true })
		.click();
	await p.locator('.version-option').first().click();
	await p.waitForFunction(
		() => window.__fixtureCommands.filter((c) => c.cmd === 'install_mod').length === 2
	);
	assert.equal(
		await p.evaluate(
			() => window.__fixtureCommands.filter((c) => c.cmd === 'install_mod').at(-1).args.versionId
		),
		'explicit-version'
	);
	checks.push('console-apply-cancel-undo-pointer-icons-double-click-no-duplicates');
	await p.close();
	// Every style/theme/profile is checked at each required viewport; representative screenshots are retained.
	if (!process.env.EXPERIENCE_SKIP_MATRIX) {
		for (const style of ['modern', 'pixel'])
			for (const theme of ['dark', 'light', 'oled'])
				for (const quality of ['Lite', 'Balanced', 'Experience']) {
					const page = await open({
						style,
						theme: theme === 'light' ? 'light' : 'dark',
						surface: theme === 'oled' ? 'oled' : 'obsidian',
						quality,
						scene: { preset: 'forest', motion: 'subtle', particles: true, fps: 24 }
					});
					await settings(page, 'Apariencia');
					for (const [width, height] of [
						[800, 600],
						[1100, 700],
						[1280, 720],
						[1366, 768],
						[1920, 1080]
					]) {
						await page.setViewportSize({ width, height });
						for (const label of ['Apariencia', 'Fondos y escenas', 'Consola y monitoreo']) {
							await settings(page, label);
							await bounds(page);
							if (width === 1280 || (width === 800 && quality === 'Lite'))
								await shot(
									page,
									`${style}-${theme}-${quality.toLowerCase()}-${width}-${label === 'Apariencia' ? 'appearance' : label === 'Fondos y escenas' ? 'scenes' : 'console'}`
								);
						}
						if (quality === 'Lite') {
							assert.equal(await page.evaluate(() => window.__webglContexts), 0);
							const alpha = await page.locator('.settings-refresh').evaluate((n) => {
								const canvas = document.createElement('canvas');
								canvas.width = canvas.height = 1;
								const context = canvas.getContext('2d');
								context.fillStyle = getComputedStyle(n).backgroundColor;
								context.fillRect(0, 0, 1, 1);
								return context.getImageData(0, 0, 1, 1).data[3];
							});
							assert.equal(alpha, 255, 'Lite dialog surface is opaque without blur');
						}
					}
					if (style === 'pixel') {
						await settings(page, 'General');
						const frames = await page.locator('.settings-search input').evaluate((n) => ({
							border: getComputedStyle(n).borderWidth,
							shadow: getComputedStyle(n).boxShadow
						}));
						assert.equal(frames.border, '0px');
						assert.equal(frames.shadow, 'none');
					}
					await page.close();
				}
		checks.push('matrix-2-styles-3-themes-3-profiles-5-viewports');
	}
	for (const quality of ['Lite', 'Balanced', 'Experience']) {
		const page = await open({
			quality,
			scene: {
				preset: 'aurora',
				mediaId: 'builtin-aurora',
				shader: true,
				balancedVideo: false,
				motion: 'full',
				fps: 24,
				gamePolicy: 'reduce'
			}
		});
		if (quality === 'Experience') {
			await page.locator('.living-scene video').waitFor();
			await page.waitForFunction(() => {
				const v = document.querySelector('.living-scene video');
				return v && !v.paused && v.readyState >= 2;
			});
			await page.evaluate(
				() => (
					Object.defineProperty(document, 'hasFocus', { configurable: true, value: () => false }),
					window.dispatchEvent(new Event('blur'))
				)
			);
			await page.waitForTimeout(80);
			assert.equal(await page.locator('.living-scene video').count(), 0);
			await page.evaluate(
				() => (
					Object.defineProperty(document, 'hasFocus', { configurable: true, value: () => true }),
					window.dispatchEvent(new Event('focus'))
				)
			);
			await page.locator('.living-scene video').waitFor();
			await page.evaluate(() => {
				window.__fixtureRunning = [{ name: 'Horizontes', account_uuid: 'fixture-account' }];
				window.__fixtureEmit('InstanceStatusChanged', { name: 'Horizontes', status: 'running' });
			});
			await page.waitForTimeout(100);
			assert.equal(await page.locator('.living-scene video').count(), 0);
			await page.evaluate(() => {
				window.__fixtureRunning = [];
				window.__fixtureEmit('InstanceStatusChanged', { name: 'Horizontes', status: 'stopped' });
			});
			await page.locator('.living-scene video').waitFor();
		} else assert.equal(await page.locator('video,canvas').count(), 0);
		await settings(page, 'Fondos y escenas');
		assert.equal(await page.locator('.living-scene video').count(), 0);
		await page.getByLabel('Fuente de vídeo').selectOption('builtin-embers');
		await page.locator('.pending-scene').getByRole('button', { name: 'Aplicar escena' }).click();
		await page.keyboard.press('Escape');
		await page.reload();
		await page.locator('.hero').waitFor();
		assert.equal(
			await page.evaluate(() => JSON.parse(localStorage.getItem('tfl-scenes-v1')).global.mediaId),
			'builtin-embers'
		);
		await page.close();
	}
	const reduced = await open({
		quality: 'Experience',
		reduced: true,
		scene: { preset: 'aurora', mediaId: 'builtin-aurora', shader: true, motion: 'full' }
	});
	assert.equal(await reduced.locator('.living-scene video,.living-scene canvas').count(), 0);
	await reduced.close();
	checks.push('media-quality-reduced-motion-focus-game-persistence');
	const mediaPage = await open({ quality: 'Experience' });
	await settings(mediaPage, 'General');
	await mediaPage.getByRole('button', { name: 'Manual', exact: true }).click();
	await settings(mediaPage, 'Fondos y escenas');
	await mediaPage.getByRole('button', { name: 'Importar vídeo', exact: true }).click();
	await mediaPage.waitForFunction(() =>
		window.__fixtureCommands.some((c) => c.cmd === 'import_scene_video')
	);
	await mediaPage.getByLabel('Fuente de vídeo').selectOption('scene-123456abcdef');
	await mediaPage
		.locator('.preview-caption')
		.getByRole('button', { name: 'Reproducir', exact: true })
		.click();
	await mediaPage.locator('.preview video').waitFor();
	assert.equal(await mediaPage.locator('.living-scene video').count(), 0);
	await mediaPage.locator('.pending-scene').getByRole('button', { name: 'Aplicar escena' }).click();
	await mediaPage.getByRole('button', { name: 'Eliminar vídeo abcdef' }).click();
	await mediaPage.getByRole('alert').filter({ hasText: 'en uso' }).waitFor();
	await mediaPage.getByLabel('Fuente de vídeo').selectOption('');
	await mediaPage.locator('.pending-scene').getByRole('button', { name: 'Aplicar escena' }).click();
	await mediaPage.getByRole('button', { name: 'Eliminar vídeo abcdef' }).click();
	await mediaPage.waitForFunction(() =>
		window.__fixtureCommands.some((c) => c.cmd === 'remove_scene_media')
	);
	await mediaPage.locator('[data-setting="scene-instance"] select').selectOption('horizons');
	await mediaPage.getByRole('button', { name: /Emerald Grotto/ }).click();
	await mediaPage.locator('.pending-scene').getByRole('button', { name: 'Aplicar escena' }).click();
	assert.equal(
		await mediaPage.evaluate(
			() => JSON.parse(localStorage.getItem('tfl-scenes-v1')).instances.horizons.preset
		),
		'cave'
	);
	await mediaPage.keyboard.press('Escape');
	// Las escenas ya no se ven en toda la app (antes cubrían el fondo
	// completo) — ahora viven en el hero de Home (la instancia más
	// reciente) y en la miniatura de cada tarjeta de la Biblioteca en
	// vista cuadrícula, que es lo único que no depende de cuál instancia
	// sea "la más reciente" en este fixture.
	await mediaPage.getByRole('button', { name: 'Biblioteca', exact: true }).click();
	await mediaPage.getByRole('button', { name: 'Cuadrícula', exact: true }).click();
	const horizontesCard = mediaPage.locator('.vr-library-row').filter({ hasText: 'Horizontes' });
	await horizontesCard.waitFor();
	assert.ok(
		(await horizontesCard.evaluate((el) => getComputedStyle(el).backgroundImage)).includes(
			'cave-thumb.webp'
		)
	);
	await shot(mediaPage, 'instance-scene-override');
	await mediaPage.close();
	checks.push('local-video-import-preview-in-use-delete-instance-override');
	const missing = await open({
		quality: 'Experience',
		scene: { preset: 'forest', mediaId: 'scene-012345abcdef', motion: 'full' }
	});
	await settings(missing, 'Fondos y escenas');
	await missing.locator('.preview .fallback').waitFor();
	assert.equal(await missing.locator('video').count(), 0);
	await shot(missing, 'missing-video-static-fallback');
	await missing.close();
	checks.push('missing-local-media-static-fallback');
	assert.deepEqual(errors, []);
	await writeFile(
		`${output}/results.json`,
		JSON.stringify({ fixture: true, checks, errors }, null, 2)
	);
	console.log(
		`PASS experience: ${checks.length} evidence entries; ${process.env.EXPERIENCE_SKIP_MATRIX ? 'functional assertions' : 'full matrix and functional assertions'}.`
	);
} finally {
	await browser.close();
}
