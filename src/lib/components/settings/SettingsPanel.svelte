<script lang="ts">
	import { onMount } from 'svelte';
	import { appState } from '$lib/state/state.svelte';
	import {
		updateSettings,
		setQualityProfile,
		getRecommendedRam,
		getJavaStatus,
		getUserList,
		switchUser,
		removeUser,
		addOfflineAccount,
		getCurrentUser
	} from '$lib/api/tflApi';
	import type { QualityProfile, MinecraftUser, JavaStatus } from '$lib/types/types';
	import { X, Check, Trash2, Plus, Coffee } from 'lucide-svelte';

	let { onClose }: { onClose: () => void } = $props();

	const QUALITY: QualityProfile[] = ['Lite', 'Balanced', 'Experience'];
	const ACCENTS = [
		{ id: 'orange', label: 'Naranja', color: '#ff7a2e' },
		{ id: 'violet', label: 'Violeta', color: '#8b5cf6' },
		{ id: 'teal', label: 'Verde azulado', color: '#14b8a6' }
	];

	let tab = $state<'general' | 'accounts' | 'java'>('general');

	let accent = $state(
		typeof localStorage !== 'undefined'
			? (localStorage.getItem('tfl-accent') ?? 'orange')
			: 'orange'
	);

	let ramTotal = $state<number | null>(null);
	let minRam = $state(appState.settings?.min_memory ?? 1024);
	let maxRam = $state(appState.settings?.max_memory ?? 2048);
	let savingRam = $state(false);

	let users = $state<MinecraftUser[]>([]);
	let newOfflineName = $state('');
	let accountBusy = $state(false);

	let javaStatuses = $state<JavaStatus[]>([]);

	onMount(async () => {
		try {
			const ram = await getRecommendedRam();
			ramTotal = ram.total_mb;
		} catch {
			// no bloquea el panel si falla
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

	async function setTheme(theme: 'dark' | 'light') {
		if (!appState.settings) return;
		document.documentElement.setAttribute('data-theme', theme);
		appState.settings.theme = theme;
		await updateSettings(appState.settings);
	}

	async function handleQuality(profile: QualityProfile) {
		if (!appState.settings) return;
		const updated = await setQualityProfile(profile);
		appState.settings = updated;
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

	async function loadJava() {
		javaStatuses = await getJavaStatus();
	}

	function selectTab(t: 'general' | 'accounts' | 'java') {
		tab = t;
		if (t === 'accounts') loadAccounts();
		if (t === 'java') loadJava();
	}
</script>

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
				class:active={tab === 'general'}
				onclick={() => selectTab('general')}>General</button
			>
			<button
				type="button"
				class="ptab"
				class:active={tab === 'accounts'}
				onclick={() => selectTab('accounts')}>Cuentas</button
			>
			<button
				type="button"
				class="ptab"
				class:active={tab === 'java'}
				onclick={() => selectTab('java')}>Java</button
			>
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
							onclick={() => setTheme('light')}>Claro</button
						>
					</div>
				</section>

				<section>
					<span class="section-label">Color de acento</span>
					<div class="row">
						{#each ACCENTS as a (a.id)}
							<button
								type="button"
								class="swatch"
								class:active={accent === a.id}
								style="background: {a.color}"
								onclick={() => applyAccent(a.id)}
								aria-label={a.label}
							></button>
						{/each}
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
			{:else if tab === 'accounts'}
				<section>
					<span class="section-label">Cuentas guardadas</span>
					<div class="accounts-list">
						{#each users as u (u.uuid)}
							<div class="account-row" class:active={appState.currentUser?.uuid === u.uuid}>
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
						{/each}
					</div>
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
		width: 420px;
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
		font-size: 1rem;
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
		padding: 6px 4px;
		margin-right: 14px;
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--text-secondary);
		font-size: 0.8rem;
		font-weight: 600;
		cursor: pointer;
	}

	.ptab.active {
		color: var(--text-primary);
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

	.swatch {
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 2px solid transparent;
		cursor: pointer;
	}

	.swatch.active {
		border-color: var(--text-primary);
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
