#!/usr/bin/env node
// Fuente única de verdad para la versión de TFL Client.
//
// La versión vive en cuatro lugares que TIENEN que coincidir:
//   - package.json                      ("version")
//   - src-tauri/tauri.conf.json         ("version")  <- la que usa el updater
//   - src-tauri/Cargo.toml              ([package] version)
//   - Cargo.lock (raíz)                 (entrada del crate de la app)
//
// Uso:
//   bun run release:version 0.11.0            subir la versión en los cuatro
//   bun run release:version 0.11.0 --force    permitir una versión que no sube
//   bun run release:version 0.11.0 --dry-run  mostrar qué cambiaría sin escribir
//   bun run release:version --check           verificar que los cuatro coinciden
//   bun run release:version --check --tag v0.11.0
//                                             además, que coincidan con el tag
//
// Sin dependencias; corre con node o bun. Los archivos se editan con un
// reemplazo quirúrgico de la línea de versión (nada de JSON.stringify), así el
// diff de un bump son solo líneas de versión. NO toca src/lib/releaseNotes.ts
// (esas notas se escriben a mano en cada release); solo avisa si falta.

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SEMVER =
	/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*))*))?$/;

function fail(msg) {
	console.error(`\nERROR: ${msg}\n`);
	process.exit(1);
}

function usage() {
	console.log(`Uso:
  release-version.mjs <X.Y.Z[-pre]> [--force] [--dry-run]
  release-version.mjs --check [--tag vX.Y.Z]

Opciones:
  --check        Verifica que package.json, tauri.conf.json, Cargo.toml y
                 Cargo.lock tengan la misma versión (exit 1 si no).
  --tag vX.Y.Z   Con --check: exige además que coincida con el tag.
  --force        Permite una versión igual o menor a la actual.
  --dry-run      No escribe nada, solo muestra los cambios.
  --root <dir>   Raíz del repo (por defecto, la carpeta padre de scripts/).`);
}

// ---- argumentos -----------------------------------------------------------

function parseArgs(argv) {
	const opts = { check: false, force: false, dryRun: false, tag: null, root: null, version: null };
	for (let i = 0; i < argv.length; i++) {
		const a = argv[i];
		if (a === '--check') opts.check = true;
		else if (a === '--force') opts.force = true;
		else if (a === '--dry-run') opts.dryRun = true;
		else if (a === '--help' || a === '-h') {
			usage();
			process.exit(0);
		} else if (a === '--tag' || a === '--root') {
			const v = argv[++i];
			if (!v || v.startsWith('--')) fail(`${a} necesita un valor.`);
			opts[a.slice(2)] = v;
		} else if (a.startsWith('--tag=')) opts.tag = a.slice(6);
		else if (a.startsWith('--root=')) opts.root = a.slice(7);
		else if (a.startsWith('--')) fail(`Opción desconocida: ${a}`);
		else if (opts.version === null) opts.version = a;
		else fail(`Argumento de más: ${a}`);
	}
	return opts;
}

// ---- semver ---------------------------------------------------------------

function parseSemver(v) {
	const m = SEMVER.exec(v);
	if (!m) return null;
	return { major: +m[1], minor: +m[2], patch: +m[3], pre: m[4] ? m[4].split('.') : [] };
}

// Devuelve <0, 0, >0 según la precedencia de semver (los prerelease van antes
// que la versión estable: 1.0.0-rc.1 < 1.0.0).
function compareSemver(a, b) {
	for (const k of ['major', 'minor', 'patch']) {
		if (a[k] !== b[k]) return a[k] - b[k];
	}
	if (a.pre.length === 0 && b.pre.length === 0) return 0;
	if (a.pre.length === 0) return 1;
	if (b.pre.length === 0) return -1;
	for (let i = 0; i < Math.max(a.pre.length, b.pre.length); i++) {
		const x = a.pre[i];
		const y = b.pre[i];
		if (x === undefined) return -1;
		if (y === undefined) return 1;
		const xn = /^\d+$/.test(x);
		const yn = /^\d+$/.test(y);
		if (xn && yn) {
			if (+x !== +y) return +x - +y;
		} else if (xn !== yn) return xn ? -1 : 1;
		else if (x !== y) return x < y ? -1 : 1;
	}
	return 0;
}

// ---- lectura / reemplazo de la línea de versión ---------------------------

const escapeRe = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

