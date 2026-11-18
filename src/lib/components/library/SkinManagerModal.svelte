<script lang="ts">
	import StudioAccounts from '$lib/components/refresh/StudioAccounts.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { tr } from '$lib/i18n/experience';

	import { modalFocus } from '$lib/visual/modalFocus';
	import { onMount } from 'svelte';
	import {
		getMojangProfile,
		getCharacterProfile,
		setSkinFromFile,
		resetSkin,
		setActiveCape,
		hideCape,
		pickImageFile
	} from '$lib/api';
	import { appState } from '$lib/state/state.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { SkinViewer3D } from '$lib/components/ui';
	import type { MojangProfile } from '$lib/types/types';
	import { X, Upload, RotateCcw, Loader2, Ban } from 'lucide-svelte';

	let { onClose }: { onClose: () => void } = $props();

	let loadedProfile = $state<{ uuid: string; profile: MojangProfile } | null>(null);
	const profile = $derived(
		loadedProfile?.uuid === appState.currentUser?.uuid ? (loadedProfile?.profile ?? null) : null
	);
	let editableUuid = $state<string | null>(null);
	const canEdit = $derived(!!editableUuid && editableUuid === appState.currentUser?.uuid);
	let requestSequence = 0;
	let loading = $state(true);
	let error = $state<string | null>(null);
	let busy = $state<string | null>(null);
	let tab = $state<'skin' | 'cape'>('skin');
	let variant = $state<'classic' | 'slim'>('classic');

	// Visor 3D a cuerpo completo (pedido del cliente, "que se vea en 3D") —
	// skinview3d es la misma librería que usan NameMC y similares: gratis,
	// MIT, sin cuenta ni suscripción, corre entero en el cliente (three.js),
	// no depende de ningún servicio externo más que la URL de la textura.
	// El ciclo de vida del canvas/viewer (crear, cargar textura, dispose) ya
	// no vive acá — está centralizado en `SkinViewer3D` (Entrega C, sección
	// 1 del encargo), que WelcomeState.svelte también usa. Conserva el zoom
	// del modal (0.85); la orientación sólo cambia por interacción explícita.
	let skinLoadError = $state(false);

	// Subir skin usa el diálogo nativo de archivos, mismo riesgo conocido que
	// ícono/wallpaper propio (ver InstanceIconPicker.svelte) — mismo gate.
	const activeSkin = $derived(profile?.skins.find((s) => s.state === 'ACTIVE') ?? null);
	const activeCape = $derived(profile?.capes.find((c) => c.state === 'ACTIVE') ?? null);

	// Reintentar con otra textura (subir/restaurar/cambiar de cuenta) no debe
	// quedar pegado en un error de carga previo.
	$effect(() => {
		void activeSkin?.url;
		skinLoadError = false;
	});

	async function load(authorize = false) {
		const account = appState.currentUser;
		const sequence = ++requestSequence;
		loadedProfile = null;
		editableUuid = null;
		if (!account) {
			loading = false;
			error = null;
			return;
		}
		loading = true;
		error = null;
		try {
			const cosmetic = await getCharacterProfile(account.uuid);
			if (sequence !== requestSequence || account.uuid !== appState.currentUser?.uuid) return;
			loadedProfile = { uuid: account.uuid, profile: cosmetic };
			// Merely viewing a cached character must never renew authentication.
			// Access to owned skins/capes is requested explicitly by the user.
			if (authorize && account.user_type === 'Microsoft') {
				const owned = await getMojangProfile();
				if (sequence !== requestSequence || account.uuid !== appState.currentUser?.uuid) return;
				loadedProfile = { uuid: account.uuid, profile: owned };
				editableUuid = account.uuid;
			}
			const current = loadedProfile.profile.skins.find((s) => s.state === 'ACTIVE');
			if (current) variant = current.variant.toLowerCase() as 'classic' | 'slim';
		} catch (e) {
			if (sequence === requestSequence) error = String(e);
		} finally {
			if (sequence === requestSequence) loading = false;
		}
	}
	onMount(() => {
		void load();
		return () => {
			requestSequence++;
		};
	});

	async function uploadSkin() {
		if (!canEdit) return;
		const accountUuid = appState.currentUser?.uuid;
		const path = await pickImageFile();
		if (!path || accountUuid !== appState.currentUser?.uuid) return;
		busy = 'upload';
		error = null;
		try {
			await setSkinFromFile(path, variant);
			await load(true);
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
	}

	async function doResetSkin() {
		if (!canEdit) return;
		busy = 'reset';
		error = null;
		try {
			await resetSkin();
			await load(true);
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
	}

	async function chooseCape(capeId: string) {
		if (!canEdit) return;
		busy = capeId;
		error = null;
		try {
			await setActiveCape(capeId);
			await load(true);
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
	}

	async function removeCape() {
		if (!canEdit) return;
		busy = 'none';
		error = null;
		try {
			await hideCape();
			await load(true);
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
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
		class="panel studio-refresh"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		use:modalFocus={onClose}
		aria-modal="true"
		tabindex="-1"
	>
		<div class="panel-header">
			<h2>{vr('studio')}</h2>
			<button type="button" class="close-btn" onclick={onClose} aria-label={t('settings.close')}>
				<X size={16} />
			</button>
		</div>

		<div class="studio-layout">
			<div class="viewer-row">
				<div
					class="viewer-wrap"
					class:account-offline={appState.currentUser?.user_type === 'Cracked'}
				>
					{#if appState.currentUser}
						{#key appState.currentUser.uuid}
							<SkinViewer3D
								width={280}
								height={360}
								zoom={0.85}
								skinUrl={skinLoadError
									? '/skins/steve.png'
									: (activeSkin?.url ?? '/skins/steve.png')}
								model={!skinLoadError && activeSkin?.variant === 'SLIM' ? 'slim' : 'default'}
								capeUrl={activeCape?.url ?? null}
								onSkinError={() => (skinLoadError = true)}
							/>
						{/key}
					{/if}
					{#if loading}
						<div class="viewer-loading"><Loader2 size={20} class="spin" /></div>
					{/if}
				</div>
				{#if skinLoadError}
					<p class="hint viewer-hint error">{t('skinManager.skinLoadError')}</p>
				{:else}
					<p class="hint viewer-hint">{vr('skinPreviewHint')}</p>
				{/if}
			</div>

			<div class="studio-controls">
				<span class="vr-eyebrow">TFL / {vr('account')}</span>
				<h3 class="studio-title">{vr('wardrobe')}</h3>
				<p class="vr-muted">{vr('wardrobeHint')}</p>
				<StudioAccounts onChange={() => load()} />
				{#if error}<p class="error" role="alert">{error}</p>{/if}
				{#if !canEdit}
					<p class="vr-muted">
						{appState.currentUser?.user_type === 'Cracked'
							? tr(
									'Cuenta offline. La skin por nombre es solo una vista cosmética; no cambia el acceso a servidores.',
									'Offline account. A name-based skin is a cosmetic preview; it does not change server access.'
								)
							: tr(
									'La vista del personaje es independiente de tu sesión.',
									'The character preview is independent of your session.'
								)}
					</p>
					{#if appState.currentUser?.user_type === 'Microsoft'}<button
							type="button"
							class="action-btn"
							disabled={loading}
							onclick={() => load(true)}
							>{tr('Administrar skin y capa oficiales', 'Manage official skin and cape')}</button
						>{/if}
				{:else}
					<div class="tabs">
						<button
							type="button"
							class="tab-btn"
							class:active={tab === 'skin'}
							onclick={() => (tab = 'skin')}
						>
							{t('skinManager.tabSkin')}
						</button>
						<button
							type="button"
							class="tab-btn"
							class:active={tab === 'cape'}
							onclick={() => (tab = 'cape')}
						>
							{t('skinManager.tabCape')}
						</button>
					</div>

					{#if !loading}
						{#if tab === 'skin'}
							<div class="skin-controls">
								<div class="variant-tabs">
									<button
										data-pixel-control
										type="button"
										class="variant-btn"
										class:active={variant === 'classic'}
										onclick={() => (variant = 'classic')}
									>
										{t('skinManager.variantClassic')}
									</button>
									<button
										data-pixel-control
										type="button"
										class="variant-btn"
										class:active={variant === 'slim'}
										onclick={() => (variant = 'slim')}
									>
										{t('skinManager.variantSlim')}
									</button>
								</div>
								<button
									data-pixel-control="primary"
									type="button"
									class="action-btn primary"
									disabled={busy === 'upload'}
									onclick={uploadSkin}
								>
									{#if busy === 'upload'}<Loader2 size={14} class="spin" />{:else}<Upload
											size={14}
										/>{/if}
									{t('skinManager.uploadSkin')}
								</button>
								<button
									data-pixel-control
									type="button"
									class="action-btn"
									disabled={busy === 'reset'}
									onclick={doResetSkin}
								>
									{#if busy === 'reset'}<Loader2 size={14} class="spin" />{:else}<RotateCcw
											size={14}
										/>{/if}
									{t('skinManager.resetSkin')}
								</button>
							</div>
						{:else if profile && profile.capes.length === 0}
							<p class="hint">{t('skinManager.noCapes')}</p>
						{:else if profile}
							<div class="cape-grid">
								<button
									type="button"
									class="cape-swatch none"
									class:active={!activeCape}
									disabled={busy === 'none'}
									onclick={removeCape}
								>
									{#if busy === 'none'}<Loader2 size={16} class="spin" />{:else}<Ban
											size={20}
										/>{/if}
									<span>{t('skinManager.noCape')}</span>
								</button>
								{#each profile.capes as cape (cape.id)}
									<button
										type="button"
										class="cape-swatch"
										class:active={cape.state === 'ACTIVE'}
										disabled={busy === cape.id}
										onclick={() => chooseCape(cape.id)}
									>
										{#if busy === cape.id}
											<Loader2 size={16} class="spin" />
										{:else}
											<img src={cape.url} alt={cape.alias ?? ''} />
										{/if}
										<span>{cape.alias ?? '—'}</span>
									</button>
								{/each}
							</div>
						{/if}
					{/if}
				{/if}
			</div>
		</div>
	</div>
</div>

<style>
	.account-offline :global(canvas),
	.account-offline :global(.pixel-person) {
		filter: grayscale(1);
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
		width: min(480px, calc(100vw - 32px));
		max-height: 78vh;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 20px;
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 14px;
		overflow: hidden;
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

	.tabs {
		display: flex;
		gap: 6px;
	}

	.tab-btn {
		flex: 1;
		padding: 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.8rem;
		font-weight: 700;
		cursor: pointer;
	}

	.tab-btn.active {
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		border-color: var(--accent);
		color: var(--text-primary);
	}

	.error {
		color: var(--color-error);
		font-size: 0.78rem;
	}

	.hint {
		font-size: 0.72rem;
		color: var(--text-muted);
	}

	.viewer-row {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
	}

	.viewer-wrap {
		position: relative;
		width: 200px;
		height: 280px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		overflow: hidden;
	}

	.viewer-loading {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		background: color-mix(in srgb, var(--bg-input) 70%, transparent);
	}

	.viewer-hint {
		text-align: center;
	}

	.viewer-hint.error {
		color: var(--color-error);
	}

	.skin-controls {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.variant-tabs {
		display: flex;
		gap: 6px;
	}

	.variant-btn {
		flex: 1;
		padding: 6px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.74rem;
		font-weight: 600;
		cursor: pointer;
	}

	.variant-btn.active {
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		border-color: var(--accent);
		color: var(--text-primary);
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 8px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.78rem;
		font-weight: 600;
		cursor: pointer;
	}

	.action-btn.primary {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		color: var(--accent);
	}

	.action-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.cape-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
		overflow-y: auto;
	}

	.cape-swatch {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 10px 6px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		cursor: pointer;
	}

	.cape-swatch.active {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		color: var(--text-primary);
	}

	.cape-swatch img {
		width: 100%;
		height: 44px;
		object-fit: contain;
		image-rendering: pixelated;
	}

	.cape-swatch.none {
		justify-content: center;
		min-height: 66px;
	}

	.cape-swatch span {
		font-size: 0.68rem;
		text-align: center;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 100%;
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

	.studio-refresh {
		width: min(1060px, calc(100vw - 40px));
		max-height: calc(100vh - 70px);
		padding: 26px;
		overflow: auto;
	}
	.studio-layout {
		display: grid;
		grid-template-columns: minmax(230px, 0.85fr) minmax(0, 1fr);
		gap: 30px;
	}
	.studio-controls {
		display: flex;
		flex-direction: column;
		gap: 22px;
		padding: 10px 0;
	}
	.studio-title {
		font-size: 2rem;
		letter-spacing: -0.04em;
		line-height: 1.15;
		margin: 0;
	}
	.studio-refresh .viewer-row {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 18px;
		min-height: 400px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		background:
			radial-gradient(
				ellipse at 50% 60%,
				color-mix(in srgb, var(--accent) 14%, transparent),
				transparent 70%
			),
			var(--surface-canvas);
	}
	.studio-refresh .viewer-wrap {
		width: 100%;
		height: auto;
		min-height: 320px;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.studio-refresh .panel-header {
		padding-bottom: 20px;
		margin-bottom: 20px;
	}
	.studio-refresh .tabs {
		margin: 0;
	}
	.studio-refresh .skin-controls {
		padding: 0;
	}
	.studio-refresh .viewer-hint {
		padding: 0 18px;
		text-align: center;
	}
	@media (max-width: 850px) {
		.studio-layout {
			grid-template-columns: 220px minmax(0, 1fr);
			gap: 20px;
		}
		.studio-refresh {
			padding: 20px;
		}
		.studio-title {
			font-size: 1.5rem;
		}
		.studio-refresh .viewer-row {
			min-height: 330px;
		}
		.studio-refresh .viewer-wrap :global(canvas) {
			max-width: 200px;
			height: 280px !important;
		}
	}
</style>
