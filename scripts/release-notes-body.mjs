#!/usr/bin/env bun
// Arma el cuerpo del GitHub Release a partir de src/lib/releaseNotes.ts.
//
// Esas notas ya existen: se escriben a mano para JUGADORES en cada versión
// (ver el comentario de ese archivo), y hasta ahora el Release de GitHub
// nunca las usaba — build.yml ponía un texto fijo genérico ("Ver el
// changelog del repo para el detalle de esta versión."). Este script lee
// la entrada real de la versión y la vuelca como el cuerpo del Release.
//
// release-version.mjs --check --tag ya exige que la entrada exista antes de
// llegar a este paso del workflow, así que un fallo acá solo debería pasar
// si algo saltó ese check.
//
// Uso:
//   bun run scripts/release-notes-body.mjs vX.Y.Z[-pre]

import { existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

function fail(msg) {
	console.error(`\nERROR: ${msg}\n`);
	process.exit(1);
}

const tag = process.argv[2];
if (!tag) fail('Uso: release-notes-body.mjs vX.Y.Z[-pre]');
const m = /^v(.+)$/.exec(tag);
if (!m) fail(`Tag inválido: "${tag}" (esperaba vX.Y.Z[-pre]).`);
const version = m[1];

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const notesPath = join(root, 'src/lib/releaseNotes.ts');
if (!existsSync(notesPath)) fail('Falta src/lib/releaseNotes.ts.');

const { RELEASE_NOTES } = await import(notesPath);
const entries = RELEASE_NOTES[version];
if (!entries || entries.length === 0) {
	fail(
		`No hay notas para "${version}" en src/lib/releaseNotes.ts ` +
			`(release-version.mjs --check --tag debería haber bloqueado esto antes de llegar acá).`
	);
}

const isPrerelease = version.includes('-');
const hasBetaLine = entries[0].toUpperCase().includes('BETA');
const lines = [];
if (isPrerelease && !hasBetaLine) {
	lines.push(
		'⚠️ **Versión BETA**: puede contener errores. Solo la reciben quienes activaron las versiones beta en Ajustes → Sistema → Actualizaciones.',
		''
	);
}
for (const entry of entries) lines.push(`- ${entry}`);

process.stdout.write(`${lines.join('\n')}\n`);
