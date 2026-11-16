// Real Chromium Canvas rasterisation of production renderer source; no native WebView claim.
import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import ts from 'typescript';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const output = process.env.MOTION_OUTPUT || 'artifacts/beta3-motion';
await mkdir(output, { recursive: true });
const source = async (name) =>
	ts.transpileModule(
		await readFile(new URL(`../../src/lib/scenes/${name}.ts`, import.meta.url), 'utf8'),
		{ compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext } }
	).outputText;
const data = (code) => `data:text/javascript;base64,${Buffer.from(code).toString('base64')}`;
const landscape = data(await source('landscape'));
const motion = data(await source('motion'));
const renderer = data(
	(await source('landscapeRenderer'))
		.replaceAll("'./landscape'", JSON.stringify(landscape))
		.replaceAll("'./motion'", JSON.stringify(motion))
);
const browser = await chromium.launch({ headless: true });
const results = [];
try {
	for (const preset of ['midnight-forest', 'dithered-sunset', 'aurora-valley']) {
		const page = await browser.newPage({ viewport: { width: 1100, height: 700 } });
		await page.setContent(
			'<style>body{margin:0;background:#10162e}canvas{width:1152px;height:648px;image-rendering:pixelated}</style><canvas></canvas>'
		);
		const result = await page.evaluate(
			async ({ renderer, preset }) => {
				const { createLandscapeRenderer } = await import(renderer);
				const canvas = document.querySelector('canvas');
				const config = { particles: false, weather: 'clear', lighting: false, adaptTime: false };
				const draw = createLandscapeRenderer(canvas, preset, true, Array(7).fill(767.99));
				draw.draw(0, 1, config, 'day');
				window.before = canvas.toDataURL();
				const ctx = canvas.getContext('2d');
				const before = ctx.getImageData(0, 0, 384, 216).data;
				draw.draw(0, 1, config, 'day', 0.1);
				window.after = canvas.toDataURL();
				const after = ctx.getImageData(0, 0, 384, 216).data;
				let mismatch = 0;
				// In these rows terrain covers the entire viewport. The same texels must cross
				// the period boundary as a one-pixel translation to the RIGHT, without a gap.
				for (let y = 190; y < 216; y++)
					for (let x = 1; x < 384; x++)
						for (let c = 0; c < 4; c++) {
							if (after[(y * 384 + x) * 4 + c] !== before[(y * 384 + x - 1) * 4 + c]) mismatch++;
						}
				const phases = draw.phases;
				draw.dispose();
				return { mismatch, phases, released: canvas.width === 0 && canvas.height === 0 };
			},
			{ renderer, preset }
		);
		assert.equal(
			result.mismatch,
			0,
			`${preset}: spatial/temporal seam raster equals one texel of rightward travel`
		);
		assert.ok(result.phases.every((p) => p >= 0 && p < 1));
		assert.equal(result.released, true);
		for (const frame of ['before', 'after']) {
			const encoded = await page.evaluate((frame) => window[frame], frame);
			await writeFile(
				`${output}/${preset}-${frame}.png`,
				Buffer.from(encoded.split(',')[1], 'base64')
			);
		}
		results.push({ preset, ...result });
		await page.close();
	}
	await writeFile(
		`${output}/results.json`,
		JSON.stringify(
			{
				browser: await browser.version(),
				scenario:
					'Chromium Canvas source harness; boundary 767.99→0; static effects, not native launcher',
				results
			},
			null,
			2
		)
	);
	console.log(
		'PASS: three landscape raster seams, rightward one-texel boundary continuity and renderer disposal'
	);
} finally {
	await browser.close();
}
