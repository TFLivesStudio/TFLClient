import { i18nState } from './index.svelte';
export const tr = (es: string, en: string) => (i18nState.locale === 'en' ? en : es);
