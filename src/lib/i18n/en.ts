import type { Dictionary } from './es';

// Traducción al inglés — misma forma exacta que `es.ts` (chequeado por
// TypeScript vía `Dictionary`), así que si `es.ts` gana una key nueva y
// acá falta, el error de tipos avisa antes de que quede un string suelto
// sin traducir.
export const en: Dictionary = {
	common: {
		cancel: 'Cancel',
		back: 'Back',
		continue: 'Continue',
		copy: 'Copy',
		copied: 'Copied'
	},
	onboarding: {
		tagline: 'Sign in to start playing',
		microsoftTitle: 'Microsoft account',
		microsoftSub: 'Full access, multiplayer included',
		offlineTitle: 'Offline account',
		offlineSub: 'Singleplayer only',
		usernameLabel: 'Username',
		offlineHint: 'Offline accounts can only play singleplayer.',
		fetchingCode: 'Getting code...',
		goTo: 'Go to',
		enterCode: 'and enter the code:',
		waitingConfirmation: 'Waiting for confirmation...',
		copyCodeFailed: 'Could not copy. Select the code and copy it manually.'
	},
	settings: {
		language: 'Language'
	},
	sidebar: {
		offline: 'No connection — play what you already have installed',
		yourInstances: 'Your instances',
		createInstance: 'Create instance',
		search: 'Search…',
		noInstancesYet: "You haven't created any instance yet",
		createFirst: 'Create the first one',
		noResultsFor: 'No results for "{{query}}"',
		offlineAccountType: 'Offline — singleplayer only',
		openSettings: 'Settings',
		logout: 'Log out'
	},
	createInstance: {
		title: 'New instance',
		chooserSubtitle: 'How do you want to build it?',
		customTitle: 'Custom',
		customDesc: 'Create your own instance with the mods you choose.',
		modpackTitle: 'Modpack',
		modpackDesc: 'Download an instance already built from TFL Selection.',
		nameLabel: 'Name',
		namePlaceholder: 'My world',
		versionLabel: 'Minecraft version',
		loadingVersions: 'Loading versions…',
		loaderLabel: 'Loader',
		loaderHint:
			"The loader version (Fabric/Forge/NeoForge/Quilt) resolves itself — the latest/recommended one for this Minecraft version.",
		experimentalToggle: 'Show experimental versions (snapshots, pre-releases, release candidates)',
		create: 'Create'
	}
};
