import { es, type Dictionary } from './es';
import { en } from './en';

export type Locale = 'es' | 'en';

const DICTIONARIES: Record<Locale, Dictionary> = { es, en };

// Une "onboarding" + "tagline" en "onboarding.tagline", recorriendo el
// diccionario entero — así `t()` autocompleta y tipa cada key contra la
// forma real de `es.ts`, no hay forma de pasarle una key que no exista.
type DotPaths<T, Prefix extends string = ''> = T extends string
	? Prefix
	: { [K in keyof T & string]: DotPaths<T[K], `${Prefix}${Prefix extends '' ? '' : '.'}${K}`> }[keyof T &
			string];

export type TranslationKey = DotPaths<Dictionary>;

const STORAGE_KEY = 'tfl-locale';

export const i18nState = $state<{ locale: Locale }>({
	locale: (typeof localStorage !== 'undefined' ? (localStorage.getItem(STORAGE_KEY) as Locale | null) : null) ?? 'es'
});

export function setLocale(locale: Locale) {
	i18nState.locale = locale;
	if (typeof localStorage !== 'undefined') localStorage.setItem(STORAGE_KEY, locale);
}

function lookup(dict: Dictionary, key: string): string | undefined {
	let node: unknown = dict;
	for (const part of key.split('.')) {
		if (typeof node !== 'object' || node === null) return undefined;
		node = (node as Record<string, unknown>)[part];
	}
	return typeof node === 'string' ? node : undefined;
}

/** Traduce `key` al idioma activo, interpolando `{{var}}` si se pasan `vars`.
 *  Llamarla dentro de un template Svelte la hace reactiva sola — lee
 *  `i18nState.locale` ($state), así que Svelte reevalúa la expresión
 *  entera cuando cambia el idioma, igual que con cualquier otro $state. */
export function t(key: TranslationKey, vars?: Record<string, string | number>): string {
	const raw = lookup(DICTIONARIES[i18nState.locale], key) ?? lookup(es, key) ?? key;
	if (!vars) return raw;
	return raw.replace(/\{\{(\w+)\}\}/g, (_, name) => String(vars[name] ?? ''));
}
