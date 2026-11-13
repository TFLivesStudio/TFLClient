#!/usr/bin/env bun
// El Markdown revisado se publica sin transformaciones en GitHub y web.
// Las notas breves in-app se conservan como alternativa para el flujo existente.
// Guía: docs/maintainers/release-notes.md.
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const TAG = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z]+(?:[.-][0-9A-Za-z]+)*)?$/;

export function validateReleaseNotes(body, tag) {
	if (!TAG.test(tag)) throw new Error(`Tag inválido: "${tag}".`);
	if (!body.trim() || body.length > 100_000) {
		throw new Error(`${tag}: las notas están vacías o superan el tamaño permitido.`);
	}
	const opening = body.trim().split(/\r?\n\s*\r?\n/, 1)[0];
	if (
		/^[#>*-]|^\s*</.test(opening) ||
		!/^## (Novedades|Mejoras|Correcciones|Mantenimiento)$/m.test(body)
	) {
		throw new Error(`${tag}: empieza por un resumen y añade una sección de cambios.`);
	}
	if (/Ver el changelog del repo|\[completar\]/i.test(body) || /\b(?:TODO|TBD)\b/.test(body)) {
		throw new Error(`${tag}: las notas contienen un marcador pendiente.`);
	}
	return body.endsWith('\n') ? body : `${body}\n`;
}

export async function releaseNotesBody(
	tag,
	root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
) {
	if (!TAG.test(tag)) throw new Error(`Tag inválido: "${tag}" (esperaba vX.Y.Z[-pre]).`);
	const markdown = join(root, 'docs/releases', `${tag}.md`);
	if (existsSync(markdown)) return validateReleaseNotes(readFileSync(markdown, 'utf8'), tag);

	const notesPath = join(root, 'src/lib/releaseNotes.ts');
	if (!existsSync(notesPath)) throw new Error('Falta src/lib/releaseNotes.ts.');
	const { RELEASE_NOTES } = await import(pathToFileURL(notesPath).href);
	const entries = RELEASE_NOTES[tag.slice(1)];
	if (!Array.isArray(entries) || entries.length === 0) {
		throw new Error(`No hay notas para "${tag}" en docs/releases ni src/lib/releaseNotes.ts.`);
	}
	const changes = entries.filter((entry) => !/^Esta es una versión BETA/i.test(entry));
	if (!changes.length) throw new Error(`${tag}: falta describir los cambios.`);
	const beta = tag.includes('-');
	const body = [
		changes[0],
		'',
		`**Canal:** ${beta ? 'Beta / prueba' : 'Estable'}`,
		'',
		...(beta ? ['> Versión de prueba: puede contener errores.', ''] : []),
		'## Mejoras',
		'',
		...changes.map((entry) => `- ${entry}`),
		'',
		'[Web oficial](https://launcher.tflives.com) · [Ayuda](https://launcher.tflives.com/ayuda) · [Discord](https://discord.gg/tflives)',
		''
	].join('\n');
	return validateReleaseNotes(body, tag);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
	try {
		process.stdout.write(await releaseNotesBody(process.argv[2] ?? ''));
	} catch (error) {
		console.error(`ERROR: ${error.message}`);
		process.exitCode = 1;
	}
}
