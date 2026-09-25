<script lang="ts">
	import { onMount } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { check as checkForUpdate, type Update } from '@tauri-apps/plugin-updater';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { appState } from '$lib/state/state.svelte';
	import { t, i18nState, setLocale, type TranslationKey } from '$lib/i18n/index.svelte';
	import Mascot from '$lib/components/ui/Mascot.svelte';
	import { MASCOTS, getMascotFor, setMascotFor, type MascotId } from '$lib/mascots';
	import {
		updateSettings,
		setQualityProfile,
		getRecommendedRam,
		clearTempCache,
		getJavaStatus,
		getUserList,
		switchUser,
		removeUser,
		addOfflineAccount,
		getCurrentUser,
		getDeviceCode,
		authenticateWithDeviceCode,
		pickImageFile,
		setCustomWallpaper,
		getCustomWallpaperPath
	} from '$lib/api/tflApi';
	import type { QualityProfile, MinecraftUser, JavaStatus } from '$lib/types/types';
	import {
		X,
		Check,
		Trash2,
		Plus,
		Coffee,
		Gamepad2,
		Loader2,
		SlidersHorizontal,
		Users,
		Palette,
		PanelLeft,
		WandSparkles,
		Copy,
		CheckCircle2,
		Download,
		RefreshCw,
		Upload
	} from 'lucide-svelte';

	let { onClose }: { onClose: () => void } = $props();

	const QUALITY: QualityProfile[] = ['Lite', 'Balanced', 'Experience'];
	const ACCENTS: { id: string; key: TranslationKey; color: string }[] = [
		{ id: 'orange', key: 'settings.accent.orange', color: '#ff7a2e' },
		{ id: 'violet', key: 'settings.accent.violet', color: '#8b5cf6' },
		{ id: 'teal', key: 'settings.accent.teal', color: '#14b8a6' },
		{ id: 'blue', key: 'settings.accent.blue', color: '#3b82f6' },
		{ id: 'rose', key: 'settings.accent.rose', color: '#f43f5e' },
		{ id: 'lime', key: 'settings.accent.lime', color: '#84cc16' },
		{ id: 'iris', key: 'settings.accent.iris', color: '#5b5bd6' },
		{ id: 'jade', key: 'settings.accent.jade', color: '#29a383' },
		{ id: 'crimson', key: 'settings.accent.crimson', color: '#e93d82' },
		{ id: 'cyan', key: 'settings.accent.cyan', color: '#00a2c7' },
		{ id: 'grass', key: 'settings.accent.grass', color: '#46a758' },
		{ id: 'plum', key: 'settings.accent.plum', color: '#ab4aba' }
	];
	const SURFACES: { id: string; key: TranslationKey }[] = [
		{ id: 'obsidian', key: 'settings.surface.obsidian' },
		{ id: 'midnight', key: 'settings.surface.midnight' },
		{ id: 'slate', key: 'settings.surface.slate' },
		{ id: 'oled', key: 'settings.surface.oled' }
	];
	const AMBIENCES: { id: string; key: TranslationKey }[] = [
		{ id: 'aurora', key: 'settings.ambience.aurora' },
		{ id: 'cosmic', key: 'settings.ambience.cosmic' },
		{ id: 'minimal', key: 'settings.ambience.minimal' },
		{ id: 'particles', key: 'settings.ambience.particles' }
	];
	const FONTS: { id: string; key?: TranslationKey; label?: string; family: string }[] = [
		{ id: 'system', key: 'settings.font.system', family: 'inherit' },
		{ id: 'nunito', label: 'Nunito', family: "'Nunito', sans-serif" },
		{ id: 'inter', label: 'Inter', family: "'Inter', sans-serif" },
		{ id: 'poppins', label: 'Poppins', family: "'Poppins', sans-serif" },
		{ id: 'jetbrains-mono', label: 'JetBrains Mono', family: "'JetBrains Mono', monospace" }
	];
	const WALLPAPERS: { id: string; key: TranslationKey; preview: string }[] = [
		{ id: 'none', key: 'settings.wallpaper.items.none', preview: 'transparent' },
		{
			id: 'void-night',
			key: 'settings.wallpaper.items.voidNight',
			preview: 'linear-gradient(135deg, #02040a, #050d1a, #030810)'
		},
		{
			id: 'nether',
			key: 'settings.wallpaper.items.nether',
			preview: 'linear-gradient(135deg, #140202, #250808, #0e0101)'
		},
		{
			id: 'end',
			key: 'settings.wallpaper.items.end',
			preview: 'linear-gradient(135deg, #070212, #0e0520, #040110)'
		},
		{
			id: 'deep-ocean',
			key: 'settings.wallpaper.items.deepOcean',
			preview: 'linear-gradient(135deg, #010f1a, #021a2e, #010c14)'
		},
		{
			id: 'aurora-gradient',
			key: 'settings.wallpaper.items.auroraGradient',
			preview: 'linear-gradient(135deg, #040d10, #041410 40%, #0a0420 80%, #04100d)'
		},
		{
			id: 'animated-aurora',
			key: 'settings.wallpaper.items.animatedAurora',
			preview: 'linear-gradient(135deg, #040d10, #041410 40%, #0a0420 80%, #04100d)'
		},
		{ id: 'obsidian-solid', key: 'settings.wallpaper.items.obsidianSolid', preview: '#08090c' },
		{ id: 'charcoal', key: 'settings.wallpaper.items.charcoal', preview: '#0f1115' },
		{ id: 'savanna', key: 'settings.wallpaper.items.savanna', preview: 'url(/wallpapers/mc-wallpaper-1.jpg)' },
		{
			id: 'golden-sunset',
			key: 'settings.wallpaper.items.goldenSunset',
			preview: 'url(/wallpapers/mc-wallpaper-2.jpg)'
		},
		{
			id: 'lake-night',
			key: 'settings.wallpaper.items.lakeNight',
			preview: 'url(/wallpapers/mc-wallpaper-3.jpg)'
		},
		{
			id: 'neon-arcade',
			key: 'settings.wallpaper.items.neonArcade',
			preview: 'url(/wallpapers/mc-wallpaper-4.jpg)'
		},
		{
			id: 'red-canyon',
			key: 'settings.wallpaper.items.redCanyon',
			preview: 'url(/wallpapers/mc-wallpaper-5.jpg)'
		},
		{
			id: 'enchanted-valley',
			key: 'settings.wallpaper.items.enchantedValley',
			preview: 'url(/wallpapers/mc-wallpaper-6.jpg)'
		},
		{
			id: 'snowy-peak',
			key: 'settings.wallpaper.items.snowyPeak',
			preview: 'url(/wallpapers/mc-wallpaper-7.jpg)'
		},
		{
			id: 'stone-bridge',
			key: 'settings.wallpaper.items.stoneBridge',
			preview: 'url(/wallpapers/mc-wallpaper-8.jpg)'
		},
		{
			id: 'misty-fortress',
			key: 'settings.wallpaper.items.mistyFortress',
			preview: 'url(/wallpapers/mc-wallpaper-9.jpg)'
		},
		{
			id: 'village-tower',
			key: 'settings.wallpaper.items.villageTower',
			preview: 'url(/wallpapers/mc-wallpaper-10.jpg)'
		},
		{
			id: 'deep-cave',
			key: 'settings.wallpaper.items.deepCave',
			preview: 'url(/wallpapers/mc-wallpaper-11.jpg)'
		},
		{
			id: 'sunset-coast',
			key: 'settings.wallpaper.items.sunsetCoast',
			preview: 'url(/wallpapers/mc-wallpaper-12.jpg)'
		},
		{
			id: 'abstract-blocks',
			key: 'settings.wallpaper.items.abstractBlocks',
			preview: 'url(/wallpapers/mc-wallpaper-13.jpg)'
		}
	];

	let tab = $state<'general' | 'appearance' | 'accounts' | 'java'>('general');

	let accent = $state(
		typeof localStorage !== 'undefined'
			? (localStorage.getItem('tfl-accent') ?? 'orange')
			: 'orange'
	);
	let surface = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-surface') ?? 'obsidian') : 'obsidian');
	let ambience = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-ambience') ?? 'aurora') : 'aurora');
	let font = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-font') ?? 'system') : 'system');
	let density = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-density') ?? 'comfortable') : 'comfortable');
	let wallpaper = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-wallpaper') ?? 'none') : 'none');
	let cardStyle = $state(
		typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-card-style') ?? 'rich') : 'rich'
	);
	let customWallpaperUrl = $state<string | null>(null);
	let customWallpaperBusy = $state(false);
	let customWallpaperError = $state<string | null>(null);

	// Subir imagen propia usa el diálogo nativo de archivos, que crashea el
	// launcher entero en macOS con la firma ad-hoc del build (ver
	// CHANGELOG_macos-dialog-crash-java26.txt) — por eso queda detrás del
	// modo global "Automático/Manual" definido más abajo, no de la
	// plataforma.
	let dialogsBlocked = $derived(appState.settings?.native_dialog_mode !== 'manual');

	let ramTotal = $state<number | null>(null);
	let minRam = $state(appState.settings?.min_memory ?? 1024);
	let maxRam = $state(appState.settings?.max_memory ?? 2048);
	let savingRam = $state(false);

	let users = $state<MinecraftUser[]>([]);
	let newOfflineName = $state('');
	let accountBusy = $state(false);
	let msCode = $state<string | null>(null);
	let msVerificationUri = $state<string | null>(null);
	let msBusy = $state(false);
	let msError = $state<string | null>(null);
	let codeCopied = $state(false);
	let mascotTick = $state(0);

	let javaStatuses = $state<JavaStatus[]>([]);

	let updateChecking = $state(false);
	let updateInstalling = $state(false);
	let updateInfo = $state<Update | null>(null);
	let updateError = $state<string | null>(null);
	let updateChecked = $state(false);

	async function handleCheckForUpdate() {
		updateChecking = true;
		updateError = null;
		try {
			updateInfo = await checkForUpdate();
			updateChecked = true;
		} catch (e) {
			// Endpoint todavía no configurado (host de updates pendiente) — no es un
			// error del usuario, solo significa que la infra de updates no está lista.
			updateError = String(e);
		} finally {
			updateChecking = false;
		}
	}

	async function handleInstallUpdate() {
		if (!updateInfo) return;
		updateInstalling = true;
		updateError = null;
		try {
			await updateInfo.downloadAndInstall();
			await relaunch();
		} catch (e) {
			updateError = String(e);
			updateInstalling = false;
		}
	}

	onMount(async () => {
		try {
			const ram = await getRecommendedRam();
			ramTotal = ram.total_mb;
		} catch {
			// no bloquea el panel si falla
		}
		try {
			const path = await getCustomWallpaperPath();
			if (path) customWallpaperUrl = `${convertFileSrc(path)}?t=${Date.now()}`;
		} catch {
			// sin wallpaper propio subido todavía, no es un error
		}
	});

	function applyAccent(id: string) {
		accent = id;
		localStorage.setItem('tfl-accent', id);
		if (id === 'orange') {
			document.documentElement.removeAttribute('data-accent');
		} else {
			document.documentElement.setAttribute('data-accent', id);
		}
	}

	function applyPreference(key: string, attribute: string, value: string, defaultValue: string) {
		localStorage.setItem(key, value);
		if (value === defaultValue) document.documentElement.removeAttribute(attribute);
		else document.documentElement.setAttribute(attribute, value);
	}

	function applySurface(id: string) {
		// OLED (negro puro) solo tiene variante oscura a propósito — el botón
		// ya viene deshabilitado en modo claro (ver markup), así que llegar
		// acá con theme==='light' no debería pasar nunca por la UI normal.
		if (id === 'oled' && appState.settings?.theme === 'light') return;
		surface = id;
		applyPreference('tfl-surface', 'data-surface', id, 'obsidian');
	}
	function applyAmbience(id: string) {
		ambience = id;
		applyPreference('tfl-ambience', 'data-ambience', id, 'aurora');
	}
	function applyFont(id: string) {
		font = id;
		applyPreference('tfl-font', 'data-font', id, 'system');
	}
	function applyWallpaper(id: string) {
		wallpaper = id;
		applyPreference('tfl-wallpaper', 'data-wallpaper', id, 'none');
		// El wallpaper propio setea --wallpaper-bg inline (ver
		// selectCustomWallpaper) — al elegir cualquier otro fondo hay que
		// sacar ese inline override, si no pisa para siempre la variable que
		// define la hoja de estilos para el fondo built-in elegido.
		if (id !== 'custom') {
			document.documentElement.style.removeProperty('--wallpaper-bg');
		}
	}

	function selectCustomWallpaper() {
		if (!customWallpaperUrl) return;
		wallpaper = 'custom';
		localStorage.setItem('tfl-wallpaper', 'custom');
		document.documentElement.setAttribute('data-wallpaper', 'custom');
		document.documentElement.style.setProperty('--wallpaper-bg', `url("${customWallpaperUrl}")`);
	}

	async function pickCustomWallpaper() {
		if (dialogsBlocked) return;
		customWallpaperError = null;
		customWallpaperBusy = true;
		try {
			const source = await pickImageFile();
			if (!source) return;
			const path = await setCustomWallpaper(source);
			customWallpaperUrl = `${convertFileSrc(path)}?t=${Date.now()}`;
			selectCustomWallpaper();
		} catch (e) {
			customWallpaperError = String(e);
		} finally {
			customWallpaperBusy = false;
		}
	}
	function applyDensity(id: string) {
		density = id;
		applyPreference('tfl-density', 'data-density', id, 'comfortable');
	}
	function applyCardStyle(id: string) {
		cardStyle = id;
		applyPreference('tfl-card-style', 'data-card-style', id, 'rich');
	}

	async function setTheme(theme: 'dark' | 'light') {
		if (!appState.settings) return;
		// OLED no tiene variante clara — el botón "Claro" ya viene
		// deshabilitado mientras surface === 'oled' (ver markup), esto es
		// solo el guard defensivo del lado de la función.
		if (theme === 'light' && surface === 'oled') return;
		document.documentElement.setAttribute('data-theme', theme);
		// Cada superficie define su propia variante clara y oscura (ver
		// global.css) — cambiar de tema ya no tiene que tocar ni resetear
		// data-surface, la elección del usuario se mantiene en los dos.
		appState.settings.theme = theme;
		await updateSettings(appState.settings);
	}

	async function toggleAutoUpdates() {
		if (!appState.settings) return;
		appState.settings.auto_updates = !appState.settings.auto_updates;
		await updateSettings(appState.settings);
	}

	async function setNativeDialogMode(mode: 'auto' | 'manual') {
		if (!appState.settings) return;
		appState.settings.native_dialog_mode = mode;
		appState.settings.native_dialog_mode_prompted = true;
		await updateSettings(appState.settings);
	}

	async function handleQuality(profile: QualityProfile) {
		if (!appState.settings) return;
		const updated = await setQualityProfile(profile);
		appState.settings = updated;
		document.documentElement.setAttribute('data-quality', profile.toLowerCase());
		document.documentElement.toggleAttribute('data-reduce-motion', profile === 'Lite');
		document.documentElement.toggleAttribute('data-no-blur', updated.disable_blur_effects);
		document.documentElement.toggleAttribute('data-no-infinite-fx', updated.disable_infinite_animations);
	}

	function changeMascot(uuid: string, id: MascotId) {
		setMascotFor(uuid, id);
		mascotTick++;
	}

	async function copyMicrosoftCode() {
		if (!msCode) return;
		try {
			await writeText(msCode);
			codeCopied = true;
			window.setTimeout(() => (codeCopied = false), 1800);
		} catch {
			msError = t('onboarding.copyCodeFailed');
		}
	}

	async function saveRam() {
		if (!appState.settings) return;
		savingRam = true;
		try {
			appState.settings.min_memory = minRam;
			appState.settings.max_memory = maxRam;
			await updateSettings(appState.settings);
		} finally {
			savingRam = false;
		}
	}

	let clearingCache = $state(false);
	let cacheClearedMsg = $state<string | null>(null);
	async function handleClearCache() {
		clearingCache = true;
		cacheClearedMsg = null;
		try {
			const freedBytes = await clearTempCache();
			const freedMb = (freedBytes / 1024 / 1024).toFixed(1);
			cacheClearedMsg = t('settings.storage.cacheCleared', { mb: freedMb });
		} catch (e) {
			cacheClearedMsg = t('settings.storage.clearCacheFailed', { error: String(e) });
		} finally {
			clearingCache = false;
		}
	}

	async function loadAccounts() {
		users = await getUserList();
	}

	async function handleSwitch(uuid: string) {
		accountBusy = true;
		try {
			appState.currentUser = await switchUser(uuid);
			await loadAccounts();
		} finally {
			accountBusy = false;
		}
	}

	async function handleRemove(uuid: string) {
		accountBusy = true;
		try {
			await removeUser(uuid);
			appState.currentUser = await getCurrentUser();
			await loadAccounts();
		} finally {
			accountBusy = false;
		}
	}

	async function handleAddOffline() {
		if (!newOfflineName.trim()) return;
		accountBusy = true;
		try {
			await addOfflineAccount(newOfflineName.trim());
			newOfflineName = '';
			await loadAccounts();
		} finally {
			accountBusy = false;
		}
	}

	async function handleAddMicrosoft() {
		msBusy = true;
		msError = null;
		msCode = null;
		try {
			const dc = await getDeviceCode();
			msCode = dc.user_code;
			msVerificationUri = dc.verification_uri;
			appState.currentUser = await authenticateWithDeviceCode(
				dc.device_code,
				dc.interval,
				dc.expires_in
			);
			msCode = null;
			await loadAccounts();
		} catch (e) {
			msError = String(e);
		} finally {
			msBusy = false;
		}
	}

	async function loadJava() {
		javaStatuses = await getJavaStatus();
	}

	function selectTab(t: 'general' | 'appearance' | 'accounts' | 'java') {
		tab = t;
		if (t === 'accounts') loadAccounts();
		if (t === 'java') loadJava();
	}
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div
	class="overlay"
	onclick={onClose}
	onkeydown={(e) => e.key === 'Escape' && onClose()}
	role="button"
	tabindex="-1"
