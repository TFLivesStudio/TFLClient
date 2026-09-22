<script lang="ts">
	import { onMount } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { check as checkForUpdate, type Update } from '@tauri-apps/plugin-updater';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { appState } from '$lib/state/state.svelte';
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
	const ACCENTS = [
		{ id: 'orange', label: 'Naranja', color: '#ff7a2e' },
		{ id: 'violet', label: 'Violeta', color: '#8b5cf6' },
		{ id: 'teal', label: 'Verde azulado', color: '#14b8a6' },
		{ id: 'blue', label: 'Azul eléctrico', color: '#3b82f6' },
		{ id: 'rose', label: 'Rosa plasma', color: '#f43f5e' },
		{ id: 'lime', label: 'Lima', color: '#84cc16' },
		{ id: 'iris', label: 'Iris', color: '#5b5bd6' },
		{ id: 'jade', label: 'Jade', color: '#29a383' },
		{ id: 'crimson', label: 'Carmesí', color: '#e93d82' },
		{ id: 'cyan', label: 'Cian', color: '#00a2c7' },
		{ id: 'grass', label: 'Césped', color: '#46a758' },
		{ id: 'plum', label: 'Ciruela', color: '#ab4aba' }
	];
	const SURFACES = [
		{ id: 'obsidian', label: 'Obsidiana' },
		{ id: 'midnight', label: 'Medianoche' },
		{ id: 'slate', label: 'Pizarra' },
		{ id: 'oled', label: 'OLED' }
	];
	const AMBIENCES = [
		{ id: 'aurora', label: 'Aurora' },
		{ id: 'cosmic', label: 'Cósmico' },
		{ id: 'minimal', label: 'Minimal' },
		{ id: 'particles', label: 'Partículas' }
	];
	const WALLPAPERS = [
		{ id: 'none', label: 'Ninguno', preview: 'transparent' },
		{ id: 'void-night', label: 'Noche vacía', preview: 'linear-gradient(135deg, #02040a, #050d1a, #030810)' },
		{ id: 'nether', label: 'Nether', preview: 'linear-gradient(135deg, #140202, #250808, #0e0101)' },
		{ id: 'end', label: 'El End', preview: 'linear-gradient(135deg, #070212, #0e0520, #040110)' },
		{ id: 'deep-ocean', label: 'Océano', preview: 'linear-gradient(135deg, #010f1a, #021a2e, #010c14)' },
		{
			id: 'aurora-gradient',
			label: 'Aurora',
			preview: 'linear-gradient(135deg, #040d10, #041410 40%, #0a0420 80%, #04100d)'
		},
		{
			id: 'animated-aurora',
			label: 'Aurora animada',
			preview: 'linear-gradient(135deg, #040d10, #041410 40%, #0a0420 80%, #04100d)'
		},
		{ id: 'obsidian-solid', label: 'Obsidiana', preview: '#08090c' },
		{ id: 'charcoal', label: 'Carbón', preview: '#0f1115' },
		{ id: 'savanna', label: 'Sabana', preview: 'url(/wallpapers/mc-wallpaper-1.jpg)' },
		{ id: 'golden-sunset', label: 'Atardecer dorado', preview: 'url(/wallpapers/mc-wallpaper-2.jpg)' },
		{ id: 'lake-night', label: 'Noche en el lago', preview: 'url(/wallpapers/mc-wallpaper-3.jpg)' },
		{ id: 'neon-arcade', label: 'Arcade nocturno', preview: 'url(/wallpapers/mc-wallpaper-4.jpg)' },
		{ id: 'red-canyon', label: 'Cañón rojo', preview: 'url(/wallpapers/mc-wallpaper-5.jpg)' },
		{ id: 'enchanted-valley', label: 'Valle encantado', preview: 'url(/wallpapers/mc-wallpaper-6.jpg)' },
		{ id: 'snowy-peak', label: 'Cumbre nevada', preview: 'url(/wallpapers/mc-wallpaper-7.jpg)' },
		{ id: 'stone-bridge', label: 'Puente de piedra', preview: 'url(/wallpapers/mc-wallpaper-8.jpg)' },
		{ id: 'misty-fortress', label: 'Fortaleza en la niebla', preview: 'url(/wallpapers/mc-wallpaper-9.jpg)' },
		{ id: 'village-tower', label: 'Torre del pueblo', preview: 'url(/wallpapers/mc-wallpaper-10.jpg)' },
		{ id: 'deep-cave', label: 'Cueva profunda', preview: 'url(/wallpapers/mc-wallpaper-11.jpg)' },
		{ id: 'sunset-coast', label: 'Costa al atardecer', preview: 'url(/wallpapers/mc-wallpaper-12.jpg)' },
		{ id: 'abstract-blocks', label: 'Cubos abstractos', preview: 'url(/wallpapers/mc-wallpaper-13.jpg)' }
	];

	let tab = $state<'general' | 'appearance' | 'accounts' | 'java'>('general');

	let accent = $state(
		typeof localStorage !== 'undefined'
			? (localStorage.getItem('tfl-accent') ?? 'orange')
			: 'orange'
	);
	let surface = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-surface') ?? 'obsidian') : 'obsidian');
	let ambience = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-ambience') ?? 'aurora') : 'aurora');
	let density = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-density') ?? 'comfortable') : 'comfortable');
	let wallpaper = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-wallpaper') ?? 'none') : 'none');
	let cardStyle = $state(
		typeof localStorage !== 'undefined' ? (localStorage.getItem('tfl-card-style') ?? 'rich') : 'rich'
	);
	let customWallpaperUrl = $state<string | null>(null);
	let customWallpaperBusy = $state(false);
	let customWallpaperError = $state<string | null>(null);

	// Subir imagen propia crashea el launcher entero en macOS (bug conocido
	// del diálogo nativo de archivos con la firma ad-hoc del build, ver
	// CHANGELOG_macos-dialog-crash-java26.txt) — bloqueado acá hasta tener
	// una firma real de Apple Developer. El resto de la app no se toca.
	const isMacOS = typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform);

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
		if (isMacOS) return;
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
			msError = 'No se pudo copiar. Seleccioná el código y copialo manualmente.';
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
			cacheClearedMsg = `Liberados ${freedMb} MB.`;
		} catch (e) {
			cacheClearedMsg = `No se pudo limpiar: ${e}`;
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
			<h2>Ajustes</h2>
			<button type="button" class="close-btn" onclick={onClose} aria-label="Cerrar">
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
				<Palette size={14} /> Estilo
			</button>
			<button
				type="button"
				class="ptab"
				class:active={tab === 'general'}
				onclick={() => selectTab('general')}
			>
				<SlidersHorizontal size={14} /> General
			</button>
			<button
				type="button"
				class="ptab"
				class:active={tab === 'accounts'}
				onclick={() => selectTab('accounts')}
			>
				<Users size={14} /> Cuentas
			</button>
			<button
				type="button"
				class="ptab"
				class:active={tab === 'java'}
				onclick={() => selectTab('java')}
			>
				<Coffee size={14} /> Java
			</button>
		</div>

		<div class="panel-body">
			{#if tab === 'general'}
				<section>
					<span class="section-label">Tema</span>
					<div class="row">
						<button
							type="button"
							class="choice"
							class:active={appState.settings?.theme === 'dark'}
							onclick={() => setTheme('dark')}>Oscuro</button
						>
						<button
							type="button"
							class="choice"
							class:active={appState.settings?.theme === 'light'}
							disabled={surface === 'oled'}
							title={surface === 'oled' ? 'Modo claro no compatible con superficie OLED' : undefined}
							onclick={() => setTheme('light')}>Claro</button
						>
					</div>
				</section>

				<section>
					<span class="section-label">Perfil de calidad</span>
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
						Memoria (RAM) global
						{#if ramTotal}<span class="ram-total"
								>— {Math.round(ramTotal / 1024)} GB detectados</span
							>{/if}
					</span>
					<div class="ram-row">
						<label>
							Mínima (MB)
							<input type="number" bind:value={minRam} min="512" step="256" />
						</label>
						<label>
							Máxima (MB)
							<input type="number" bind:value={maxRam} min="512" step="256" />
						</label>
					</div>
					<button type="button" class="save-btn" disabled={savingRam} onclick={saveRam}>
						<Check size={13} /> Guardar
					</button>
				</section>

				<section>
					<span class="section-label">Almacenamiento</span>
					<button type="button" class="save-btn" disabled={clearingCache} onclick={handleClearCache}>
						{#if clearingCache}<Loader2 size={13} class="spin" />{:else}<Trash2 size={13} />{/if}
						Limpiar caché temporal
					</button>
					{#if cacheClearedMsg}<p class="hint">{cacheClearedMsg}</p>{/if}
				</section>

				<section>
					<span class="section-label">Actualizaciones</span>
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
						Buscar actualizaciones
					</button>

					{#if updateChecked && !updateInfo && !updateError}
						<p class="hint">Ya tenés la última versión.</p>
					{/if}

					{#if updateInfo}
						<p class="hint">Actualización disponible: v{updateInfo.version}</p>
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
							Descargar e instalar
						</button>
					{/if}

					{#if updateError}<p class="error">{updateError}</p>{/if}
				</section>
			{:else if tab === 'appearance'}
				<section>
					<span class="section-label">Color de acento</span>
					<div class="swatches">
						{#each ACCENTS as a (a.id)}
							<button type="button" class="swatch" class:active={accent === a.id} style="background: {a.color}; color: {a.color}" onclick={() => applyAccent(a.id)} aria-label={a.label}></button>
						{/each}
					</div>
				</section>
				<section>
					<span class="section-label"><PanelLeft size={13} /> Superficie</span>
					<div class="row">{#each SURFACES as item (item.id)}<button type="button" class="choice" class:active={surface === item.id} disabled={item.id === 'oled' && appState.settings?.theme === 'light'} title={item.id === 'oled' && appState.settings?.theme === 'light' ? 'Superficie OLED no compatible con modo claro' : undefined} onclick={() => applySurface(item.id)}>{item.label}</button>{/each}</div>
				</section>
				<section>
					<span class="section-label"><WandSparkles size={13} /> Efecto ambiental</span>
					<div class="row">{#each AMBIENCES as item (item.id)}<button type="button" class="choice" class:active={ambience === item.id} onclick={() => applyAmbience(item.id)}>{item.label}</button>{/each}</div>
				</section>
				<section>
					<span class="section-label"><WandSparkles size={13} /> Fondo de pantalla</span>
					<div class="wallpaper-grid">
						{#each WALLPAPERS as item (item.id)}
							<button
								type="button"
								class="wallpaper-swatch"
								class:active={wallpaper === item.id}
								style="background: {item.preview}; background-size: cover; background-position: center;"
								onclick={() => applyWallpaper(item.id)}
								aria-label={item.label}
								title={item.label}
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
								aria-label="Tu imagen"
								title="Tu imagen"
							>
								{#if wallpaper === 'custom'}<Check size={12} />{/if}
							</button>
						{/if}
						<button
							type="button"
							class="wallpaper-swatch wallpaper-upload"
							onclick={pickCustomWallpaper}
							disabled={customWallpaperBusy || isMacOS}
							aria-label={customWallpaperUrl ? 'Cambiar tu imagen' : 'Subir tu imagen'}
							title={isMacOS
								? 'No disponible en macOS por ahora'
								: customWallpaperUrl
									? 'Cambiar tu imagen'
									: 'Subir tu imagen'}
						>
							<Upload size={13} />
						</button>
					</div>
					{#if isMacOS}<p class="mac-notice">Subir tu propia imagen no está disponible en macOS por ahora.</p>{/if}
					{#if customWallpaperError}<p class="error-text">{customWallpaperError}</p>{/if}
				</section>
				<section>
					<span class="section-label">Densidad de interfaz</span>
					<div class="row"><button type="button" class="choice" class:active={density === 'comfortable'} onclick={() => applyDensity('comfortable')}>Cómoda</button><button type="button" class="choice" class:active={density === 'compact'} onclick={() => applyDensity('compact')}>Compacta</button></div>
				</section>
				<section>
					<span class="section-label">Estilo de tarjeta</span>
					<div class="row">
						<button type="button" class="choice" class:active={cardStyle === 'rich'} onclick={() => applyCardStyle('rich')}>Rica</button>
						<button type="button" class="choice" class:active={cardStyle === 'minimal'} onclick={() => applyCardStyle('minimal')}>Minimal</button>
					</div>
				</section>
			{:else if tab === 'accounts'}
				<section>
					<span class="section-label">Cuentas guardadas</span>
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
										Usar
									</button>
								{:else}
									<span class="active-tag">Activa</span>
								{/if}
								<button
									type="button"
									class="mini-icon-btn"
									disabled={accountBusy}
									onclick={() => handleRemove(u.uuid)}
									aria-label="Quitar cuenta"
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
					<span class="section-label">Agregar cuenta Microsoft</span>
					{#if msCode}
						<div class="ms-pending">
							<p>Andá a <a href={msVerificationUri} target="_blank" rel="noreferrer">{msVerificationUri}</a> e ingresá:</p>
						<div class="ms-code-row">
							<code class="ms-code" tabindex="0">{msCode}</code>
							<button type="button" class="copy-code-btn" onclick={copyMicrosoftCode} aria-label="Copiar código">
								{#if codeCopied}<CheckCircle2 size={14} /> Copiado{:else}<Copy size={14} /> Copiar{/if}
							</button>
						</div>
							<p class="hint">Esperando confirmación…</p>
						</div>
					{:else}
						<button type="button" class="mini-btn ms-btn" disabled={msBusy} onclick={handleAddMicrosoft}>
							{#if msBusy}<Loader2 size={13} class="spin" />{:else}<Gamepad2 size={13} />{/if}
							Iniciar con Microsoft
						</button>
					{/if}
					{#if msError}<p class="error">{msError}</p>{/if}
				</section>

				<section>
					<span class="section-label">Agregar cuenta offline</span>
					<div class="add-account-row">
						<input
							type="text"
							bind:value={newOfflineName}
							placeholder="Nombre de usuario"
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
					<p class="hint">Las cuentas offline solo pueden jugar en singleplayer.</p>
				</section>
			{:else if tab === 'java'}
				<section>
					<span class="section-label">Runtimes de Java</span>
					<p class="hint">
						Se instalan solos la primera vez que una instancia los necesita — no hace falta instalar
						Java a mano.
					</p>
					<div class="java-list">
						{#each javaStatuses as j (j.major)}
							<div class="java-row">
								<Coffee size={14} />
								<span class="java-major">Java {j.major}</span>
								{#if j.installed}
									<span class="java-tag installed">Instalado</span>
								{:else}
									<span class="java-tag">No instalado</span>
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
