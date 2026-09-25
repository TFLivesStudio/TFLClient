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
	},
	contentManager: {
		manageTab: 'Manage',
		downloadTab: 'Download',
		categories: 'Categories',
		selectNone: 'None',
		selectAll: 'Select all',
		manualModeRequired: 'Enable Manual mode in Settings to use this',
		addByFile: 'Add from file',
		checkUpdates: 'Check for updates',
		remove: 'Remove',
		autoModeDisablesAddByFile: 'Add from file is disabled in Automatic mode — enable "Manual" in Settings.',
		allUpToDate: 'Everything up to date — no updates.',
		noChangelogNotes: 'No changelog notes.',
		changelogLoadFailed: 'Could not load: {{error}}',
		partialLocalAdd: 'Added {{added}} of {{total}} files — the rest did not have the expected extension or could not be copied.',
		removeFailedPartial: 'Could not remove {{failed}} of {{total}}: {{names}}',
		duplicateOne: 'There is one mod',
		duplicateMany: 'There are {{count}} mods',
		duplicateWarning: 'installed twice (different versions of the same mod at once) — can cause crashes. Check: {{list}}',
		updateAll: 'Update all',
		notInstalledYet: "You haven't installed any {{kind}} here yet.",
		noneMatchCategories: 'No installed {{kind}} matches those categories.',
		select: 'Select',
		changelogButton: "What's new",
		searchPlaceholder: 'Search {{noun}} on Modrinth…',
		favorites: 'Favorites',
		results: 'Results',
		favorite: 'Favorite',
		viewVersions: 'View versions',
		alreadyInstalled: 'Already installed',
		noCompatibleVersions: 'No versions compatible with this instance.'
	}
};
