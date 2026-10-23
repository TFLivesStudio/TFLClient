import { appState } from '$lib/state/state.svelte';
export type Section = 'home' | 'library' | 'discover' | 'activity';
export const navigation = $state<{ section: Section }>({ section: 'home' });
export function navigate(section: Section) {
	navigation.section = section;
	appState.selectedInstance = null;
}
