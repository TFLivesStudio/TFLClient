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
			performance: 'Performance',
			system: 'System',
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
		multiInstance: {
			label: 'Multiple instances warning',
			hint: "If you open an instance while another is already running, you'll get a heads-up first (same account can break multiplayer; different accounts only affect performance). You can turn this warning off here.",
			on: 'Warn: On',
			off: 'Warn: Off'
		},
		dialogs: {
			label: 'Native file dialogs',
			auto: 'Automatic',
			manual: 'Manual',
			hint: 'Automatic (recommended): uploading your own icon/wallpaper or adding mods from a file stays disabled — no need, everything installs itself. Manual: enables it, but in some cases it can close the launcher abruptly (native file dialog).'
		},
		updates: {
			betaEnabled: 'Beta versions: On',
			betaDisabled: 'Beta versions: Off',
			betaHint: 'Betas are early previews of what is coming: they arrive sooner but may have bugs. You can turn them off any time.',
			currentVersion: 'Installed version: {{version}}',
			betaWarningTitle: 'Turn on beta versions?',
			betaWarningMessage: 'Beta versions are early previews: they may contain bugs, crash, or leave your instances in an odd state. Back up any important worlds before trying them. You can turn them off any time, but the launcher does not go back to the stable version by itself: you stay on the beta until a newer stable version is released.',
			betaWarningConfirm: 'Turn on betas',
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
		logout: 'Log out',
		joinServer: 'Join server',
		contextOpen: 'Open'
	},
	joinServer: {
		favorites: 'Favorites',
		offline: 'No response',
		saveFavorite: 'Save to favorites',
		alreadyFavorite: 'Already in favorites',
		removeFavorite: 'Remove from favorites',
		favoriteNamePlaceholder: 'Name (optional)',
		saveFavoriteConfirm: 'Save',
		title: 'Join server',
		addressLabel: 'Server address',
		addressPlaceholder: 'example.com:25565',
		modeExisting: 'One of my instances',
		modeQuick: 'Quick Join',
		noInstances: "You don't have any client instance yet — use Quick Join.",
		quickHint: "Creates (or reuses) a Vanilla instance with that version and connects you straight in, skipping the Multiplayer menu.",
		quickPlayUnsupported: "This version is older than 1.20.2 and doesn't support direct connect — the game will open normally, connect manually from the Multiplayer menu.",
		crackedBlocked: 'Offline accounts can only play singleplayer — sign in with a Microsoft account to join a server.',
		join: 'Join',
		joining: 'Connecting…'
	},
	createInstance: {
		importTitle: 'Import from another launcher',
		importDesc: 'Bring your instances from Prism, MultiMC or CurseForge with their mods and worlds.',
		title: 'New instance',
		chooserSubtitle: 'How do you want to build it?',
		customTitle: 'Custom',
		customDesc: 'Create your own instance with the mods you choose.',
		modpackTitle: 'Modpack',
		modpackDesc: 'Download an instance already built from TFL Selection.',
		serverTitle: 'Server',
		serverDesc: 'Create your own Minecraft server to play with others.',
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
	createServer: {
		title: 'New server',
		serverTypeLabel: 'Server type',
		vanillaDesc: "Mojang's official server, no plugins.",
		paperDesc: 'The most used — fast and with plugin support.',
		purpurDesc: 'Based on Paper, with more customization options.',
		versionLabel: 'Minecraft version',
		noVersionsForType: 'No versions available for this server type yet.',
		loadingVersions: 'Loading versions…',
		create: 'Create server',
		creating: 'Downloading the server…'
	},
	worldBackups: {
		title: 'World backups',
		hint: 'They are created automatically before updating mods. You can make one by hand any time and restore it if something goes wrong (the last 10 are kept).',
		create: 'Back up now',
		created: 'Backup created.',
		noWorlds: 'This instance has no worlds to back up yet.',
		empty: 'No backups yet.',
		restore: 'Restore',
		delete: 'Delete backup',
		restored: 'Worlds restored. A backup of the previous state was saved first.',
		restoreTitle: 'Restore this backup?',
		restoreMessage: 'Your current worlds will be replaced with the ones from the {{date}} backup. A backup of how they are now is saved first, in case you change your mind.'
	},
	importInstance: {
		title: 'Import instance',
		subtitle: 'We found these instances in other launchers installed on your computer.',
		searching: 'Looking for instances…',
		none: 'We did not find Prism, MultiMC, PolyMC or CurseForge instances in the usual folders.',
		unknownVersion: 'unknown version',
		modCount: '{{count}} mods',
		nameLabel: 'Name in TFL Client',
		hint: 'A new instance is created with its mods, configs, worlds and packs. The original is not touched. The loader is installed at its latest version for that Minecraft version.',
		import: 'Import',
		importing: 'Importing…'
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
		noCompatibleVersions: 'No versions compatible with this instance.',
		cleanDuplicates: 'Remove duplicates',
		duplicatesCleaned: 'Removed {{count}} duplicate files.',
		enable: 'Enable',
		disable: 'Disable',
		disabledTag: 'disabled',
		rollbackButton: 'Undo last update ({{count}})',
		rollbackHint: 'Puts mods back to the version they had before "Update all".',
		rolledBack: '{{count}} mods went back to their previous version.'
	},
	instanceDetail: {
		playTimeHours: '{{h}} h {{m}} min played',
		playTimeMinutes: '{{m}} min played',
		playTimeTitle: 'Total time this instance has been open',
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
		play: 'Play',
		otherInstanceRunning: 'Also running right now: {{names}}',
		multiInstanceTitle: 'Another instance is already running',
		multiInstanceSameAccount:
			"It's the same account you're about to use here — multiplayer might break (can't be the same player in two places) and it adds CPU/RAM load. Open this one anyway?",
		multiInstanceDifferentAccount:
			"It's a different account, so no \"same player\" issue — but two instances at once means more CPU/RAM load, and only one can actually be played by one person at a time. Open this one anyway?",
		multiInstanceProceed: 'Open anyway',
		multiInstanceDontAskAgain: "Don't ask me again (editable in Settings)",
		modsNotSupportedVanilla: 'Not supported in Vanilla',
		loading: 'Loading…',
		modsInstalledCount: '{{count}} installed',
		folder: 'Folder',
		instanceFiles: 'Instance files',
		createShortcut: 'Create shortcut',
		createShortcutHint: 'Open it without opening the launcher first',
		shortcutCreated: 'Created on your Desktop',
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
		suspectsTitle: 'Possible culprit mods',
		suspectsHint: 'Based on the crash report. Disabling them does not delete them — you can re-enable them from Mods.',
		disableMod: 'Disable',
		suspectDisabled: 'Disabled',
		rollbackHint: 'If the game started failing right after updating mods:',
		rollbackMods: 'Go back to previous versions',
		rolledBackDone: 'Done: mods went back to the version they had before updating.',
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
	},
	pluginsPanel: {
		vanillaHint: "Vanilla doesn't support plugins — pick Paper or Purpur when creating the server."
	},
	serverConsole: {
		notRunning: 'The server is not running.',
		waitingOutput: 'Waiting for server output…',
		commandPlaceholder: 'Type a command and press Enter…',
		send: 'Send'
	},
	serverWorlds: {
		hint: "Worlds detected in the server's folder.",
		empty: "No world generated yet — it's created automatically the first time the server starts.",
		delete: 'Delete world',
		confirmDelete: 'Sure? Delete'
	},
	serverFileManager: {
		newFolder: 'New folder',
		newFolderPrompt: 'Folder name',
		up: 'Go up one level',
		empty: 'This folder is empty.',
		save: 'Save'
	},
	serverInstance: {
		start: 'Start server',
		starting: 'Starting…',
		stop: 'Stop server',
		stopping: 'Stopping…',
		build: 'Build',
		tabPlugins: 'Plugins',
		tabWorlds: 'Worlds',
		tabFiles: 'Files',
		tabConsole: 'Console',
		connectionLabel: 'How to connect',
		connectionLanHint: 'For another PC on the same Wi-Fi, give them this address as is.',
		connectionPublicReady: "This address works for anyone, anywhere — TFL Client opened the port on your router automatically.",
		connectionPublicManual:
			"This address works for anyone, but your router didn't let TFL Client open it automatically: you'll need to port forward {{port}} manually — search \"port forwarding\" plus your router model.",
		connectionLanFallback: 'On the same Wi-Fi, {{address}} also works directly.',
		connectionUnavailable: "Couldn't detect a network address — check that you have internet or a local network connection.",
		connectionTunnelReady: 'This address works for anyone, anywhere — it doesn\'t depend on your router.',
		playitPitch: "Still fighting with the connection? Link a free playit.gg account (one click, no card) and the address will always work, no router needed.",
		playitLinkButton: 'Link playit.gg',
		playitLinking: 'Waiting for confirmation in the browser…'
	},
	skinManager: {
		title: 'Skin & cape',
		tabSkin: 'Skin',
		tabCape: 'Cape',
		variantClassic: 'Classic',
		variantSlim: 'Slim (thin arms)',
		uploadSkin: 'Upload skin',
		resetSkin: 'Reset to default',
		noCapes: "This account has no capes — they come from Mojang events/promotions.",
		noCape: 'No cape',
		viewerHint: 'Drag to rotate, scroll to zoom'
	},
	serverConnectionInfo: {
		title: 'How others connect to your server',
		lead: "When you start the server, TFL Client tries to make the address work on its own. Here's what can happen:",
		upnpTitle: 'The normal case: automatic',
		upnpSub: "If your router supports UPnP (most do), TFL Client opens the port on its own — copy the address and that's it, nothing else to do.",
		playitTitle: "If your router doesn't cooperate: playit.gg",
		playitSub: 'A button appears to link a free playit.gg account — one click, once, never needed again after that.',
		gotIt: 'Got it'
	}
};
