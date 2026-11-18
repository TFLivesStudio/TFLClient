export type PluralMessage = Partial<Record<Intl.LDMLPluralRule, string>> & { other: string };
export type LanguageMessage = string | PluralMessage;
export interface AddonLanguage {
	schemaVersion: 1;
	locale: string;
	name: string;
	flag: string;
	direction: 'ltr' | 'rtl';
	fallback: 'es-419' | 'en-US';
	messages: Record<string, LanguageMessage>;
}
export interface LanguageOption {
	locale: string;
	name: string;
	flag: string;
	direction: 'ltr' | 'rtl';
	fallback: 'es-419' | 'en-US';
	packageId?: string;
	messages: Record<string, LanguageMessage>;
}
export const BUILTIN_LOCALES = ['es-419', 'en-US'] as const;
export function canonicalLocale(value: unknown): string | null {
	if (value === 'es') return 'es-419';
	if (value === 'en') return 'en-US';
	if (typeof value !== 'string' || value.length > 40) return null;
	try {
		const [locale] = Intl.getCanonicalLocales(value);
		return locale &&
			/^[a-z]{2,3}(?:-[A-Z][a-z]{3})?(?:-(?:[A-Z]{2}|[0-9]{3}))?(?:-[a-z0-9]{5,8})*$/.test(locale)
			? locale
			: null;
	} catch {
		return null;
	}
}
/** Stable compatibility IDs for the pre-existing two-string helpers. The source
 * catalog additionally checks the original English string against hash collisions. */
export function phraseKey(source: string): string {
	let hash = 0x811c9dc5;
	for (let i = 0; i < source.length; i++) hash = Math.imul(hash ^ source.charCodeAt(i), 0x01000193);
	return `phrase.${(hash >>> 0).toString(16).padStart(8, '0')}`;
}
export function placeholders(text: string): string[] {
	return [
		...new Set([...text.matchAll(/\{\{([A-Za-z_][A-Za-z0-9_]*)\}\}/g)].map((m) => m[1]))
	].sort();
}
export function validLanguageMessage(message: unknown, source: string): message is LanguageMessage {
	const forms =
		typeof message === 'string'
			? [message]
			: message &&
				  typeof message === 'object' &&
				  !Array.isArray(message) &&
				  typeof (message as Record<string, unknown>).other === 'string' &&
				  Object.keys(message).every((key) =>
						['zero', 'one', 'two', 'few', 'many', 'other'].includes(key)
				  )
				? Object.values(message)
				: [];
	const expected = JSON.stringify(placeholders(source));
	return (
		forms.length > 0 &&
		forms.every(
			(form) =>
				typeof form === 'string' &&
				form.length > 0 &&
				form.length <= 4096 &&
				// Control characters are intentionally rejected in untrusted language data.
				// eslint-disable-next-line no-control-regex
				!/[<>\u0000-\u0008\u000b\u000c\u000e-\u001f\u202a-\u202e\u2066-\u2069]/.test(form) &&
				!/(javascript\s*:|expression\s*\(|url\s*\(|\$\{)/i.test(form) &&
				JSON.stringify(placeholders(form)) === expected &&
				!form.replace(/\{\{[A-Za-z_][A-Za-z0-9_]*\}\}/g, '').match(/\{\{|\}\}/)
		)
	);
}
export function formatLanguageMessage(
	message: LanguageMessage,
	locale: string,
	vars?: Record<string, string | number>
): string {
	let text: string;
	if (typeof message === 'string') text = message;
	else {
		const count = Number(vars?.count);
		const category = Number.isFinite(count) ? new Intl.PluralRules(locale).select(count) : 'other';
		text = message[category] ?? message.other;
	}
	return text.replace(/\{\{([A-Za-z_][A-Za-z0-9_]*)\}\}/g, (token, name) =>
		vars?.[name] === undefined ? token : String(vars[name])
	);
}
