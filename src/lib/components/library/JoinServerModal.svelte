<script lang="ts">
	import {
		getAvailableVersions,
		createInstance,
		launchInstance,
		getFavoriteServers,
		addFavoriteServer,
		removeFavoriteServer,
		pingServer
	} from '$lib/api/tflApi';
	import type {
		InstanceData,
		MinecraftUser,
		MinecraftVersion,
		FavoriteServer,
		ServerStatus
	} from '$lib/types/types';
	import { t } from '$lib/i18n/index.svelte';
	import { Loader2, Star, Trash2, Play } from 'lucide-svelte';

	let {
		instances,
		currentUser,
		onClose,
		onLaunched
	}: {
		instances: InstanceData[];
		currentUser: MinecraftUser | null;
		onClose: () => void;
		onLaunched: () => void;
	} = $props();

	// Unirse a un servidor es multijugador — las cuentas offline/cracked no
	// pueden (ver [[project_gameplay_rules]] / account_policy.rs, misma
	// regla ya aplicada del lado del backend, esto es solo para no dejar
	// ni intentarlo desde acá).
	const blocked = $derived(currentUser?.user_type === 'Cracked');

	// Solo instancias de cliente — una instancia de servidor no tiene
	// sentido "unirla" a otro servidor.
	const joinableInstances = $derived(instances.filter((i) => !i.server_type));

	let mode = $state<'existing' | 'quick'>('quick');
	let address = $state('');
	let selectedInstance = $state('');
	let modeInitialized = $state(false);
	$effect(() => {
		if (modeInitialized) return;
		if (joinableInstances.length > 0) {
			mode = 'existing';
			selectedInstance = joinableInstances[0].name;
		}
		modeInitialized = true;
	});

	let allVersions = $state<MinecraftVersion[]>([]);
	let selectedVersion = $state('');
	let loadingVersions = $state(true);
	const releaseVersions = $derived(allVersions.filter((v) => v.type === 'release'));

	// Quick Play (--quickPlayMultiplayer, lo que conecta directo) lo agregó
	// Mojang recién en 1.20.2 — versiones anteriores ni entienden el flag,
	// así que el juego abre normal en vez de conectar. El versionado nuevo
	// post-renombre (ej. "26.3", sin el "1." adelante) es todo posterior,
	// siempre soportado.
	function supportsQuickPlay(versionId: string): boolean {
		const parts = versionId.split('.').map(Number);
		if (parts[0] !== 1) return true;
		const minor = parts[1] ?? 0;
		const patch = parts[2] ?? 0;
		if (minor !== 20) return minor > 20;
		return patch >= 2;
	}

	const targetVersion = $derived(
		mode === 'existing' ? joinableInstances.find((i) => i.name === selectedInstance)?.mc_version : selectedVersion
	);
	const quickPlayUnsupported = $derived(!!targetVersion && !supportsQuickPlay(targetVersion));

	let joining = $state(false);
	let error = $state<string | null>(null);

	getAvailableVersions()
		.then((v) => {
			allVersions = v;
			selectedVersion = v.find((x) => x.type === 'release')?.id ?? v[0]?.id ?? '';
		})
		.finally(() => (loadingVersions = false));

	// ── Favoritos ───────────────────────────────────────────────────────
	// Lista guardada con el estado de cada servidor (se consulta al abrir el
	// modal, en paralelo): así se ve cuáles están arriba y cuánta gente hay
	// antes de unirse, y un click los carga.
	let favorites = $state<FavoriteServer[]>([]);
	let statuses = $state<Record<string, ServerStatus | 'loading'>>({});
	let savingFavorite = $state(false);
	let favoriteName = $state('');
	let favoriteError = $state<string | null>(null);

	async function refreshStatus(fav: FavoriteServer) {
		statuses[fav.id] = 'loading';
		try {
			statuses[fav.id] = await pingServer(fav.address);
		} catch {
			statuses[fav.id] = { online: false, motd: null, players_online: null, players_max: null, version: null, latency_ms: null };
		}
	}

	async function loadFavorites() {
		try {
			favorites = await getFavoriteServers();
			favorites.forEach((f) => void refreshStatus(f));
		} catch {
			favorites = [];
		}
	}
	loadFavorites();

	const alreadyFavorite = $derived(
		favorites.some((f) => f.address.toLowerCase() === address.trim().toLowerCase())
	);

	async function handleSaveFavorite() {
		favoriteError = null;
		try {
			const added = await addFavoriteServer(favoriteName, address.trim());
			favorites = [...favorites, added];
			savingFavorite = false;
			favoriteName = '';
			void refreshStatus(added);
		} catch (e) {
			favoriteError = String(e);
		}
	}

	async function handleRemoveFavorite(fav: FavoriteServer) {
		try {
			await removeFavoriteServer(fav.id);
			favorites = favorites.filter((f) => f.id !== fav.id);
		} catch (e) {
			favoriteError = String(e);
		}
	}

	async function joinFavorite(fav: FavoriteServer) {
		address = fav.address;
		await handleJoin();
	}

	async function handleJoin() {
		const trimmed = address.trim();
		if (!trimmed || blocked) return;
		joining = true;
		error = null;
		try {
			if (mode === 'existing') {
				if (!selectedInstance) return;
				await launchInstance(selectedInstance, trimmed);
			} else {
				if (!selectedVersion) return;
				// Reusa una instancia Vanilla que ya tenga esta versión en vez
				// de crear una nueva cada vez que se hace quick join.
				const existing = instances.find((i) => i.loader === 'vanilla' && i.mc_version === selectedVersion && !i.server_type);
				const instanceName = existing?.name ?? (await createInstance(`Quick Join ${selectedVersion}`, selectedVersion, 'vanilla')).name;
				await launchInstance(instanceName, trimmed);
			}
			onLaunched();
		} catch (e) {
			error = String(e);
		} finally {
			joining = false;
		}
	}
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && !joining && onClose()} />

