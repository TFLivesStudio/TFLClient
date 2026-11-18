import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir, mkdtemp, rm } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { pathToFileURL } from 'node:url';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';
import {
	canonicalLocale,
	phraseKey,
	validLanguageMessage,
	formatLanguageMessage,
	placeholders
} from '../../src/lib/i18n/language.ts';

const root = resolve(import.meta.dirname, '../..');
const contract = JSON.parse(await readFile(resolve(root, 'src/lib/i18n/catalog.v1.json'), 'utf8'));
const example = JSON.parse(
	await readFile(resolve(root, 'examples/addons/portuguese-br/manifest.json'), 'utf8')
);
test('locale migration is canonical and language IDs remain explicit', () => {
	assert.equal(canonicalLocale('es'), 'es-419');
	assert.equal(canonicalLocale('en'), 'en-US');
	assert.equal(canonicalLocale('pt-br'), 'pt-BR');
	for (const invalid of [null, 'LATAM', 'es_419', '<script>', 'a'.repeat(70)])
		assert.equal(canonicalLocale(invalid), null);
	assert.notEqual(phraseKey('Copy'), phraseKey('Copied'));
	assert.equal(phraseKey('Copy'), phraseKey('Copy'));
});
test('language-only .tfl example uses versioned messages and safe placeholders', async () => {
	assert.equal(example.schemaVersion, 1);
	assert.equal(example.themes.length + example.scenes.length, 0);
	assert.deepEqual(example.permissions, []);
	for (const language of example.languages) {
		assert.equal(canonicalLocale(language.locale), language.locale);
		assert.ok(language.flag.startsWith('assets/'));
		for (const [key, message] of Object.entries(language.messages)) {
			assert.ok(Object.hasOwn(contract.messages, key), key);
			assert.ok(validLanguageMessage(message, contract.messages[key].source), key);
		}
	}
	const archive = await readFile(resolve(root, 'examples/addons/portuguese-br.tfl'));
	assert.equal(archive.readUInt32LE(0), 0x04034b50);
	assert.equal(archive.readUInt16LE(archive.length - 22 + 8), 2);
});
test('plural categories use the selected locale and never drop missing placeholders', () => {
	const message = { one: '{{count}} idioma disponível', other: '{{count}} idiomas disponíveis' };
	assert.equal(formatLanguageMessage(message, 'pt-BR', { count: 1 }), '1 idioma disponível');
	assert.equal(formatLanguageMessage(message, 'pt-BR', { count: 2 }), '2 idiomas disponíveis');
	assert.equal(formatLanguageMessage('Hello {{name}}', 'en-US'), 'Hello {{name}}');
	assert.deepEqual(placeholders('{{name}} {{count}} {{name}}'), ['count', 'name']);
});
test('translation data rejects HTML, scripts, CSS, unknown forms and placeholder changes', () => {
	for (const value of [
		'<img src=x onerror=alert(1)>',
		'javascript:alert(1)',
		'url(file:///private)',
		'expression(x)',
		'${IPC()}',
		'Hello\u202e',
		'Hello {{extra}}',
		'Hello {{name}',
		'',
		{ one: 'one' },
		{ other: '{{name}}', execute: 'bad' }
	])
		assert.equal(validLanguageMessage(value, 'Hello {{name}}'), false, JSON.stringify(value));
	assert.equal(validLanguageMessage('Olá {{name}}', 'Hello {{name}}'), true);
	assert.equal(
		validLanguageMessage({ one: '{{name}}', other: '{{name}}' }, 'Hello {{name}}'),
		true
	);
});