// JSON: el "version" de nivel raíz es el que tiene la menor indentación.
function jsonTarget(text, label) {
	let current;
	try {
		current = JSON.parse(text).version;
	} catch (e) {
		fail(`${label}: no es JSON válido (${e.message}).`);
	}
	if (typeof current !== 'string') fail(`${label}: no tiene un campo "version" de nivel raíz.`);
	const re = new RegExp(`^([\\t ]*"version"[\\t ]*:[\\t ]*")${escapeRe(current)}(")`, 'gm');
	const hits = [...text.matchAll(re)];
	if (hits.length === 0) fail(`${label}: no pude ubicar la línea "version".`);
	const minIndent = Math.min(...hits.map((h) => /^[\t ]*/.exec(h[0])[0].length));
	const top = hits.filter((h) => /^[\t ]*/.exec(h[0])[0].length === minIndent);
	if (top.length !== 1) fail(`${label}: la línea "version" de nivel raíz es ambigua.`);
	const start = top[0].index;
	const prefix = top[0][1];
	const versionStart = start + prefix.length;
	return {
		current,
		replace: (next) =>
			text.slice(0, versionStart) + next + text.slice(versionStart + current.length)
	};
}

// Cargo.toml: la línea `version = "..."` dentro de la sección [package].
function cargoTomlTarget(text, label) {
	const header = /^\[package\][\t ]*\r?$/m.exec(text);
	if (!header) fail(`${label}: no encontré la sección [package].`);
	const bodyStart = header.index + header[0].length;
	const nextHeader = /^\[/m.exec(text.slice(bodyStart));
	const bodyEnd = nextHeader ? bodyStart + nextHeader.index : text.length;
	const body = text.slice(bodyStart, bodyEnd);
	const re = /^(version[\t ]*=[\t ]*")([^"\r\n]+)(")/m;
	const m = re.exec(body);
	if (!m) fail(`${label}: [package] no tiene una línea version = "..." (¿version.workspace?).`);
	const nameMatch = /^name[\t ]*=[\t ]*"([^"\r\n]+)"/m.exec(body);
	if (!nameMatch) fail(`${label}: [package] no tiene name.`);
	const versionStart = bodyStart + m.index + m[1].length;
	return {
		name: nameMatch[1],
		current: m[2],
		replace: (next) => text.slice(0, versionStart) + next + text.slice(versionStart + m[2].length)
	};
}

// Cargo.lock: solo la entrada [[package]] del crate de la app.
function cargoLockTarget(text, crateName, label) {
	const re = new RegExp(
		`(\\[\\[package\\]\\]\\r?\\nname = "${escapeRe(crateName)}"\\r?\\nversion = ")([^"\\r\\n]+)(")`,
		'g'
	);
	const hits = [...text.matchAll(re)];
	if (hits.length === 0)
		fail(`${label}: no encontré la entrada del crate "${crateName}" (¿Cargo.lock sin versionar?).`);
	if (hits.length > 1) fail(`${label}: hay varias entradas para "${crateName}".`);
	const m = hits[0];
	const versionStart = m.index + m[1].length;
	return {
		current: m[2],
		replace: (next) => text.slice(0, versionStart) + next + text.slice(versionStart + m[2].length)
	};
}

// ---- main -----------------------------------------------------------------

const opts = parseArgs(process.argv.slice(2));
const root = resolve(opts.root ?? join(dirname(fileURLToPath(import.meta.url)), '..'));

if (opts.check && opts.version) fail('--check no lleva versión; usá --check [--tag vX.Y.Z].');
if (!opts.check && !opts.version) {
	usage();
	process.exit(1);
}
if (!opts.check && opts.tag) fail('--tag solo se usa junto con --check.');

const FILES = {
	pkg: 'package.json',
	tauri: 'src-tauri/tauri.conf.json',
	cargo: 'src-tauri/Cargo.toml',
	lock: 'Cargo.lock'
};
const texts = {};
for (const [k, rel] of Object.entries(FILES)) {
	const p = join(root, rel);
	if (!existsSync(p)) fail(`Falta ${rel} (raíz: ${root}).`);
	texts[k] = readFileSync(p, 'utf8');
}

const targets = {
	pkg: jsonTarget(texts.pkg, FILES.pkg),
	tauri: jsonTarget(texts.tauri, FILES.tauri),
	cargo: cargoTomlTarget(texts.cargo, FILES.cargo)
};
targets.lock = cargoLockTarget(texts.lock, targets.cargo.name, FILES.lock);

const versions = Object.entries(FILES).map(([k, rel]) => ({
	key: k,
	file: rel === FILES.lock ? `${rel} (${targets.cargo.name})` : rel,
	version: targets[k].current
}));

function printVersions() {
	const w = Math.max(...versions.map((v) => v.file.length));
	for (const v of versions) console.log(`  ${v.file.padEnd(w)}  ${v.version}`);
}

