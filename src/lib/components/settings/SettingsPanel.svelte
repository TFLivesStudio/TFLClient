<script lang="ts">
	import IntegrationsSettings from './IntegrationsSettings.svelte';
	import ConsoleSettings from '$lib/components/console/ConsoleSettings.svelte';
	import { tr } from '$lib/i18n/experience';
	import {
		PAGES,
		searchSettings,
		setting,
		type SettingId,
		type SettingsPage
	} from '$lib/settings/catalog';
	import { settingsIntent, openModal } from '$lib/state/uiState.svelte';
	import SettingGroup from './SettingGroup.svelte';
	import AppearanceSettings from './AppearanceSettings.svelte';
	import SceneSettings from './SceneSettings.svelte';
	import WallpaperSettings from './WallpaperSettings.svelte';
	import LaunchSettings from './LaunchSettings.svelte';

	import { modalFocus } from '$lib/visual/modalFocus';
	import { onMount, tick } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { applyQualityVisuals } from '$lib/state/appearanceState.svelte';
	import { appState } from '$lib/state/state.svelte';
	import {
		updateState,
		checkForUpdates,
		downloadUpdate,
		installDownloadedUpdate,
		retryUpdate
	} from '$lib/state/updateState.svelte';
	import { canStartCheck, progressPercent } from '$lib/state/updateMachine';
	import { t, i18nState, setLocale } from '$lib/i18n/index.svelte';
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
	} from '$lib/api';
	import type { QualityProfile, MinecraftUser, JavaStatus, Settings } from '$lib/types/types';
	import { formatVersion } from '$lib/version';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import {
		X,
		Check,
		Trash2,
		Plus,
		Gamepad2,
		Loader2,
		SlidersHorizontal,
		Users,
		Palette,
		WandSparkles,
		Copy,
		CheckCircle2,
		Download,
		RefreshCw,
		Terminal,
		Search
	} from 'lucide-svelte';
	import InfoTooltip from '$lib/components/ui/InfoTooltip.svelte';

	let { onClose }: { onClose: () => void } = $props();

	let tab = $state<SettingsPage>('general');
	const icons = {
		general: SlidersHorizontal,
		appearance: Palette,
		scenes: WandSparkles,
		game: Gamepad2,
		accounts: Users,
		console: Terminal,
		integrations: WandSparkles,
		updates: RefreshCw
	};
	let settingsQuery = $state(''),
		returnQuery = $state(''),
		highlighted = $state<SettingId | null>(null);
	const locale = $derived(i18nState.locale === 'en' ? 1 : 0);
	const currentPage = $derived(PAGES.find((p) => p.id === tab)!);
	const results = $derived(searchSettings(settingsQuery, i18nState.locale));
	let highlightTimer: ReturnType<typeof setTimeout> | undefined;
	async function jump(id: SettingId) {
		returnQuery = settingsQuery || returnQuery;
		settingsQuery = '';
		selectTab(setting(id).page);
		await tick();
		highlighted = id;
		const target = document.querySelector<HTMLElement>(`[data-setting="${id}"]`);
		target?.scrollIntoView({ block: 'start', behavior: 'instant' });
		target?.focus({ preventScroll: true });
		clearTimeout(highlightTimer);
		highlightTimer = setTimeout(() => (highlighted = null), 3500);
	}
	$effect(() => {
		document
			.querySelectorAll('[data-highlighted]')
			.forEach((el) => el.removeAttribute('data-highlighted'));
		if (highlighted)
			document
				.querySelector(`[data-setting="${highlighted}"]`)
				?.setAttribute('data-highlighted', '');
	});
	$effect(() => {
		if (settingsIntent.target) {
			const target = settingsIntent.target;
			settingsIntent.target = null;
			void jump(target);
		}
	});
	onMount(() => () => clearTimeout(highlightTimer));

	let settingsError = $state(''),
		ramMessage = $state('');
	let surface = $state(
		typeof localStorage !== 'undefined'
			? (localStorage.getItem('tfl-surface') ?? 'obsidian')
			: 'obsidian'
	);
	let ambience = $state(
		typeof localStorage !== 'undefined'
			? (localStorage.getItem('tfl-ambience') ?? 'aurora')
			: 'aurora'
	);
	let font = $state(
		typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-font') ?? 'nunito') : 'nunito'
	);
	let density = $state(
		typeof localStorage !== 'undefined'
			? (localStorage.getItem('tfl-density') ?? 'comfortable')
			: 'comfortable'
	);
	let wallpaper = $state(
		typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-wallpaper') ?? 'none') : 'none'
	);
	let cardStyle = $state(
		typeof localStorage !== 'undefined'
			? (localStorage.getItem('tfl-card-style') ?? 'rich')
			: 'rich'
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

	// El chequeo/descarga/instalación viven en el store compartido
	// (updateState): el badge flotante y este panel muestran el mismo estado.
	// Con auto-actualizar apagado, el chequeo manual solo avisa — la descarga
	// la dispara el usuario con el botón.
	const updatePercent = $derived(progressPercent(updateState));

	function handleCheckForUpdate() {
		void checkForUpdates({
			userInitiated: true,
			autoDownload: appState.settings?.auto_updates === true
		});
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

	async function savePatch(patch: Partial<Settings>) {
		if (!appState.settings) return false;
		try {
			const next = { ...appState.settings, ...patch };
			await updateSettings(next);
			appState.settings = next;
			settingsError = '';
			return true;
		} catch (e) {
			settingsError = String(e);
			return false;
		}
	}
	async function setTheme(theme: 'dark' | 'light') {
		if (theme === 'light' && surface === 'oled') return;
		if (await savePatch({ theme })) {
			localStorage.setItem('tfl-theme', theme);
			document.documentElement.setAttribute('data-theme', theme);
		}
	}

	let showBetaWarning = $state(false);

	// Activar las betas siempre pide confirmación (cada vez, no solo la primera):
	// pueden traer errores. Desactivarlas es directo.
	async function toggleBetaUpdates() {
		if (!appState.settings) return;
		if (!appState.settings.beta_updates) {
			showBetaWarning = true;
			return;
		}
		appState.settings.beta_updates = false;
		await updateSettings(appState.settings);
	}

	async function confirmBetaUpdates() {
		showBetaWarning = false;
		if (!appState.settings) return;
		appState.settings.beta_updates = true;
		await updateSettings(appState.settings);
		// Para que se vea enseguida si ya hay una beta disponible.
		handleCheckForUpdate();
	}

	async function toggleAutoUpdates() {
		await savePatch({ auto_updates: !appState.settings?.auto_updates });
	}
	async function setNativeDialogMode(mode: 'auto' | 'manual') {
		await savePatch({ native_dialog_mode: mode, native_dialog_mode_prompted: true });
	}
	async function toggleMultiInstanceWarning() {
		await savePatch({
			multi_instance_warning_dismissed: !appState.settings?.multi_instance_warning_dismissed
		});
	}

	async function handleQuality(profile: QualityProfile) {
		try {
			const updated = await setQualityProfile(profile);
			appState.settings = updated;
			applyQualityVisuals(updated);
			settingsError = '';
		} catch (e) {
			settingsError = String(e);
		}
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
		if (
			!Number.isInteger(minRam) ||
			!Number.isInteger(maxRam) ||
			minRam < 512 ||
			maxRam < minRam ||
			maxRam > 1048576
		) {
			settingsError = tr(
				'Usa valores enteros: mínimo 512 MB y máximo igual o mayor al mínimo.',
				'Use whole numbers: minimum 512 MB and maximum at least the minimum.'
			);
			return;
		}
		savingRam = true;
		settingsError = '';
		ramMessage = '';
		try {
			const next = { ...appState.settings, min_memory: minRam, max_memory: maxRam };
			await updateSettings(next);
			appState.settings = next;
			ramMessage = tr('Memoria guardada', 'Memory saved');
		} catch (e) {
			settingsError = String(e);
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
		try {
			users = await getUserList();
		} catch (e) {
			settingsError = String(e);
		}
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
		try {
			javaStatuses = await getJavaStatus();
		} catch (e) {
			settingsError = String(e);
		}
	}

	function selectTab(t: SettingsPage) {
		tab = t;
		if (t === 'accounts') loadAccounts();
		if (t === 'game') loadJava();
		document.querySelector('.settings-refresh .panel-body')?.scrollTo(0, 0);
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
		class="panel settings-refresh"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		use:modalFocus={onClose}
		aria-modal="true"
		tabindex="-1"
	>
		<div class="panel-header">
			<h2>{t('settings.title')}</h2>
			<button type="button" class="close-btn" onclick={onClose} aria-label={t('settings.close')}>
				<X size={16} />
			</button>
		</div>

		<nav class="panel-tabs" aria-label={t('settings.title')}>
			<label class="settings-search"
				><Search size={14} /><input
					type="search"
					bind:value={settingsQuery}
					placeholder={tr('Buscar…', 'Search…')}
					aria-label={tr('Buscar ajustes', 'Search settings')}
				/></label
			>
			{#each PAGES as page (page.id)}{@const Icon = icons[page.id]}<button
					type="button"
					class="ptab"
					class:active={tab === page.id && !settingsQuery.trim()}
					aria-current={tab === page.id && !settingsQuery.trim() ? 'page' : undefined}
					onclick={() => {
						settingsQuery = '';
						returnQuery = '';
						selectTab(page.id);
					}}><Icon size={16} />{page.label[locale]}</button
				>{/each}
		</nav>
		<div class="panel-body">
			{#if settingsQuery.trim()}
				<header class="settings-page-heading">
					<h3>{tr('Resultados de búsqueda', 'Search results')}</h3>
					<p>
						{results.length} · {tr(
							'Cada resultado abre la opción exacta.',
							'Each result opens the exact setting.'
						)}
					</p>
				</header>
				<div class="settings-results">
					{#each results as result (result.id)}<button
							type="button"
							class="settings-result"
							onclick={() => jump(result.id)}
							><small>{PAGES.find((p) => p.id === result.page)!.label[locale]}</small><strong
								>{result.label[locale]}</strong
							><span>{tr('Abrir opción', 'Open setting')} →</span></button
						>{:else}<p role="status">
							{tr(
								'No encontramos esa opción. Prueba «memoria», «fondo» o «colores».',
								'No matching option. Try “memory”, “background” or “colors”.'
							)}
						</p>{/each}
				</div>
			{:else}
				{#if returnQuery}<button
						type="button"
						class="return-search"
						onclick={() => {
							settingsQuery = returnQuery;
							returnQuery = '';
						}}>← {tr('Volver a resultados', 'Back to results')} · {returnQuery}</button
					>{/if}
				<header class="settings-page-heading">
					<h3>{currentPage.label[locale]}</h3>
					<p>{currentPage.description[locale]}</p>
				</header>
				{#if tab === 'general'}<SettingGroup id="language"
						><section>
							<span class="section-label">{t('settings.language')}</span>
							<div class="row">
								<button
									type="button"
									class="choice"
									class:active={i18nState.locale === 'es'}
									onclick={() => setLocale('es')}>Español</button
								>
								<button
									type="button"
									class="choice"
									class:active={i18nState.locale === 'en'}
									onclick={() => setLocale('en')}>English</button
								>
							</div>
						</section></SettingGroup
					><SettingGroup id="files"
						><section>
							<span class="section-label">
								{t('settings.dialogs.label')}
								<InfoTooltip text={t('settings.dialogs.hint')} />
							</span>
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
						</section></SettingGroup
					><SettingGroup id="storage"
						><section>
							<span class="section-label">{t('settings.storage.label')}</span>
							<button
								type="button"
								class="save-btn"
								disabled={clearingCache}
								onclick={handleClearCache}
							>
								{#if clearingCache}<Loader2 size={13} class="spin" />{:else}<Trash2
										size={13}
									/>{/if}
								{t('settings.storage.clearCache')}
							</button>
							{#if cacheClearedMsg}<p class="hint">{cacheClearedMsg}</p>{/if}
						</section></SettingGroup
					>{:else if tab === 'appearance'}<AppearanceSettings
						onQuality={handleQuality}
						onTheme={setTheme}
						onSurface={applySurface}
						onAccent={applyAccent}
						onFont={applyFont}
						onDensity={applyDensity}
						onCards={applyCardStyle}
						{font}
						{surface}
						{density}
						cards={cardStyle}
					/>
				{:else if tab === 'scenes'}<SceneSettings
						><WallpaperSettings
							{ambience}
							{wallpaper}
							{customWallpaperUrl}
							{customWallpaperBusy}
							{customWallpaperError}
							{dialogsBlocked}
							{applyAmbience}
							{applyWallpaper}
							{selectCustomWallpaper}
							{pickCustomWallpaper}
						/></SceneSettings
					>
				{:else if tab === 'game'}<SettingGroup id="ram"
						><section>
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
							<p class="hint">
								{tr(
									'La RAM residente del sistema también necesita espacio. Los ajustes de cada instancia pueden reemplazar estos valores.',
									'Leave memory available for the operating system. Individual instance settings can override these values.'
								)}
							</p>
							<label class="ram-slider"
								>{tr('Límite máximo (MB)', 'Maximum limit (MB)')}<input
									type="range"
									min="512"
									max={Math.max(ramTotal ?? 16384, maxRam)}
									step="256"
									bind:value={maxRam}
								/><output>{maxRam} MB</output></label
							>{#if ramMessage}<p role="status" class="hint">{ramMessage}</p>{/if}<button
								type="button"
								class="save-btn"
								disabled={savingRam}
								onclick={saveRam}
							>
								<Check size={13} />
								{t('settings.ram.save')}
							</button>
						</section></SettingGroup
					><SettingGroup id="multi-instance"
						><section>
							<span class="section-label">
								{t('settings.multiInstance.label')}
								<InfoTooltip text={t('settings.multiInstance.hint')} />
							</span>
							<button
								type="button"
								class="choice auto-update-toggle"
								class:active={!appState.settings?.multi_instance_warning_dismissed}
								onclick={toggleMultiInstanceWarning}
							>
								{appState.settings?.multi_instance_warning_dismissed
									? t('settings.multiInstance.off')
									: t('settings.multiInstance.on')}
							</button>
						</section></SettingGroup
					><LaunchSettings
						statuses={javaStatuses}
						onSaved={loadJava}
					/>{:else if tab === 'accounts'}<SettingGroup id="accounts"
						><section>
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
						</section></SettingGroup
					><SettingGroup id="account-microsoft"
						><section>
							<span class="section-label">{t('settings.accounts.addMicrosoft')}</span>
							{#if msCode}
								<div class="ms-pending">
									<p>
										{t('onboarding.goTo')}
										<a href={msVerificationUri} target="_blank" rel="noreferrer"
											>{msVerificationUri}</a
										>
										{t('settings.accounts.microsoftEnter')}
									</p>
									<div class="ms-code-row">
										<code class="ms-code" tabindex="0">{msCode}</code>
										<button
											type="button"
											class="copy-code-btn"
											onclick={copyMicrosoftCode}
											aria-label={t('settings.accounts.copyCode')}
										>
											{#if codeCopied}<CheckCircle2 size={14} /> {t('common.copied')}{:else}<Copy
													size={14}
												/>
												{t('common.copy')}{/if}
										</button>
									</div>
									<p class="hint">{t('settings.accounts.waitingConfirmation')}</p>
								</div>
							{:else}
								<button
									type="button"
									class="mini-btn ms-btn"
									disabled={msBusy}
									onclick={handleAddMicrosoft}
								>
									{#if msBusy}<Loader2 size={13} class="spin" />{:else}<Gamepad2 size={13} />{/if}
									{t('settings.accounts.startMicrosoft')}
								</button>
							{/if}
							{#if msError}<p class="error">{msError}</p>{/if}
						</section></SettingGroup
					><SettingGroup id="account-offline"
						><section>
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
						</section></SettingGroup
					><SettingGroup
						id="account-skin"
						hint={tr(
							'Abre el estudio para gestionar la skin de tu cuenta activa. Las mascotas de cuentas sin conexión se eligen en la lista de cuentas.',
							'Open the studio to manage your active account skin. Choose offline mascots in the account list.'
						)}
						><button
							type="button"
							class="save-btn"
							onclick={() => {
								onClose();
								openModal('skinManager');
							}}>{tr('Abrir estudio de skins', 'Open skin studio')}</button
						></SettingGroup
					>{:else if tab === 'console'}<ConsoleSettings
					/>{:else if tab === 'integrations'}<IntegrationsSettings
					/>{:else if tab === 'updates'}<SettingGroup id="updates"
						><section>
							<span class="section-label">
								{t('settings.updates.label')}
								<InfoTooltip text={t('settings.updates.hint')} />
							</span>
							<button
								type="button"
								class="choice auto-update-toggle"
								class:active={appState.settings?.auto_updates}
								onclick={toggleAutoUpdates}
							>
								{appState.settings?.auto_updates
									? t('settings.updates.autoEnabled')
									: t('settings.updates.autoDisabled')}
							</button>
							<button
								type="button"
								class="choice auto-update-toggle"
								class:active={appState.settings?.beta_updates}
								onclick={toggleBetaUpdates}
							>
								{appState.settings?.beta_updates
									? t('settings.updates.betaEnabled')
									: t('settings.updates.betaDisabled')}
							</button>
							<p class="hint">{t('settings.updates.betaHint')}</p>
							<p class="hint">
								{t('settings.updates.currentVersion', { version: formatVersion(__APP_VERSION__) })}
							</p>
							<button
								type="button"
								class="save-btn"
								disabled={!canStartCheck(updateState)}
								onclick={handleCheckForUpdate}
							>
								{#if updateState.phase === 'checking'}
									<Loader2 size={13} class="spin" />
								{:else}
									<RefreshCw size={13} />
								{/if}
								{t('settings.updates.check')}
							</button>

							{#if updateState.phase === 'none' && updateState.upToDate}
								<p class="hint">{t('settings.updates.upToDate')}</p>
							{/if}

							{#if updateState.phase === 'available'}
								<p class="hint">
									{t('settings.updates.available', {
										version: formatVersion(updateState.version ?? '')
									})}
								</p>
								<button type="button" class="save-btn" onclick={downloadUpdate}>
									<Download size={13} />
									{t('settings.updates.download')}
								</button>
							{:else if updateState.phase === 'downloading'}
								<p class="hint">
									{updatePercent === null
										? t('settings.updates.downloadingUnknown')
										: t('settings.updates.downloading', { percent: updatePercent })}
								</p>
							{:else if updateState.phase === 'downloaded'}
								<p class="hint">
									{t('settings.updates.ready', {
										version: formatVersion(updateState.version ?? '')
									})}
								</p>
								<button type="button" class="save-btn" onclick={installDownloadedUpdate}>
									<RefreshCw size={13} />
									{t('settings.updates.restart')}
								</button>
							{:else if updateState.phase === 'installing'}
								<p class="hint">{t('settings.updates.installing')}</p>
							{:else if updateState.phase === 'error' && updateState.error}
								<p class="error">{updateState.error.message}</p>
								<button type="button" class="save-btn" onclick={retryUpdate}>
									<RefreshCw size={13} />
									{t('settings.updates.retry')}
								</button>
							{/if}
						</section></SettingGroup
					>{/if}{#if settingsError}<p class="error" role="alert">{settingsError}</p>{/if}{/if}
		</div>
	</div>
</div>

{#if showBetaWarning}
	<ConfirmDialog
		title={t('settings.updates.betaWarningTitle')}
		message={t('settings.updates.betaWarningMessage')}
		confirmLabel={t('settings.updates.betaWarningConfirm')}
		danger
		onConfirm={confirmBetaUpdates}
		onCancel={() => (showBetaWarning = false)}
	/>
{/if}

<style>
	.settings-search {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px;
		margin-bottom: 10px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		color: var(--text-muted);
	}
	.settings-search input {
		background: transparent;
		border: 0;
		color: var(--text-primary);
		width: 100%;
		min-width: 0;
		font-size: 12px;
		outline-offset: 4px;
	}
	.settings-page-heading {
		border-bottom: 1px solid var(--border);
		padding-bottom: 20px;
	}
	.settings-page-heading h3 {
		font-size: 22px;
		margin: 0 0 8px;
		letter-spacing: -0.03em;
	}
	.settings-page-heading p {
		color: var(--text-secondary);
		font-size: 13px;
		line-height: 1.7;
		max-width: 66ch;
		margin: 0;
	}

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
		width: min(620px, calc(100vw - 32px));
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
		display: flex;
		align-items: center;
		gap: 5px;
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
		opacity: 0.42;
		cursor: not-allowed;
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
		font-size: 0.7rem;
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

	.settings-refresh {
		width: min(1180px, calc(100vw - 40px));
		height: calc(100vh - 76px);
		max-height: none;
		display: grid;
		grid-template-columns: 215px minmax(0, 1fr);
		grid-template-rows: auto minmax(0, 1fr);
		gap: 0;
		padding: 0;
		overflow: hidden;
		background: var(--surface-elevated);
	}
	.settings-refresh .panel-header {
		grid-column: 1/-1;
		padding: 22px 28px;
		border-bottom: 1px solid var(--border);
	}
	.settings-refresh .panel-tabs {
		flex-direction: column;
		border-bottom: 0;
		border-right: 1px solid var(--border);
		padding: 22px 12px;
		gap: 6px;
		overflow: auto;
		background: var(--surface-workspace);
	}
	.settings-refresh .ptab {
		padding: 13px 14px;
		margin: 0;
		border: 1px solid transparent;
		border-radius: var(--border-radius-sm);
		font-size: var(--text-small);
		gap: 12px;
		text-align: left;
	}
	.settings-refresh .ptab.active {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
		border-color: color-mix(in srgb, var(--accent) 20%, transparent);
	}
	.settings-refresh .panel-body {
		padding: 28px;
		min-height: 0;
		gap: 26px;
	}
	.settings-refresh .section-label {
		font-size: var(--text-small);
		line-height: 1.5;
		margin-bottom: 10px;
	}
	.overlay {
		backdrop-filter: blur(10px);
	}
	@media (max-width: 850px) {
		.settings-refresh {
			grid-template-columns: 155px minmax(0, 1fr);
			width: calc(100vw - 24px);
			height: calc(100vh - 60px);
		}
		.settings-refresh .panel-body {
			padding: 20px;
		}
		.settings-refresh .ptab {
			padding: 12px 8px;
			font-size: 12px;
		}
	}
	.settings-results {
		display: grid;
		gap: 10px;
	}
	.settings-result {
		display: flex;
		flex-direction: column;
		gap: 7px;
		text-align: left;
		padding: 14px 16px;
		background: var(--surface-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		color: var(--text-primary);
		cursor: pointer;
	}
	.settings-result:hover {
		border-color: var(--accent);
	}
	.settings-result small,
	.settings-result span {
		font-size: 11px;
		color: var(--text-secondary);
	}
	.settings-result strong {
		font-size: 14px;
	}
	.return-search {
		border: 0;
		background: none;
		color: var(--accent);
		font-size: 12px;
		cursor: pointer;
		padding: 0 0 16px;
	}

	.ram-slider {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 12px;
		margin: 16px 0;
	}
	.ram-slider input {
		flex: 1;
		min-width: 60px;
		accent-color: var(--accent);
	}
	.panel-body > :global(.setting-group) {
		margin-bottom: 26px;
	}
</style>