// Exercise the real compiled Svelte i18n state with isolated settings/IPC boundaries.
// These are unit tests, not native launcher or external-client evidence.
async function runtime() {
	const parent = resolve(root, 'artifacts/unit-i18n');
	await mkdir(parent, { recursive: true });
	const temp = await mkdtemp(resolve(parent, 'fixture-'));
	const stateUrl = pathToFileURL(resolve(temp, 'state.mjs')).href;
	const apiUrl = pathToFileURL(resolve(temp, 'api.mjs')).href;
	const invokeUrl = pathToFileURL(resolve(temp, 'invoke.mjs')).href;
	await writeFile(
		resolve(temp, 'state.mjs'),
		"export const appState = { settings: { locale: 'es-419' } };\n"
	);
	await writeFile(
		resolve(temp, 'api.mjs'),
		`import {appState} from ${JSON.stringify(stateUrl)}; export const fixture = { fail:false,writes:[] }; export async function patchSettings(patch) { if(fixture.fail) throw Error('write denied'); fixture.writes.push(patch); appState.settings = {...appState.settings,...patch}; return appState.settings; }`
	);
	await writeFile(
		resolve(temp, 'invoke.mjs'),
		"export async function invoke(){ return 'data:image/png;base64,AAAA'; }"
	);
	const source = await readFile(resolve(root, 'src/lib/i18n/index.svelte.ts'), 'utf8');
	let js = ts.transpileModule(source, {
		compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ESNext }
	}).outputText;
	const mappings = {
		'./es': pathToFileURL(resolve(root, 'src/lib/i18n/es.ts')).href,
		'./en': pathToFileURL(resolve(root, 'src/lib/i18n/en.ts')).href,
		'./language': pathToFileURL(resolve(root, 'src/lib/i18n/language.ts')).href,
		'$lib/state/state.svelte': stateUrl,
		'$lib/api/settings': apiUrl,
		'$lib/api/invoke': invokeUrl
	};
	for (const [from, to] of Object.entries(mappings))
		js = js.replaceAll(`'${from}'`, JSON.stringify(to)).replaceAll(`"${from}"`, JSON.stringify(to));
	js = js.replace(
		/from ['"]\.\/catalog\.v1\.json['"];/,
		`from ${JSON.stringify(pathToFileURL(resolve(root, 'src/lib/i18n/catalog.v1.json')).href)} with { type: 'json' };`
	);
	await writeFile(
		resolve(temp, 'runtime.mjs'),
		compileModule(js, { filename: resolve(temp, 'runtime.svelte.js'), generate: 'client' }).js.code
	);
	const module = await import(pathToFileURL(resolve(temp, 'runtime.mjs')).href);
	const { fixture } = await import(apiUrl);
	const { appState } = await import(stateUrl);
	return {
		module,
		fixture,
		appState,
		async close() {
			assert.equal(dirname(temp), parent);
			await rm(temp, { recursive: true, force: true });
		}
	};
}
test('real runtime switches live, updates, falls back on removal and retains last valid language on write failure', async () => {
	const context = await runtime();
	try {
		const { module: i18n, fixture } = context;
		const manifest = structuredClone(example);
		manifest.languages[0].messages[phraseKey('Copied')] = 'Copiado pelo exemplo';
		manifest.languages[0].messages['selectionLibrary.install'] = 'Instalar {{name}}';
		const snapshot = { safeMode: false, addons: [{ manifest, enabled: true }] };
		i18n.syncAddonLanguages(snapshot);
		await i18n.setLocale('pt-BR');
		assert.equal(i18n.i18nState.locale, 'pt-BR');
		assert.equal(i18n.t('common.back'), 'Voltar');
		assert.equal(i18n.t('ui.retry'), 'Retry');
		assert.equal(i18n.t('selectionLibrary.install', { name: 'TFLClient' }), 'Instalar TFLClient');
		assert.equal(i18n.localizePhrase('Copiado', 'Copied'), 'Copiado pelo exemplo');
		assert.equal(
			i18n.translateMessage(
				'language.available',
				'{{count}} idiomas',
				'{{count}} languages available',
				{ count: 2 }
			),
			'2 idiomas disponíveis'
		);
		manifest.languages[0].messages['common.back'] = 'Retornar';
		i18n.syncAddonLanguages(snapshot);
		assert.equal(i18n.t('common.back'), 'Retornar');
		i18n.syncAddonLanguages({ ...snapshot, safeMode: true });
		assert.equal(i18n.i18nState.locale, 'en-US');
		assert.equal(i18n.t('common.back'), 'Back');
		await i18n.setLocale('en-US');
		fixture.fail = true;
		await i18n.setLocale('es-419');
		assert.equal(i18n.i18nState.locale, 'en-US');
		assert.ok(i18n.languageState.error);
		assert.equal(i18n.languageState.saving, false);
		assert.deepEqual(fixture.writes, [{ locale: 'pt-BR' }, { locale: 'en-US' }]);
	} finally {
		await context.close();
	}
});
test('runtime rejects conflicting package locales and malicious messages deterministically', async () => {
	const context = await runtime();
	try {
		const { module: i18n } = context;
		const malicious = structuredClone(example);
		malicious.languages[0].messages['common.back'] = '<script>IPC()</script>';
		i18n.syncAddonLanguages({ safeMode: false, addons: [{ manifest: malicious, enabled: true }] });
		await i18n.setLocale('pt-BR');
		assert.equal(i18n.t('common.back'), 'Back');
		const other = structuredClone(example);
		other.id = 'test.other-language';
		for (const addons of [
			[
				{ manifest: example, enabled: true },
				{ manifest: other, enabled: true }
			],
			[
				{ manifest: other, enabled: true },
				{ manifest: example, enabled: true }
			]
		]) {
			i18n.syncAddonLanguages({ safeMode: false, addons });
			assert.ok(!i18n.languageOptions().some((language) => language.locale === 'pt-BR'));
			assert.equal(i18n.i18nState.locale, 'en-US');
		}
	} finally {
		await context.close();
	}
});
