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
		language: 'Language',
		title: 'Settings',
		close: 'Close',
		tabs: {
			appearance: 'Style',
			general: 'General',
			accounts: 'Accounts',
			java: 'Java'
		},
		theme: {
			label: 'Theme',
			dark: 'Dark',
			light: 'Light',
			lightDisabledTitle: 'Light mode not compatible with OLED surface'
		},
		quality: {
			label: 'Quality profile'
		},
		ram: {
			label: 'Global memory (RAM)',
			detected: '— {{gb}} GB detected',
			min: 'Minimum (MB)',
			max: 'Maximum (MB)',
			save: 'Save'
		},
		storage: {
			label: 'Storage',
			clearCache: 'Clear temp cache',
			cacheCleared: 'Freed {{mb}} MB.',
			clearCacheFailed: 'Could not clear: {{error}}'
		},
		dialogs: {
			label: 'Native file dialogs',
			auto: 'Automatic',
			manual: 'Manual',
			hint: 'Automatic (recommended): uploading your own icon/wallpaper or adding mods from a file stays disabled — no need, everything installs itself. Manual: enables it, but in some cases it can close the launcher abruptly (native file dialog).'
		},
		updates: {
			label: 'Updates',
			autoEnabled: 'Auto-update: On',
			autoDisabled: 'Auto-update: Off',
			hint: 'With this on, the launcher checks for and downloads updates by itself on startup, in the background — you choose when to install with the notice that appears.',
			check: 'Check for updates',
			upToDate: "You're on the latest version.",
			available: 'Update available: v{{version}}',
			download: 'Download and install'
		},
		accent: {
			label: 'Accent color',
			orange: 'Orange',
			violet: 'Violet',
			teal: 'Teal',
			blue: 'Electric blue',
			rose: 'Plasma pink',
			lime: 'Lime',
			iris: 'Iris',
			jade: 'Jade',
			crimson: 'Crimson',
			cyan: 'Cyan',
			grass: 'Grass',
			plum: 'Plum'
		},
		surface: {
			label: 'Surface',
			incompatibleTitle: 'OLED surface not compatible with light mode',
			obsidian: 'Obsidian',
			midnight: 'Midnight',
			slate: 'Slate',
			oled: 'OLED'
		},
		ambience: {
			label: 'Ambient effect',
			aurora: 'Aurora',
			cosmic: 'Cosmic',
			minimal: 'Minimal',
			particles: 'Particles'
		},
		wallpaper: {
			label: 'Wallpaper',
			yourImage: 'Your image',
			change: 'Change your image',
			upload: 'Upload your image',
			uploadDisabledTitle: 'Enable Manual mode (below) to use this',
			autoModeDisabled: 'Disabled in Automatic mode — enable "Manual" in the section below.',
			items: {
				none: 'None',
				voidNight: 'Void night',
				nether: 'Nether',
				end: 'The End',
				deepOcean: 'Ocean',
				auroraGradient: 'Aurora',
				animatedAurora: 'Animated aurora',
				obsidianSolid: 'Obsidian',
				charcoal: 'Charcoal',
				savanna: 'Savanna',
				goldenSunset: 'Golden sunset',
				lakeNight: 'Night at the lake',
				neonArcade: 'Night arcade',
				redCanyon: 'Red canyon',
				enchantedValley: 'Enchanted valley',
				snowyPeak: 'Snowy peak',
				stoneBridge: 'Stone bridge',
				mistyFortress: 'Misty fortress',
				villageTower: 'Village tower',
				deepCave: 'Deep cave',
				sunsetCoast: 'Sunset coast',
				abstractBlocks: 'Abstract blocks'
			}
		},
		density: {
			label: 'Interface density',
			comfortable: 'Comfortable',
			compact: 'Compact'
		},
		cardStyle: {
			label: 'Card style',
			rich: 'Rich',
			minimal: 'Minimal'
		},
		font: {
			label: 'Fonts',
			system: 'System',
			hint: 'Applies instantly, no need to restart the launcher.'
		},
		accounts: {
			saved: 'Saved accounts',
			use: 'Use',
			active: 'Active',
			remove: 'Remove account',
			addMicrosoft: 'Add Microsoft account',
			microsoftEnter: 'and enter:',
			waitingConfirmation: 'Waiting for confirmation…',
			copyCode: 'Copy code',
			startMicrosoft: 'Sign in with Microsoft',
			addOffline: 'Add offline account'
		},
		java: {
			label: 'Java runtimes',
			hint: 'They install automatically the first time an instance needs them — no need to install Java by hand.',
			major: 'Java {{major}}',
			installed: 'Installed',
			notInstalled: 'Not installed'
		}
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
		partialLocalAdd: "Added {{added}} of {{total}} files — the rest didn't have the expected extension or couldn't be copied.",
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
	},
	instanceDetail: {
		changeIcon: 'Change instance icon',
		rename: 'Rename',
		duplicate: 'Duplicate',
		duplicateTitle: 'Duplicate instance',
		exportMrpack: 'Export as .mrpack',
		verifyIntegrity: 'Verify integrity',
		delete: 'Delete',
		deleteTitle: 'Delete instance',
		deleteMessage:
			'"{{name}}" and all its files (worlds, mods, configs) will be deleted. This can\'t be undone.',
		exported: 'Exported: {{path}}',
		integrityOk: 'Everything is fine — no installed file is missing.',
		integrityMissing: 'Missing {{count}} file{{plural}} installed by a modpack: {{list}}',
		neverPlayed: 'Never played',
		playedToday: 'Today',
		playedYesterday: 'Yesterday',
		playedDaysAgo: '{{days}} days ago',
		preparing: 'Preparing…',
		instanceRunning: '"{{name}}" is running',
		play: 'Play',
		modsNotSupportedVanilla: 'Not supported in Vanilla',
		loading: 'Loading…',
		modsInstalledCount: '{{count}} installed',
		folder: 'Folder',
		instanceFiles: 'Instance files',
		tabs: {
			details: 'Details',
			mods: 'Mods',
			shaders: 'Shaders',
			resourcePacks: 'Resource Packs',
			modpacks: 'Modpacks',
			screenshots: 'Screenshots'
		},
		ram: {
			sectionLabel: 'Memory (this instance)',
			hint: 'Empty uses the global value from Settings{{recommended}}.',
			recommendedSuffix: ' (recommended: {{mb}} MB)',
			globalPlaceholder: 'Global'
		}
	},
	instanceIconPicker: {
		title: 'Instance icon',
		uploadOwn: 'Upload my own image',
		autoModeNotice: 'Disabled in Automatic mode — enable "Manual" in Settings to use it.'
	},
	instanceLogWindow: {
		stopFailed: 'Could not stop: {{error}}',
		running: 'Running…',
		exited: 'Process finished{{codeSuffix}}',
		exitedCodeSuffix: ' (code {{code}})',
		stats: '· CPU {{cpu}}% · RAM {{ram}} MB',
		stop: 'Stop',
		closeNotice: 'Close notice',
		waitingOutput: 'Waiting for game output…',
		errors: {
			outOfMemory: 'Ran out of allocated RAM — try raising the maximum RAM in Settings.',
			unsupportedJavaVersion:
				'This Minecraft version needs a newer Java version than the one installed.',
			insufficientSystemRam:
				'You asked for more RAM than your computer has available — lower the maximum in Settings.',
			jniError: 'A conflict between mods, or a Java version incompatible with this instance.',
			mixinConflict: 'A mod (mixin) clashed with another — try removing the last mod you installed.',
			missingDependency:
				'A mod is missing a dependency it needs — check whether you installed everything it required.',
			duplicateMod: "You have the same mod installed twice — check the instance's mod list.",
			nativeCrash:
				'The JVM crashed natively (not a specific mod error) — could be outdated video drivers, or not enough real RAM on the computer. Java left an hs_err_pid*.log with details in the instance folder.'
		},
		crashSummary: 'The JVM crashed natively — report summary:\n{{summary}}',
		exitOom:
			"The operating system cut off the game due to lack of RAM (not just what's assigned to the game — the computer's actual memory). Close other programs or lower the maximum memory in Settings.",
		exitNullCode:
			'The game closed abruptly without warning — usually the operating system cutting off the process due to lack of RAM. Try raising the maximum memory in Settings.',
		exitOtherCode:
			'The game closed with code {{code}} without a recognizable error in the log — with several mods installed, this is usually a lack of RAM. Try raising the maximum memory in Settings.'
	},
	nativeDialogModePrompt: {
		title: 'How do you want to install your own content?',
		lead: 'This determines whether you can upload your own icon/wallpaper or add mods from a file by hand. You can change it anytime from Settings.',
		autoRecommended: 'Automatic (recommended)',
		autoSub: 'Everything installs itself — no file dialogs, no crash risk.',
		manualSub:
			'Enables uploading your own icon/wallpaper and mods from a file — in some cases it can close the launcher abruptly.',
		bannerSub: 'Automatic: no file dialogs. Manual: enables them, with crash risk.',
		dismissAriaLabel: 'Close without choosing yet',
		dismissTitle: 'Close without choosing — you will be asked again next time'
	},
	screenshotsPanel: {
		hint: "Screenshots for this instance (F2 in-game saves them here). The launcher doesn't take screenshots on its own, it only shows and manages them.",
		openFolder: 'Open folder',
		empty: 'No screenshots yet. Play and press F2 to save one.',
		copyImage: 'Copy image',
		delete: 'Delete'
	},
	tflSelection: {
		eyebrow: 'Featured content',
		subtitle: 'Discover curated packs and add them to a compatible instance, or create a new one on the spot.',
		allTab: 'All',
		empty: 'No modpacks in the selection yet.',
		createNewOption: '＋ Create new instance — {{mcVersion}} ({{loader}})',
		needsLoaderInstance: 'You need an instance with Fabric, Forge, NeoForge or Quilt to install this pack.',
		noMatchingInstance: "None of your instances match this pack's versions.",
		added: 'Added',
		add: 'Add',
		noVersionBuild: 'There is no build of this pack for that version',
		noInstanceBuild: 'There is no build of this pack for that instance'
	},
	modpacksPanel: {
		vanillaNotSupported: "Vanilla doesn't support modpacks — choose Fabric, Forge, NeoForge or Quilt when creating the instance.",
		hint: "Installing a modpack adds its mods and configuration to this instance — it doesn't replace what you already had. Removing it deletes exactly what it brought, nothing more.",
		installedCount: 'Installed ({{count}})',
		update: 'Update'
	},
	shadersPanel: {
		qualityLabel: 'Graphics quality (quick)',
		presetLow: 'Low',
		presetMedium: 'Medium',
		presetHigh: 'High'
	},
	home: {
		exploreTflSelection: 'Explore TFL Selection',
		title: 'Your Minecraft library,\nnicely organized.',
		subtitle: 'Create an instance to play, install content, and tweak each profile.',
		modsCardTitle: 'Mods and modpacks',
		modsCardHint: 'Per instance',
		javaCardTitle: 'Automatic Java',
		javaCardHint: 'No manual setup'
	},
	whatsNewTips: {
		welcomeTitle: 'Welcome to TFL Client',
		updatesTitle: "What's new in v{{version}}",
		tip1: 'Ctrl+K (⌘K on Mac) opens a palette to jump between instances or actions without touching the mouse.',
		tip2: '"TFL Selection" brings curated modpacks and whatever the team publishes — one click and they\'re ready.',
		tip3: 'Each instance has its own Mods, Shaders and Modpacks tab — nothing mixes between instances.',
		tip4: 'In Settings → Style you can change accent, surface, density, and even the animated background.',
		updatedToVersionWithNotes: 'It auto-updated to version {{version}}:',
		viewTechnicalDetail: 'View the technical details on GitHub',
		updatedToVersionNoNotes:
			'It auto-updated to version {{version}}. The full details of what changed are in the GitHub Release.',
		viewFullRelease: 'View the full Release',
		gotIt: 'Got it'
	},
	titleBar: {
		minimize: 'Minimize',
		maximize: 'Maximize',
		restore: 'Restore'
	},
	updateBadge: {
		available: 'Update available',
		installing: 'Installing…',
		clickToInstall: 'v{{version}} — click to install'
	},
	commandPalette: {
		action: 'Action',
		searchPlaceholder: 'Search instances or actions…',
		noResults: 'No results'
	},
	confirmDialog: {
		confirm: 'Confirm'
	},
	downloadProgressBar: {
		resolving: 'Resolving…',
		library: 'Downloading libraries',
		asset: 'Downloading assets',
		native: 'Downloading natives',
		client: 'Downloading client',
		verifying: 'Verifying',
		extracting: 'Extracting',
		processing: 'Processing',
		jre: 'Downloading Java',
		downloading: 'Downloading',
		generic: 'Working…'
	},
	modsPanel: {
		vanillaHint: "Vanilla doesn't support mods — pick Fabric, Forge, NeoForge or Quilt when creating the instance."
	},
	resourcePacksPanel: {
		hint: "Resource packs work on any instance, with or without mods — you don't need anything extra installed. Enable them from Minecraft's menu (Options → Resource Packs)."
	}
};