if (opts.check) {
	let ok = true;
	const problems = [];
	const distinct = new Set(versions.map((v) => v.version));
	if (distinct.size !== 1) {
		ok = false;
		problems.push('Las versiones de los archivos NO coinciden entre sí.');
	}
	if (!parseSemver(versions[0].version)) {
		ok = false;
		problems.push(`"${versions[0].version}" no es semver válido (MAJOR.MINOR.PATCH[-pre]).`);
	}
	if (opts.tag !== null) {
		const m = /^v(.+)$/.exec(opts.tag);
		if (!m || !parseSemver(m[1])) {
			ok = false;
			problems.push(`El tag "${opts.tag}" no tiene la forma vMAJOR.MINOR.PATCH[-pre].`);
		} else if (distinct.size !== 1 || !distinct.has(m[1])) {
			ok = false;
			problems.push(
				`El tag "${opts.tag}" (versión ${m[1]}) no coincide con las versiones de los archivos.`
			);
		}
	}
	console.log(opts.tag !== null ? `Tag: ${opts.tag}` : 'Versiones:');
	printVersions();
	if (!ok) {
		console.error('');
		for (const p of problems) console.error(`ERROR: ${p}`);
		console.error(
			'\nPara alinear los archivos: bun run release:version <X.Y.Z> --force  (si el que está mal es el tag, borralo y creá el correcto).'
		);
		process.exit(1);
	}
	console.log(
		`\nOK: todo coincide en ${versions[0].version}${opts.tag ? ` y con el tag ${opts.tag}` : ''}.`
	);
	process.exit(0);
}

// ---- bump -----------------------------------------------------------------

const next = opts.version.replace(/^v/, '');
const nextParsed = parseSemver(next);
if (!nextParsed)
	fail(
		`"${opts.version}" no es una versión válida. Formato: MAJOR.MINOR.PATCH, opcional -prerelease (ej. 0.11.0, 1.0.0-rc.1). Sin metadata de build (+...).`
	);

const parsedCurrent = versions.map((v) => parseSemver(v.version)).filter(Boolean);
if (parsedCurrent.length !== versions.length)
	console.warn('AVISO: alguna versión actual no es semver válido; no se compara contra ella.');
const distinctNow = new Set(versions.map((v) => v.version));
if (distinctNow.size !== 1)
	console.warn('AVISO: las versiones actuales NO coinciden entre sí; este bump las deja iguales.');

const highest = parsedCurrent.length
	? versions
			.filter((v) => parseSemver(v.version))
			.reduce((a, b) => (compareSemver(parseSemver(b.version), parseSemver(a.version)) > 0 ? b : a))
	: null;
if (highest && compareSemver(nextParsed, parseSemver(highest.version)) <= 0 && !opts.force)
	fail(
		`La versión ${next} no es mayor que la actual (${highest.version}, en ${highest.file}). Usá --force si es a propósito.`
	);

console.log(`Versión actual:`);
printVersions();
console.log(`\nNueva versión: ${next}${opts.dryRun ? '  (dry-run, no se escribe nada)' : ''}`);

// Se calcula todo antes de escribir: o se actualizan los cuatro o ninguno.
const updated = {};
for (const k of Object.keys(FILES)) updated[k] = targets[k].replace(next);

// Re-validar sobre el texto nuevo antes de tocar el disco.
const re = {
	pkg: jsonTarget(updated.pkg, FILES.pkg).current,
	tauri: jsonTarget(updated.tauri, FILES.tauri).current,
	cargo: cargoTomlTarget(updated.cargo, FILES.cargo).current,
	lock: cargoLockTarget(updated.lock, targets.cargo.name, FILES.lock).current
};
for (const [k, v] of Object.entries(re)) {
	if (v !== next)
		fail(`Verificación interna falló en ${FILES[k]} (quedó ${v}). No se escribió nada.`);
}

if (!opts.dryRun) {
	for (const [k, rel] of Object.entries(FILES)) {
		if (updated[k] !== texts[k]) writeFileSync(join(root, rel), updated[k]);
	}
	console.log('Archivos actualizados:');
	for (const rel of Object.values(FILES)) console.log(`  ${rel}`);
}

// Recordatorio (solo lectura): las notas in-app se escriben a mano.
const notesPath = join(root, 'src/lib/releaseNotes.ts');
if (existsSync(notesPath)) {
	const notes = readFileSync(notesPath, 'utf8');
	const hasEntry = new RegExp(`^\\s*'${escapeRe(next)}'\\s*:`, 'm').test(notes);
	console.log(
		hasEntry
			? `\nsrc/lib/releaseNotes.ts ya tiene notas para ${next}.`
			: `\nRECORDATORIO: agregá a mano las notas de ${next} en src/lib/releaseNotes.ts (este script no lo toca).`
	);
}
console.log(
	`Siguiente: commit "chore: bump versión a ${next}", luego el tag v${next} (el workflow de release verifica que todo coincida).`
);
