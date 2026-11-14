import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { releaseNotesBody, validateReleaseNotes } from '../../scripts/release-notes-body.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');

function fixture(t, entries) {
	const dir = mkdtempSync(join(tmpdir(), 'tfl-notes-test-'));
	t.after(() => rmSync(dir, { recursive: true, force: true }));
	mkdirSync(join(dir, 'src/lib'), { recursive: true });
	mkdirSync(join(dir, 'docs/releases'), { recursive: true });
	writeFileSync(
		join(dir, 'src/lib/releaseNotes.ts'),
		`export const RELEASE_NOTES = ${JSON.stringify(entries)};`
	);
	return dir;
}

test('all reconstructed release bodies publish exactly as the reviewed Markdown', async () => {
	const files = readdirSync(join(root, 'docs/releases')).filter((name) => /^v.+\.md$/.test(name));
	const provenance = JSON.parse(readFileSync(join(root, 'docs/releases/provenance.json'), 'utf8'));
	const tags = provenance.releases.map(({ tag }) => tag);
	for (const tag of tags) assert.ok(files.includes(`${tag}.md`), `missing ${tag}`);
	for (const file of files) {
		const tag = file.slice(0, -3);
		const expected = readFileSync(join(root, 'docs/releases', file), 'utf8');
		assert.equal(await releaseNotesBody(tag, root), expected);
		assert.ok(expected.split(/\n\s*\n/, 1)[0].length <= 260, `${tag}: card summary is too long`);
	}
});

test('full Markdown takes precedence over brief in-app notes', async (t) => {
	const dir = fixture(t, { '1.0.0': ['Nota breve.'] });
	const full = 'Resumen completo.\n\n## Correcciones\n\n- Se corrige el fallo documentado.\n';
	writeFileSync(join(dir, 'docs/releases/v1.0.0.md'), full);
	assert.equal(await releaseNotesBody('v1.0.0', dir), full);
});

test('missing Markdown preserves the existing in-app release flow and beta channel', async (t) => {
	const dir = fixture(t, {
		'1.0.0-beta.1': ['Esta es una versión BETA de prueba.', 'Se corrige el selector.']
	});
	const body = await releaseNotesBody('v1.0.0-beta.1', dir);
	assert.ok(body.startsWith('Se corrige el selector.\n\n'));
	assert.match(body, /\*\*Canal:\*\* Beta \/ prueba/);
	assert.equal((body.match(/Versión de prueba:/g) || []).length, 1);
});

test('invalid authored notes fail instead of silently falling back', async (t) => {
	const dir = fixture(t, { '1.0.0': ['Una corrección real.'] });
	writeFileSync(join(dir, 'docs/releases/v1.0.0.md'), 'Ver el changelog del repo para el detalle.');
	await assert.rejects(releaseNotesBody('v1.0.0', dir));
});

test('unknown releases and beta warnings without changes cannot publish generic notes', async (t) => {
	const dir = fixture(t, { '1.0.0-beta.1': ['Esta es una versión BETA de prueba.'] });
	await assert.rejects(releaseNotesBody('v2.0.0', dir), /No hay notas/);
	await assert.rejects(releaseNotesBody('v1.0.0-beta.1', dir), /falta describir/);
});

test('tag validation blocks path traversal and notes require a real opening summary', async (t) => {
	const dir = fixture(t, {});
	for (const tag of ['', '../v1.0.0', 'v1.0.0/../../README', '1.0.0']) {
		await assert.rejects(releaseNotesBody(tag, dir), /Tag inválido/);
	}
	assert.throws(() => validateReleaseNotes('# Título\n\n## Mejoras\n\n- Cambio.', 'v1.0.0'));
	assert.throws(() => validateReleaseNotes('Resumen.\n\n## Mejoras\n\n- TODO', 'v1.0.0'));
});
