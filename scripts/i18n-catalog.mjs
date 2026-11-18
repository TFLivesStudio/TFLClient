// Regenerate the versioned translation contract after adding/removing UI messages.
// Source English strings identify legacy phrase helpers without installing runtime code.
import { readdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, relative } from 'node:path';
import ts from 'typescript';
import { parse } from 'svelte/compiler';
import { format, resolveConfig } from 'prettier';
import { phraseKey, placeholders } from '../src/lib/i18n/language.ts';

const root = resolve(import.meta.dirname, '..');
const messages = new Map();
function add(key, source) {
	if (typeof source !== 'string' || !source.length) return;
	if (messages.has(key) && messages.get(key).source !== source)
		throw new Error(`Translation ID collision: ${key}`);
	messages.set(key, { source, placeholders: placeholders(source) });
}
async function dictionary(path, names) {
	const raw = await readFile(resolve(root, path), 'utf8');
	const withoutImports = raw
		.replace(/^import[^;]+;\s*/gm, '')
		.replace(/\bexport\s+(?=(?:const|function|type|interface))/g, '');
	const js = ts.transpileModule(`${withoutImports}\nexport { ${names.join(',')} };`, {
		compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext }
	}).outputText;
	return import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
}
function flatten(value, prefix = '') {
	for (const [key, entry] of Object.entries(value)) {
		const id = prefix ? `${prefix}.${key}` : key;
		if (typeof entry === 'string') add(id, entry);
		else flatten(entry, id);
	}
}
const { en } = await dictionary('src/lib/i18n/en.ts', ['en']);
flatten(en);
const { words } = await dictionary('src/lib/i18n/workbench.ts', ['words']);
for (const [key, pair] of Object.entries(words)) add(`workbench.${key}`, pair[1]);
const refresh = await dictionary('src/lib/i18n/refresh.ts', ['es', 'en']);
for (const [key, source] of Object.entries(refresh.en)) add(`refresh.${key}`, source);
add('language.available', '{{count}} languages available');

function staticString(node) {
	if (!node) return null;
	if (ts.isStringLiteralLike(node)) return node.text;
	if (node.type === 'Literal' && typeof node.value === 'string') return node.value;
	if (node.type === 'TemplateLiteral' && !node.expressions?.length)
		return node.quasis.map((q) => q.value.cooked).join('');
	return null;
}
function sources(node) {
	const value = staticString(node);
	if (value !== null) return [value];
	if (node && ts.isConditionalExpression(node))
		return [...sources(node.whenTrue), ...sources(node.whenFalse)];
	if (node?.type === 'ConditionalExpression')
		return [...sources(node.consequent), ...sources(node.alternate)];
	return [];
}
function walkSvelte(node) {
	if (!node || typeof node !== 'object') return;
	if (node.type === 'CallExpression' && ['tr', 'authText'].includes(node.callee?.name)) {
		for (const source of sources(node.arguments?.[1])) add(phraseKey(source), source);
	}
	if (node.type === 'ArrayExpression' && node.elements?.length === 2) {
		const first = staticString(node.elements[0]),
			second = staticString(node.elements[1]);
		if (first && second) add(phraseKey(second), second);
	}
	for (const [key, value] of Object.entries(node)) {
		if (['parent', 'loc', 'start', 'end'].includes(key)) continue;
		if (Array.isArray(value)) for (const child of value) walkSvelte(child);
		else if (value && typeof value === 'object') walkSvelte(value);
	}
}
function walkTs(node) {
	if (
		ts.isCallExpression(node) &&
		ts.isIdentifier(node.expression) &&
		['tr', 'authText'].includes(node.expression.text)
	) {
		for (const source of sources(node.arguments[1])) add(phraseKey(source), source);
	}
	if (ts.isArrayLiteralExpression(node) && node.elements.length === 2) {
		const first = staticString(node.elements[0]),
			second = staticString(node.elements[1]);
		if (first && second) add(phraseKey(second), second);
	}
	ts.forEachChild(node, walkTs);
}
async function scan(path) {
	for (const entry of await readdir(path, { withFileTypes: true })) {
		const child = resolve(path, entry.name);
		if (entry.isDirectory()) await scan(child);
		else if (/\.(svelte|ts)$/.test(entry.name)) {
			const raw = await readFile(child, 'utf8');
			if (entry.name.endsWith('.svelte')) {
				try {
					walkSvelte(parse(raw, { modern: true }));
				} catch (error) {
					throw new Error(`Cannot inventory ${relative(root, child)}: ${error.message}`, {
						cause: error
					});
				}
			} else walkTs(ts.createSourceFile(child, raw, ts.ScriptTarget.Latest, true));
		}
	}
}
await scan(resolve(root, 'src'));
const destination = resolve(root, 'src/lib/i18n/catalog.v1.json');
const body = await format(
	JSON.stringify({
		schemaVersion: 1,
		messages: Object.fromEntries([...messages].sort(([a], [b]) => a.localeCompare(b, 'en')))
	}),
	{ ...(await resolveConfig(destination)), filepath: destination }
);
if (process.argv.includes('--check')) {
	if ((await readFile(destination, 'utf8')) !== body)
		throw new Error('Translation catalog changed. Run node scripts/i18n-catalog.mjs.');
} else await writeFile(destination, body);
console.log(
	`i18n API 1: ${messages.size} messages${process.argv.includes('--check') ? ' checked' : ' inventoried'}.`
);