>
	<div
		class="panel"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div class="panel-header">
			<h2>{t('settings.title')}</h2>
			<button type="button" class="close-btn" onclick={onClose} aria-label={t('settings.close')}>
				<X size={16} />
			</button>
		</div>

		<div class="panel-tabs">
			<button
				type="button"
				class="ptab"
				class:active={tab === 'appearance'}
				onclick={() => selectTab('appearance')}
			>
				<Palette size={14} /> {t('settings.tabs.appearance')}
			</button>
			<button
				type="button"
				class="ptab"
				class:active={tab === 'general'}
				onclick={() => selectTab('general')}
			>
				<SlidersHorizontal size={14} /> {t('settings.tabs.general')}
			</button>
			<button
				type="button"
				class="ptab"
				class:active={tab === 'accounts'}
				onclick={() => selectTab('accounts')}
			>
				<Users size={14} /> {t('settings.tabs.accounts')}
			</button>
			<button
				type="button"
				class="ptab"
				class:active={tab === 'java'}
				onclick={() => selectTab('java')}
			>
				<Coffee size={14} /> {t('settings.tabs.java')}
			</button>
		</div>

		<div class="panel-body">
			{#if tab === 'general'}
				<section>
					<span class="section-label">{t('settings.language')}</span>
					<div class="row">
						<button type="button" class="choice" class:active={i18nState.locale === 'es'} onclick={() => setLocale('es')}>Español</button>
						<button type="button" class="choice" class:active={i18nState.locale === 'en'} onclick={() => setLocale('en')}>English</button>
					</div>
				</section>

				<section>
					<span class="section-label">{t('settings.theme.label')}</span>
					<div class="row">
						<button
							type="button"
							class="choice"
							class:active={appState.settings?.theme === 'dark'}
							onclick={() => setTheme('dark')}>{t('settings.theme.dark')}</button
						>
						<button
							type="button"
							class="choice"
							class:active={appState.settings?.theme === 'light'}
							disabled={surface === 'oled'}
							title={surface === 'oled' ? t('settings.theme.lightDisabledTitle') : undefined}
							onclick={() => setTheme('light')}>{t('settings.theme.light')}</button
						>
					</div>
				</section>

				<section>
					<span class="section-label">{t('settings.quality.label')}</span>
					<div class="row">
						{#each QUALITY as q (q)}
							<button
								type="button"
								class="choice"
								class:active={appState.settings?.quality_profile === q}
								onclick={() => handleQuality(q)}>{q}</button
							>
						{/each}
					</div>
				</section>

				<section>
					<span class="section-label">
						{t('settings.ram.label')}
						{#if ramTotal}<span class="ram-total"
								>{t('settings.ram.detected', { gb: Math.round(ramTotal / 1024) })}</span
							>{/if}
					</span>
					<div class="ram-row">
						<label>
							{t('settings.ram.min')}
							<input type="number" bind:value={minRam} min="512" step="256" />
						</label>
						<label>
							{t('settings.ram.max')}
							<input type="number" bind:value={maxRam} min="512" step="256" />
						</label>
					</div>
					<button type="button" class="save-btn" disabled={savingRam} onclick={saveRam}>
						<Check size={13} /> {t('settings.ram.save')}
					</button>
				</section>

				<section>
					<span class="section-label">{t('settings.storage.label')}</span>
					<button type="button" class="save-btn" disabled={clearingCache} onclick={handleClearCache}>
						{#if clearingCache}<Loader2 size={13} class="spin" />{:else}<Trash2 size={13} />{/if}
						{t('settings.storage.clearCache')}
					</button>
					{#if cacheClearedMsg}<p class="hint">{cacheClearedMsg}</p>{/if}
				</section>

				<section>
					<span class="section-label">{t('settings.dialogs.label')}</span>
					<div class="row">
						<button
							type="button"
							class="choice"
							class:active={(appState.settings?.native_dialog_mode ?? 'auto') === 'auto'}
							onclick={() => setNativeDialogMode('auto')}
						>
							{t('settings.dialogs.auto')}
						</button>
						<button
							type="button"
							class="choice"
							class:active={appState.settings?.native_dialog_mode === 'manual'}
							onclick={() => setNativeDialogMode('manual')}
						>
							{t('settings.dialogs.manual')}
						</button>
					</div>
					<p class="hint">
						{t('settings.dialogs.hint')}
					</p>
				</section>

				<section>
					<span class="section-label">{t('settings.updates.label')}</span>
					<button
						type="button"
						class="choice auto-update-toggle"
						class:active={appState.settings?.auto_updates}
						onclick={toggleAutoUpdates}
					>
						{appState.settings?.auto_updates ? t('settings.updates.autoEnabled') : t('settings.updates.autoDisabled')}
					</button>
					<p class="hint">
						{t('settings.updates.hint')}
					</p>
					<button
						type="button"
						class="save-btn"
						disabled={updateChecking || updateInstalling}
						onclick={handleCheckForUpdate}
					>
						{#if updateChecking}
							<Loader2 size={13} class="spin" />
						{:else}
							<RefreshCw size={13} />
						{/if}
						{t('settings.updates.check')}
					</button>

					{#if updateChecked && !updateInfo && !updateError}
						<p class="hint">{t('settings.updates.upToDate')}</p>
					{/if}

					{#if updateInfo}
						<p class="hint">{t('settings.updates.available', { version: updateInfo.version })}</p>
						<button
							type="button"
							class="save-btn"
							disabled={updateInstalling}
							onclick={handleInstallUpdate}
						>
							{#if updateInstalling}
								<Loader2 size={13} class="spin" />
							{:else}
								<Download size={13} />
							{/if}
							{t('settings.updates.download')}
						</button>
					{/if}

					{#if updateError}<p class="error">{updateError}</p>{/if}
				</section>
			{:else if tab === 'appearance'}
				<section>
					<span class="section-label">{t('settings.accent.label')}</span>
					<div class="swatches">
						{#each ACCENTS as a (a.id)}
							<button type="button" class="swatch" class:active={accent === a.id} style="background: {a.color}; color: {a.color}" onclick={() => applyAccent(a.id)} aria-label={t(a.key)}></button>
						{/each}
					</div>
				</section>
				<section>
					<span class="section-label"><PanelLeft size={13} /> {t('settings.surface.label')}</span>
					<div class="row">{#each SURFACES as item (item.id)}<button type="button" class="choice" class:active={surface === item.id} disabled={item.id === 'oled' && appState.settings?.theme === 'light'} title={item.id === 'oled' && appState.settings?.theme === 'light' ? t('settings.surface.incompatibleTitle') : undefined} onclick={() => applySurface(item.id)}>{t(item.key)}</button>{/each}</div>
				</section>
				<section>
					<span class="section-label"><WandSparkles size={13} /> {t('settings.ambience.label')}</span>
					<div class="row">{#each AMBIENCES as item (item.id)}<button type="button" class="choice" class:active={ambience === item.id} onclick={() => applyAmbience(item.id)}>{t(item.key)}</button>{/each}</div>
				</section>
				<section>
					<span class="section-label"><WandSparkles size={13} /> {t('settings.wallpaper.label')}</span>
					<div class="wallpaper-grid">
						{#each WALLPAPERS as item (item.id)}
							<button
								type="button"
								class="wallpaper-swatch"
								class:active={wallpaper === item.id}
								style="background: {item.preview}; background-size: cover; background-position: center;"
								onclick={() => applyWallpaper(item.id)}
								aria-label={t(item.key)}
								title={t(item.key)}
							>
								{#if wallpaper === item.id}<Check size={12} />{/if}
							</button>
						{/each}
						{#if customWallpaperUrl}
							<button
								type="button"
								class="wallpaper-swatch"
								class:active={wallpaper === 'custom'}
								style="background-image: url('{customWallpaperUrl}'); background-size: cover; background-position: center;"
								onclick={selectCustomWallpaper}
								aria-label={t('settings.wallpaper.yourImage')}
								title={t('settings.wallpaper.yourImage')}
							>
								{#if wallpaper === 'custom'}<Check size={12} />{/if}
							</button>
						{/if}
						<button
							type="button"
							class="wallpaper-swatch wallpaper-upload"
							onclick={pickCustomWallpaper}
							disabled={customWallpaperBusy || dialogsBlocked}
							aria-label={customWallpaperUrl ? t('settings.wallpaper.change') : t('settings.wallpaper.upload')}
							title={dialogsBlocked
								? t('settings.wallpaper.uploadDisabledTitle')
								: customWallpaperUrl
									? t('settings.wallpaper.change')
									: t('settings.wallpaper.upload')}
						>
							<Upload size={13} />
						</button>
					</div>
					{#if dialogsBlocked}<p class="mac-notice">{t('settings.wallpaper.autoModeDisabled')}</p>{/if}
					{#if customWallpaperError}<p class="error-text">{customWallpaperError}</p>{/if}
				</section>
				<section>
					<span class="section-label">{t('settings.density.label')}</span>
					<div class="row"><button type="button" class="choice" class:active={density === 'comfortable'} onclick={() => applyDensity('comfortable')}>{t('settings.density.comfortable')}</button><button type="button" class="choice" class:active={density === 'compact'} onclick={() => applyDensity('compact')}>{t('settings.density.compact')}</button></div>
				</section>
				<section>
					<span class="section-label">{t('settings.cardStyle.label')}</span>
					<div class="row">
						<button type="button" class="choice" class:active={cardStyle === 'rich'} onclick={() => applyCardStyle('rich')}>{t('settings.cardStyle.rich')}</button>
						<button type="button" class="choice" class:active={cardStyle === 'minimal'} onclick={() => applyCardStyle('minimal')}>{t('settings.cardStyle.minimal')}</button>
					</div>
				</section>
				<section>
					<span class="section-label">{t('settings.font.label')}</span>
					<div class="row">
						{#each FONTS as f (f.id)}
							<button
								type="button"
								class="choice"
								class:active={font === f.id}
								style="font-family: {f.family}"
								onclick={() => applyFont(f.id)}
							>
								{f.key ? t(f.key) : f.label}
							</button>
						{/each}
					</div>
					<p class="hint">{t('settings.font.hint')}</p>
				</section>
			{:else if tab === 'accounts'}
				<section>
					<span class="section-label">{t('settings.accounts.saved')}</span>
					<div class="accounts-list">
						{#each users as u (u.uuid)}
							<div class="account-row" class:active={appState.currentUser?.uuid === u.uuid}>
								{#if u.user_type === 'Cracked'}
									<span class="account-avatar">
										{#key mascotTick}
											<Mascot id={getMascotFor(u.uuid)} size={24} />
										{/key}
									</span>
								{/if}
								<div class="account-info">
									<span class="account-name">{u.username}</span>
									<span class="account-type">{u.user_type}</span>
								</div>
								{#if appState.currentUser?.uuid !== u.uuid}
									<button
										type="button"
										class="mini-btn"
										disabled={accountBusy}
										onclick={() => handleSwitch(u.uuid)}
									>
										{t('settings.accounts.use')}
									</button>
								{:else}
									<span class="active-tag">{t('settings.accounts.active')}</span>
								{/if}
								<button
									type="button"
									class="mini-icon-btn"
									disabled={accountBusy}
									onclick={() => handleRemove(u.uuid)}
									aria-label={t('settings.accounts.remove')}
								>
									<Trash2 size={13} />
								</button>
							</div>
							{#if u.user_type === 'Cracked' && appState.currentUser?.uuid === u.uuid}
								<div class="mascot-picker">
									{#key mascotTick}
										{#each MASCOTS as m (m.id)}
											<button
												type="button"
												class="mascot-swatch"
												class:active={getMascotFor(u.uuid) === m.id}
												onclick={() => changeMascot(u.uuid, m.id)}
												aria-label={m.label}
												title={m.label}
											>
												<Mascot id={m.id} size={26} />
											</button>
										{/each}
									{/key}
								</div>
							{/if}
						{/each}
					</div>
				</section>

				<section>
					<span class="section-label">{t('settings.accounts.addMicrosoft')}</span>
					{#if msCode}
						<div class="ms-pending">
							<p>{t('onboarding.goTo')} <a href={msVerificationUri} target="_blank" rel="noreferrer">{msVerificationUri}</a> {t('settings.accounts.microsoftEnter')}</p>
						<div class="ms-code-row">
							<code class="ms-code" tabindex="0">{msCode}</code>
							<button type="button" class="copy-code-btn" onclick={copyMicrosoftCode} aria-label={t('settings.accounts.copyCode')}>
								{#if codeCopied}<CheckCircle2 size={14} /> {t('common.copied')}{:else}<Copy size={14} /> {t('common.copy')}{/if}
							</button>
						</div>
							<p class="hint">{t('settings.accounts.waitingConfirmation')}</p>
						</div>
					{:else}
						<button type="button" class="mini-btn ms-btn" disabled={msBusy} onclick={handleAddMicrosoft}>
							{#if msBusy}<Loader2 size={13} class="spin" />{:else}<Gamepad2 size={13} />{/if}
							{t('settings.accounts.startMicrosoft')}
						</button>
					{/if}
					{#if msError}<p class="error">{msError}</p>{/if}
				</section>

				<section>
					<span class="section-label">{t('settings.accounts.addOffline')}</span>
					<div class="add-account-row">
						<input
							type="text"
							bind:value={newOfflineName}
							placeholder={t('onboarding.usernameLabel')}
							maxlength="16"
						/>
						<button
							type="button"
							class="mini-btn"
							disabled={accountBusy || !newOfflineName.trim()}
							onclick={handleAddOffline}
						>
							<Plus size={13} />
						</button>
					</div>
					<p class="hint">{t('onboarding.offlineHint')}</p>
				</section>
			{:else if tab === 'java'}
				<section>
					<span class="section-label">{t('settings.java.label')}</span>
					<p class="hint">
						{t('settings.java.hint')}
					</p>
					<div class="java-list">
						{#each javaStatuses as j (j.major)}
							<div class="java-row">
								<Coffee size={14} />
								<span class="java-major">{t('settings.java.major', { major: j.major })}</span>
								{#if j.installed}
									<span class="java-tag installed">{t('settings.java.installed')}</span>
								{:else}
									<span class="java-tag">{t('settings.java.notInstalled')}</span>
								{/if}
							</div>
						{/each}
					</div>
				</section>
			{/if}
		</div>
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.panel {
		width: 440px;
		max-height: 80vh;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 22px;
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.panel-header h2 {
		font-size: var(--text-lg);
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.panel-tabs {
		display: flex;
		gap: 4px;
		border-bottom: 1px solid var(--border);
	}

	.ptab {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 4px;
		margin-right: 18px;
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--text-secondary);
		font-size: 0.8rem;
		font-weight: 600;
		cursor: pointer;
		transition: color 0.15s;
	}

	.ptab:hover {
		color: var(--text-primary);
	}

	.ptab.active {
		color: var(--accent);
		border-bottom-color: var(--accent);
	}

	.panel-body {
		display: flex;
		flex-direction: column;
		gap: 18px;
		overflow-y: auto;
		overflow-x: hidden;
		/* Sin este padding lateral, el anillo del swatch de color activo
		   (box-shadow que sobresale ~6px del círculo) queda pegado al
		   borde de este contenedor y overflow-y:auto recorta también el
		   eje horizontal (comportamiento real de la spec: si un eje no es
		   "visible", el otro se computa como "auto" aunque no se haya
		   puesto explícito) — se veía cortado/"saliendo" del panel. */
		padding: 0 4px;
	}

	.section-label {
		display: block;
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
		margin-bottom: 8px;
	}

	.section-label :global(svg) {
		vertical-align: -2px;
		margin-right: 4px;
		color: var(--accent);
	}

	.ram-total {
		text-transform: none;
		font-weight: 400;
		color: var(--text-muted);
		letter-spacing: 0;
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.swatches {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
	}

	.choice {
		flex: 1;
		padding: 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.78rem;
		font-weight: 600;
		cursor: pointer;
	}

	.choice.active {
		background: var(--accent);
		border-color: var(--accent);
		color: var(--accent-text);
	}

	.choice:disabled {
		opacity: .42;
		cursor: not-allowed;
	}

	.swatch {
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 2px solid var(--bg-card);
		box-shadow: 0 0 0 1px var(--border);
		cursor: pointer;
		transition: box-shadow 0.15s;
	}

	.swatch.active {
		box-shadow: 0 0 0 2px var(--bg-card), 0 0 0 4px currentColor;
	}

	.wallpaper-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 8px;
	}

	.wallpaper-swatch {
		aspect-ratio: 16 / 10;
		border-radius: var(--border-radius-sm);
		border: 2px solid var(--border);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #ffffff;
		opacity: 0.75;
		transition:
			opacity 0.15s,
			border-color 0.15s,
			transform 0.15s;
	}

	.wallpaper-swatch:hover {
		opacity: 1;
		transform: translateY(-1px);
	}

	.wallpaper-swatch.active {
		opacity: 1;
		border-color: var(--accent);
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 30%, transparent);
	}

	.wallpaper-upload {
		background: var(--bg-input);
		border-style: dashed;
		color: var(--text-muted);
	}

	.wallpaper-upload:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.error-text {
		margin-top: 8px;
		font-size: 0.75rem;
		color: var(--color-error);
	}

	.mac-notice {
		margin-top: 8px;
		font-size: 0.75rem;
		color: var(--text-secondary);
	}

	.ram-row {
		display: flex;
		gap: 10px;
		margin-bottom: 10px;
	}

	.ram-row label {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 0.72rem;
		color: var(--text-secondary);
	}

	.ram-row input {
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.85rem;
	}

	.save-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: var(--accent);
		color: var(--accent-text);
		font-size: 0.75rem;
		font-weight: 700;
		cursor: pointer;
	}

	.save-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.accounts-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.account-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
	}

	.account-row.active {
		border-color: var(--accent);
	}

	.account-avatar {
		flex-shrink: 0;
		display: flex;
		border-radius: var(--border-radius-sm);
		overflow: hidden;
	}

	.mascot-picker {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		padding: 8px 10px 2px;
	}

	.mascot-swatch {
		display: flex;
		border-radius: var(--border-radius-sm);
		border: 2px solid transparent;
		cursor: pointer;
		overflow: hidden;
		opacity: 0.7;
		transition:
			opacity 0.15s,
			border-color 0.15s;
	}

	.mascot-swatch:hover {
		opacity: 1;
	}

	.mascot-swatch.active {
		opacity: 1;
		border-color: var(--accent);
	}

	.account-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.account-name {
		font-size: 0.8rem;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.account-type {
		font-size: 0.62rem;
		color: var(--text-muted);
	}

	.active-tag {
		font-size: 0.62rem;
		font-weight: 700;
		color: var(--accent);
		text-transform: uppercase;
	}

	.mini-btn {
		padding: 5px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: transparent;
		color: var(--text-secondary);
		font-size: 0.7rem;
		font-weight: 600;
		cursor: pointer;
		white-space: nowrap;
	}

	.mini-icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.mini-icon-btn:hover {
		color: var(--color-error);
	}

	.add-account-row {
		display: flex;
		gap: 8px;
	}

	.add-account-row input {
		flex: 1;
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.82rem;
	}

	.hint {
		font-size: 0.7rem;
		color: var(--text-muted);
		margin-top: 6px;
	}

	.auto-update-toggle {
		flex: none;
		width: fit-content;
	}

	.error {
		font-size: 0.72rem;
		color: var(--color-error);
		margin-top: 6px;
	}

	.ms-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		justify-content: center;
		padding: 8px;
	}

	.ms-pending {
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 0.76rem;
		color: var(--text-secondary);
	}

	.ms-pending a {
		color: var(--accent);
	}

	.ms-code-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.ms-code {
		font-family: monospace;
		font-size: 1rem;
		font-weight: 700;
		letter-spacing: 2px;
		padding: 6px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		-webkit-user-select: text;
		user-select: text;
		cursor: text;
	}

	.copy-code-btn {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 7px 9px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: .7rem;
		font-weight: 700;
		cursor: pointer;
	}

	.copy-code-btn:hover {
		color: var(--accent);
		border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
	}

	:global(.spin) {
		animation: tfl-spin 0.8s linear infinite;
	}

	@keyframes tfl-spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	.java-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.java-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		border: 1px solid var(--border);
		color: var(--text-secondary);
	}

	.java-major {
		flex: 1;
		font-size: 0.8rem;
		color: var(--text-primary);
		font-weight: 600;
	}

	.java-tag {
		font-size: 0.65rem;
		font-weight: 700;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.java-tag.installed {
		color: var(--color-success);
	}
</style>
