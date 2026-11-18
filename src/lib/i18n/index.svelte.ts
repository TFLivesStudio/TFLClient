import { es, type Dictionary } from './es';
import { en } from './en';
import catalog from './catalog.v1.json';
import { appState } from '$lib/state/state.svelte';
import { patchSettings } from '$lib/api/settings';
import { invoke } from '$lib/api/invoke';
import type { AddonCatalog } from '$lib/addons/model';
import {
	canonicalLocale,
	formatLanguageMessage,
	phraseKey,
	validLanguageMessage,
	type LanguageOption,
	type LanguageMessage
} from './language';

export type Locale = string;
type DotPaths<T, Prefix extends string = ''> = T extends string
	? Prefix
	: {
			[K in keyof T & string]: DotPaths<T[K], `${Prefix}${Prefix extends '' ? '' : '.'}${K}`>;
		}[keyof T & string];
export type TranslationKey = DotPaths<Dictionary>;
const STORAGE_KEY = 'tfl-locale';
let legacy = 'es-419';
try {
	legacy =
		canonicalLocale(
			typeof localStorage === 'undefined' ? null : localStorage.getItem(STORAGE_KEY)
		) ?? 'es-419';
} catch {
	/* Existing profiles with unavailable browser storage still open. */
}

const builtins: LanguageOption[] = [
	{
		locale: 'es-419',
		name: 'Español (Latinoamérica)',
		flag: '/flags/latin-america.svg',
		direction: 'ltr',
		fallback: 'es-419',
		messages: {}
	},
	{
		locale: 'en-US',
		name: 'English (United States)',
		flag: '/flags/us.svg',
		direction: 'ltr',
		fallback: 'en-US',
		messages: {}
	}
];
export const languageState = $state({
	addons: [] as LanguageOption[],
	loaded: false,
	saving: false,
	error: '',
	retryLocale: null as Locale | null
});
let generation = 0;
let lastFallback: 'es-419' | 'en-US' = 'es-419';
export function languageOptions(): LanguageOption[] {
	return [...builtins, ...languageState.addons];
}
function requestedLocale() {
	return canonicalLocale(appState.settings?.locale) ?? legacy;
}
function activeLanguage(): LanguageOption {
	return (
		languageOptions().find((language) => language.locale === requestedLocale()) ??
		builtins.find((language) => language.locale === lastFallback)!
	);
}
export const i18nState = {
	get locale(): Locale {
		return activeLanguage().locale;
	}
};

export async function setLocale(locale: Locale): Promise<void> {
	const normalized = canonicalLocale(locale);
	if (!normalized || !languageOptions().some((language) => language.locale === normalized)) {
		languageState.retryLocale = null;
		languageState.error = 'El idioma ya no está disponible. Activa su addon y vuelve a intentarlo.';
		return;
	}
	if (languageState.saving) return;
	languageState.saving = true;
	languageState.error = '';
	languageState.retryLocale = null;
	try {
		await patchSettings({ locale: normalized });
		legacy = normalized;
	} catch {
		languageState.error = 'No se pudo guardar el idioma. Se conserva la elección anterior.';
		languageState.retryLocale = normalized;
	} finally {
		languageState.saving = false;
	}
}

/** One lifecycle, started with Addons. Settings remain the sole durable preference store. */
export function startLanguageLifecycle() {
	const stop = $effect.root(() => {
		$effect(() => {
			const chosen = activeLanguage();
			if (typeof document !== 'undefined') {
				document.documentElement.lang = chosen.locale;
				document.documentElement.dir = chosen.direction;
			}
		});
		$effect(() => {
			const settings = appState.settings;
			const requested = requestedLocale();
			const loaded = languageState.loaded;
			const available = languageOptions().some((language) => language.locale === requested);
			if (!settings || languageState.saving || languageState.error) return;
			if (!settings.locale && available) {
				void setLocale(requested);
				return;
			}
			if (loaded && !available) {
				void setLocale(lastFallback);
			}
		});
	});
	return () => {
		generation++;
		stop();
	};
}

/** Catalog snapshots are validated in Rust, and checked again before entering translation lookup. */
export function syncAddonLanguages(addons: AddonCatalog) {
	const previous = activeLanguage();
	if (previous.packageId) lastFallback = previous.fallback;
	const current = ++generation;
	const options: LanguageOption[] = [];
	const collisions = new Set<string>();
	const sources = catalog.messages as Record<string, { source: string }>;
	const packages = addons.safeMode
		? []
		: addons.addons
				.filter((addon) => addon.enabled)
				.sort((a, b) => a.manifest.id.localeCompare(b.manifest.id, 'en'));
	for (const addon of packages)
		for (const language of addon.manifest.languages ?? []) {
			if (
				language.schemaVersion !== 1 ||
				canonicalLocale(language.locale) !== language.locale ||
				builtins.some((builtin) => builtin.locale === language.locale) ||
				!['ltr', 'rtl'].includes(language.direction) ||
				!['es-419', 'en-US'].includes(language.fallback)
			)
				continue;
			if (options.some((option) => option.locale === language.locale))
				collisions.add(language.locale);
			const messages: Record<string, LanguageMessage> = {};
			for (const [key, message] of Object.entries(language.messages))
				if (Object.hasOwn(sources, key) && validLanguageMessage(message, sources[key].source))
					messages[key] = message;
			options.push({ ...language, messages, flag: '', packageId: addon.manifest.id });
		}
	languageState.addons = options
		.filter((language) => !collisions.has(language.locale))
		.sort((a, b) => a.locale.localeCompare(b.locale, 'en'));
	languageState.loaded = true;
	if (collisions.size)
		languageState.error = `Locales de idioma en conflicto: ${[...collisions].sort().join(', ')}.`;
	for (const option of languageState.addons) {
		const definition = packages
			.find((addon) => addon.manifest.id === option.packageId)
			?.manifest.languages?.find((language) => language.locale === option.locale);
		if (!definition) continue;
		void invoke<string>('addons_resource', { id: option.packageId, path: definition.flag })
			.then((flag) => {
				if (current !== generation || !/^data:image\/png;base64,[A-Za-z0-9+/=]+$/.test(flag))
					return;
				languageState.addons = languageState.addons.map((language) =>
					language.locale === option.locale ? { ...language, flag } : language
				);
			})
			.catch(() => {
				/* A readable locale/name remains if the flag is unavailable. */
			});
	}
}

function lookup(dict: Dictionary, key: string): string | undefined {
	let node: unknown = dict;
	for (const part of key.split('.')) {
		if (typeof node !== 'object' || node === null || !Object.hasOwn(node, part)) return undefined;
		node = (node as Record<string, unknown>)[part];
	}
	return typeof node === 'string' ? node : undefined;
}
export function translateMessage(
	key: string,
	spanish: string,
	english: string,
	vars?: Record<string, string | number>
): string {
	const language = activeLanguage();
	const source = (catalog.messages as Record<string, { source: string }>)[key];
	const contribution = language.messages[key];
	const value =
		contribution && source?.source === english && validLanguageMessage(contribution, english)
			? contribution
			: language.fallback === 'en-US'
				? english
				: spanish;
	return formatLanguageMessage(value, language.locale, vars);
}
export function localizePhrase(spanish: string, english: string): string {
	return translateMessage(phraseKey(english), spanish, english);
}
export function t(key: TranslationKey, vars?: Record<string, string | number>): string {
	return translateMessage(
		key,
		lookup(es, key) ?? key,
		lookup(en, key) ?? lookup(es, key) ?? key,
		vars
	);
}