<div
	class="overlay"
	onclick={() => !joining && onClose()}
	onkeydown={(e) => e.key === 'Escape' && !joining && onClose()}
	role="button"
	tabindex="-1"
>
	<div
		class="modal"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<h2>{t('joinServer.title')}</h2>

		{#if blocked}
			<p class="error">{t('joinServer.crackedBlocked')}</p>
		{:else}
			{#if favorites.length > 0}
				<span class="fav-label">{t('joinServer.favorites')}</span>
				<div class="fav-list">
					{#each favorites as fav (fav.id)}
						{@const st = statuses[fav.id]}
						<div class="fav-row">
							<button type="button" class="fav-main" disabled={joining} onclick={() => (address = fav.address)}>
								<span
									class="dot"
									class:online={st !== undefined && st !== 'loading' && st.online}
									class:loading={st === undefined || st === 'loading'}
								></span>
								<span class="fav-text">
									<span class="fav-name">{fav.name}</span>
									<span class="fav-sub">
										{#if st === undefined || st === 'loading'}
											{fav.address}
										{:else if st.online}
											{st.motd ?? fav.address}
										{:else}
											{t('joinServer.offline')} · {fav.address}
										{/if}
									</span>
								</span>
								{#if st && st !== 'loading' && st.online}
									<span class="fav-meta">
										{#if st.players_online !== null}{st.players_online}{#if st.players_max !== null}/{st.players_max}{/if}{/if}
										{#if st.latency_ms !== null}<small>{st.latency_ms} ms</small>{/if}
									</span>
								{/if}
							</button>
							<button
								type="button"
								class="fav-icon"
								disabled={joining || blocked}
								onclick={() => joinFavorite(fav)}
								title={t('joinServer.join')}
								aria-label={t('joinServer.join')}
							>
								<Play size={13} />
							</button>
							<button
								type="button"
								class="fav-icon danger"
								disabled={joining}
								onclick={() => handleRemoveFavorite(fav)}
								title={t('joinServer.removeFavorite')}
								aria-label={t('joinServer.removeFavorite')}
							>
								<Trash2 size={13} />
							</button>
						</div>
					{/each}
				</div>
			{/if}

			<label for="join-address">{t('joinServer.addressLabel')}</label>
			<div class="address-row">
				<input
					id="join-address"
					type="text"
					bind:value={address}
					placeholder={t('joinServer.addressPlaceholder')}
					autocomplete="off"
					disabled={joining}
				/>
				<button
					type="button"
					class="star-btn"
					disabled={joining || !address.trim() || alreadyFavorite}
					title={alreadyFavorite ? t('joinServer.alreadyFavorite') : t('joinServer.saveFavorite')}
					aria-label={t('joinServer.saveFavorite')}
					onclick={() => (savingFavorite = !savingFavorite)}
				>
					<Star size={15} fill={alreadyFavorite ? 'currentColor' : 'none'} />
				</button>
			</div>
			{#if savingFavorite && !alreadyFavorite}
				<div class="address-row">
					<input
						type="text"
						bind:value={favoriteName}
						placeholder={t('joinServer.favoriteNamePlaceholder')}
						maxlength="40"
						onkeydown={(e) => e.key === 'Enter' && handleSaveFavorite()}
					/>
					<button type="button" class="btn" onclick={handleSaveFavorite}>{t('joinServer.saveFavoriteConfirm')}</button>
				</div>
			{/if}
			{#if favoriteError}<p class="error">{favoriteError}</p>{/if}

			<div class="mode-tabs">
				<button
					type="button"
					class="mode-tab"
					class:active={mode === 'existing'}
					disabled={joining || joinableInstances.length === 0}
					onclick={() => (mode = 'existing')}
				>
					{t('joinServer.modeExisting')}
				</button>
				<button
					type="button"
					class="mode-tab"
					class:active={mode === 'quick'}
					disabled={joining}
					onclick={() => (mode = 'quick')}
				>
					{t('joinServer.modeQuick')}
				</button>
			</div>

			{#if mode === 'existing'}
				{#if joinableInstances.length === 0}
					<p class="hint">{t('joinServer.noInstances')}</p>
				{:else}
					<select bind:value={selectedInstance} disabled={joining}>
						{#each joinableInstances as inst (inst.uuid)}
							<option value={inst.name}>{inst.name} · {inst.mc_version}</option>
						{/each}
					</select>
				{/if}
			{:else if loadingVersions}
				<div class="loading">
					<Loader2 size={16} class="spin" /> {t('createInstance.loadingVersions')}
				</div>
			{:else}
				<select bind:value={selectedVersion} disabled={joining}>
					{#each releaseVersions as v (v.id)}
						<option value={v.id}>{v.id}</option>
					{/each}
				</select>
				<p class="hint">{t('joinServer.quickHint')}</p>
			{/if}

			{#if quickPlayUnsupported}
				<p class="hint warn">{t('joinServer.quickPlayUnsupported')}</p>
			{/if}

			{#if error}
				<p class="error">{error}</p>
			{/if}
		{/if}

		<div class="actions">
			<button type="button" class="btn" onclick={onClose} disabled={joining}>{t('common.cancel')}</button>
			{#if !blocked}
				<button
					type="button"
					class="btn primary"
					disabled={joining ||
						!address.trim() ||
						(mode === 'existing' ? !selectedInstance : !selectedVersion)}
					onclick={handleJoin}
				>
					{#if joining}<Loader2 size={16} class="spin" />{/if}
					{joining ? t('joinServer.joining') : t('joinServer.join')}
				</button>
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

	.modal {
		width: 440px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		box-shadow: var(--shadow-lg);
	}

	.modal h2 {
		font-size: var(--text-lg);
		margin-bottom: 8px;
	}

	label {
		font-size: 0.75rem;
		color: var(--text-secondary);
		margin-top: 8px;
	}

	input,
	select {
		padding: 10px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.88rem;
	}

	.address-row {
		display: flex;
		gap: 6px;
	}

	.address-row input {
		flex: 1;
		min-width: 0;
	}

	.star-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-muted);
		cursor: pointer;
	}

	.star-btn:hover:not(:disabled) {
		color: var(--accent);
		border-color: var(--accent);
	}

	.star-btn:disabled {
		opacity: 0.6;
		cursor: default;
	}

	.fav-label {
		font-size: 0.75rem;
		color: var(--text-secondary);
	}

	.fav-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 168px;
		overflow-y: auto;
	}

	.fav-row {
		display: flex;
		align-items: center;
		gap: 4px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		border: 1px solid var(--border);
	}

	.fav-main {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 10px;
		border: none;
		background: transparent;
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
		background: var(--color-error);
	}

	.dot.online {
		background: #4ade80;
	}

	.dot.loading {
		background: var(--text-muted);
		opacity: 0.6;
	}

	.fav-text {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.fav-name {
		font-size: 0.84rem;
		font-weight: 700;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.fav-sub {
		font-size: 0.7rem;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.fav-meta {
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		font-size: 0.76rem;
		font-weight: 600;
	}

	.fav-meta small {
		font-size: 0.64rem;
		font-weight: 400;
		color: var(--text-muted);
	}

	.fav-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		flex-shrink: 0;
		border: none;
		border-radius: var(--border-radius-sm);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.fav-icon:hover:not(:disabled) {
		color: var(--accent);
	}

	.fav-icon.danger:hover:not(:disabled) {
		color: var(--color-error);
	}

	.fav-icon:disabled {
		opacity: 0.45;
		cursor: default;
	}

	.mode-tabs {
		display: flex;
		gap: 6px;
		margin-top: 10px;
	}

	.mode-tab {
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

	.mode-tab.active {
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		border-color: var(--accent);
		color: var(--text-primary);
	}

	.mode-tab:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.hint {
		font-size: 0.75rem;
		color: var(--text-muted);
		margin-top: 4px;
	}

	.hint.warn {
		color: var(--color-warning);
	}

	.error {
		font-size: 0.75rem;
		color: var(--color-error);
	}

	.loading {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.8rem;
		color: var(--text-secondary);
		margin-top: 6px;
	}

	.actions {
		display: flex;
		gap: 8px;
		margin-top: 16px;
	}

	.btn {
		flex: 1;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-weight: 600;
		font-size: 0.82rem;
		cursor: pointer;
	}

	.btn.primary {
		background: var(--accent);
		color: var(--accent-text);
		border-color: var(--accent);
	}

	.btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
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
</style>
