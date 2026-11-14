// Actual frontend with isolated IPC. Timings and screenshots describe Chromium, not native Tauri.
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { installFixture } from './fixture.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const appVersion = JSON.parse(
	await readFile(new URL('../../package.json', import.meta.url), 'utf8')
).version;
const output = process.env.SIDEBAR_OUTPUT || 'artifacts/sidebar-refinement';
await mkdir(output, { recursive: true });
const browser = await chromium.launch({
	headless: true,
	executablePath: process.env.CHROMIUM_PATH || undefined
});
const results = [],
	errors = [];
async function open(o) {
	const p = await browser.newPage({
		viewport: { width: 1280, height: 720 },
		deviceScaleFactor: o.dpr,
		reducedMotion: o.quality === 'Lite' ? 'reduce' : 'no-preference'
	});
	p.setDefaultTimeout(15000);
	p.on('pageerror', (e) => errors.push(e.message));
	await p.addInitScript(installFixture, {
		premium: true,
		appVersion,
		quality: o.quality,
		theme: o.theme === 'light' ? 'light' : 'dark',
		language: o.language
	});
	await p.addInitScript((o) => {
		localStorage.setItem(
			'tfl-visual-refresh-v1',
			JSON.stringify({
				style: o.style,
				pixelType: o.font,
				customAccent: o.theme === 'light' ? '#5432b6' : '#92d9c2'
			})
		);
		localStorage.setItem('tfl-surface', o.theme === 'oled' ? 'oled' : 'obsidian');
		// A long list exercises scrolling and the fixed footer, including long accessible names.
		const invoke = window.__TAURI_INTERNALS__.invoke;
		window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
			const result = await invoke(cmd, args);
			if (cmd === 'get_instances')
				return [
					...result,
					...Array.from({ length: 40 }, (_, i) => ({
						...result[0],
						uuid: `long-${i}`,
						name: `Long survival instance ${i} with a deliberately descriptive name`,
						last_played: 0
					}))
				];
			return result;
		};
	}, o);
	await p.goto(process.env.VISUAL_URL || 'http://127.0.0.1:4184');
	await p.locator('.sidebar').waitFor();
	return p;
}
async function settle(p, collapsed) {
	await p.waitForFunction((c) => {
		const el = document.querySelector('.sidebar');
		return (
			el?.classList.contains('collapsed') === c &&
			Math.abs(el.getBoundingClientRect().width - (c ? 72 : 232)) < 0.5
		);
	}, collapsed);
}
async function geometry(p, o, collapsed, label) {
	await settle(p, collapsed);
	const m = await p.evaluate(() => {
		const rail = document.querySelector('.sidebar'),
			tab = document.querySelector('.sidebar-edge-toggle');
		const r = rail.getBoundingClientRect(),
			b = tab.getBoundingClientRect();
		const hit = [
			[4, 4],
			[b.width - 4, 4],
			[4, b.height - 4],
			[b.width - 4, b.height - 4],
			[b.width / 2, b.height / 2]
		].every(([x, y]) => tab.contains(document.elementFromPoint(b.x + x, b.y + y)));
		const footer = document.querySelector('.user-chip').getBoundingClientRect();
		const icons = [...document.querySelectorAll('.primary-nav .home-btn svg')].map((el) => {
			const a = el.getBoundingClientRect();
			return a.x + a.width / 2;
		});
		return {
			edge: b.x + b.width / 2,
			rail: r.right,
			center: b.y + b.height / 2,
			expected: r.y + r.height / 2,
			hit,
			width: b.width,
			height: b.height,
			footerFits: footer.bottom <= r.bottom + 1,
			overflow: getComputedStyle(rail).overflowX,
			documentFits: document.documentElement.scrollWidth <= innerWidth,
			icons,
			buttonBorder: getComputedStyle(document.querySelector('.primary-nav button')).borderWidth,
			listScroll:
				document.querySelector('.instance-list').scrollHeight >
				document.querySelector('.instance-list').clientHeight
		};
	});
	assert.ok(Math.abs(m.edge - m.rail) <= 1, `${label}: tab stays centered on the same rail edge`);
	assert.ok(Math.abs(m.center - m.expected) <= 1, `${label}: tab stays vertically centered`);
	assert.ok(
		m.width >= 28 && m.height >= 36 && m.hit,
		`${label}: entire tab hitbox is above the workspace`
	);
	assert.ok(
		m.footerFits && m.documentFits && m.listScroll,
		`${label}: scrollable list preserves footer and viewport`
	);
	assert.equal(m.overflow, 'hidden', `${label}: rail contents keep their clipping boundary`);
	if (collapsed)
		for (const center of m.icons)
			assert.ok(Math.abs(center - 36) < 1, `${label}: compact icons are centered`);
	if (o.style === 'pixel')
		assert.equal(m.buttonBorder, '1px', `${label}: arcade relief uses a thin frame`);
	results.push({ label, ...m });
}
async function toggle(p, collapsed) {
	const before = performance.now();
	await p.locator('.sidebar-edge-toggle').click();
	await settle(p, collapsed);
	return performance.now() - before;
}
try {
	for (const [variant, style, font] of [
		['modern', 'modern', 'readable'],
		['pixel-readable', 'pixel', 'readable'],
		['pixel-classic', 'pixel', 'classic']
	])
		for (const [ti, theme] of ['dark', 'light', 'oled'].entries())
			for (const [qi, quality] of ['Lite', 'Balanced', 'Experience'].entries()) {
				const o = {
					style,
					font,
					theme,
					quality,
					dpr: ((ti + qi) % 2) + 1,
					language: qi % 2 ? 'en' : 'es'
				};
				const p = await open(o),
					label = `${variant}-${theme}-${quality}`;
				const navigationControl = p.locator('.primary-nav .home-btn').nth(1);
				const restingBox = await navigationControl.boundingBox();
				await navigationControl.hover();
				const hoverShadow = await navigationControl.evaluate((n) => getComputedStyle(n).boxShadow);
				await p.mouse.down();
				const pressedShadow = await navigationControl.evaluate(
					(n) => getComputedStyle(n).boxShadow
				);
				assert.deepEqual(
					await navigationControl.boundingBox(),
					restingBox,
					'hover/pressed preserve layout geometry'
				);
				if (style === 'pixel')
					assert.notEqual(
						pressedShadow,
						hoverShadow,
						'Pixel press visibly sinks the navigation control'
					);
				await p.mouse.up();
				await p.locator('.primary-nav .home-btn').first().click();
				await geometry(p, o, false, `${label}-expanded`);
				await p.screenshot({ path: `${output}/${label}-expanded.png` });
				await toggle(p, true);
				await geometry(p, o, true, `${label}-compact`);
				await p.screenshot({ path: `${output}/${label}-compact.png` });
				// Repeated pointer + keyboard changes must write exactly the requested manual preference.
				for (let i = 0; i < 2; i++) {
					await toggle(p, false);
					await toggle(p, true);
				}
				const edge = p.locator('.sidebar-edge-toggle');
				await edge.focus();
				await p.keyboard.press('Space');
				await settle(p, false);
				const focus = await edge.evaluate((n) => ({
					style: getComputedStyle(n).outlineStyle,
					width: getComputedStyle(n).outlineWidth
				}));
				assert.deepEqual(focus, { style: 'solid', width: '2px' });
				const writes = () =>
					p.evaluate(
						() => window.__fixtureCommands.filter((c) => c.cmd === 'update_settings').length
					);
				const count = await writes();
				for (const [width, height] of [
					[800, 600],
					[1100, 700],
					[1280, 720],
					[1366, 768],
					[1920, 1080]
				]) {
					await p.setViewportSize({ width, height });
					await geometry(p, o, width < 960, `${label}-${width}`);
					if (width === 800) {
						assert.equal(await edge.getAttribute('aria-disabled'), 'true');
						await edge.focus();
						await p.keyboard.press('Space');
						assert.equal(
							await writes(),
							count,
							'automatic compact override never changes persisted preference'
						);
						await p.screenshot({ path: `${output}/${label}-800.png` });
					}
				}
				assert.equal(await writes(), count, 'resize alone never writes a manual preference');
				const latency = await toggle(p, true);
				await p.locator('.instance-item').first().focus();
				const tip = p.getByRole('tooltip');
				await tip.waitFor();
				assert.ok(
					await tip.evaluate((n) => n.parentElement === document.body),
					'tooltip escapes clipping with a body portal'
				);
				const tipBox = await tip.boundingBox();
				assert.ok(tipBox.x >= 8 && tipBox.x + tipBox.width <= 1912, JSON.stringify(tipBox));
				assert.ok(await p.locator('.instance-item').first().getAttribute('aria-describedby'));
				await p.keyboard.press('Escape');
				assert.equal(await p.getByRole('tooltip').count(), 0);
				await p
					.getByRole('button', { name: o.language === 'en' ? 'Settings' : 'Ajustes', exact: true })
					.click();
				assert.ok(
					await edge.evaluate((n) => !!n.closest('[inert]')),
					'modal blocks background collapse control'
				);
				await p.keyboard.press('Escape');
				results.push({ label, latencyMs: latency, dpr: o.dpr, language: o.language });
				await p.close();
			}
	assert.deepEqual(errors, []);
	await writeFile(`${output}/results.json`, JSON.stringify({ results, errors }, null, 2));
	console.log(
		`PASS: ${results.length} sidebar geometry/timing observations; repeated manual/automatic collapse, keyboard, long lists, overlays and portalled tooltips.`
	);
} finally {
	await browser.close();
}
