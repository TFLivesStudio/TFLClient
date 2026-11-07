<script lang="ts">
	import { modalFocus } from '$lib/visual/modalFocus';
	import { onMount } from 'svelte';
	import type { InstanceData } from '$lib/types/types';
	import {
		getSelectionCatalog,
		installSelectionTarget,
		verifySelectionIntegrity,
		repairSelectionInstance,
		checkSelectionUpdate,
		type SelectionCatalog,
		type SelectionTarget,
		type SelectionIntegrityReport,
		type SelectionUpdateInfo
	} from '$lib/api/tflSelection';
	import { launchInstance } from '$lib/api';
	import { downloadTasks } from '$lib/state/downloadState.svelte';
	import { gameSession } from '$lib/state/gameSession.svelte';
	import { addNotice } from '$lib/visual/notifications.svelte';
	import {
		X,
		Download,
		Loader2,
		Check,
		Play,
		RefreshCw,
		ShieldCheck,
		Wrench,
		AlertTriangle,
		ChevronDown,
		ChevronUp,
		Package,
		ShieldAlert
	} from 'lucide-svelte';
	import Tfl from '$lib/icons/Tfl.svelte';
	import { t } from '$lib/i18n/index.svelte';

	let {
		instances,
		onClose,
		onInstanceCreated
	}: {
		instances: InstanceData[];
		onClose: () => void;
		onInstanceCreated?: (instance: InstanceData) => void;
	} = $props();

	let catalog = $state<SelectionCatalog | null>(null);
	let loading = $state(true);
	let catalogError = $state<string | null>(null);

	let installing = $state(false);
	let installError = $state<string | null>(null);
	let customInstanceName = $state('TFL Selection 1.8.9');

	let checkingUpdate = $state(false);
	let updateInfo = $state<SelectionUpdateInfo | null>(null);
	let upToDate = $state(false);

	let verifying = $state(false);
	let integrityReport = $state<SelectionIntegrityReport | null>(null);

	let repairing = $state(false);
	let repairSuccess = $state(false);

	let showComponents = $state(false);

	// Targets que tengan al menos un canal publicado (ej. mc-1.8.9-ornithe en canal beta)
	const publishedTargets = $derived.by(() => {
		if (!catalog) return [];
		return catalog.targets.filter((target) => {
			const ch = target.channels;
			return !!(ch.stable || ch.beta || ch.dev);
		});
	});

	const activeTarget = $derived<SelectionTarget | undefined>(publishedTargets[0]);

	// Canal por defecto (prioridad stable > beta > dev)
	const defaultChannel = $derived.by(() => {
		if (!activeTarget) return 'beta';
		if (activeTarget.channels.stable) return 'stable';
		if (activeTarget.channels.beta) return 'beta';
		if (activeTarget.channels.dev) return 'dev';
		return 'beta';
	});

	let selectedChannel = $state<string>('beta');

	$effect(() => {
		if (defaultChannel) selectedChannel = defaultChannel;
	});

	// Buscar si ya existe una instancia de esta selección instalada
	const installedInstance = $derived.by(() => {
		return instances.find(
			(i) =>
				i.loader === 'ornithe-gen2' ||
				i.name.toLowerCase().includes('tfl selection') ||
				(i.launch_version_id && i.launch_version_id.includes('ornithe-gen2'))
		);
	});

	const isRunning = $derived.by(() => {
		if (!installedInstance) return false;
		return gameSession.running.some((r) => r.name === installedInstance.name);
	});

	const activeDownloadTask = $derived.by(() => {
		if (!activeTarget) return null;
		return downloadTasks[`selection:${activeTarget.target_id}`] ?? null;
	});

	onMount(async () => {
		await loadCatalog(false);
	});

	async function loadCatalog(forceRefresh = false) {
		loading = true;
		catalogError = null;
		try {
			catalog = await getSelectionCatalog(forceRefresh);
		} catch (e) {
			catalogError = String(e);
		} finally {
			loading = false;
		}
	}

	async function handleInstall() {
		if (!activeTarget || installing) return;
		installing = true;
		installError = null;
		try {
			const name = customInstanceName.trim() || 'TFL Selection 1.8.9';
			const created = await installSelectionTarget(name, activeTarget.target_id, selectedChannel);
			onInstanceCreated?.(created);
			addNotice({
				kind: 'success',
				title: 'finished',
				message: `Instancia ${name} instalada correctamente con Ornithe Gen2.`
			});
		} catch (e) {
			installError = String(e);
			addNotice({
				kind: 'error',
				title: 'failed',
				message: `Error al instalar TFL Selection: ${String(e)}`
			});
		} finally {
			installing = false;
		}
	}

	async function handleLaunch() {
		if (!installedInstance || isRunning) return;
		try {
			await launchInstance(installedInstance.name);
			onClose();
		} catch (e) {
			addNotice({
				kind: 'error',
				title: 'failed',
				message: `Error al iniciar: ${String(e)}`
			});
		}
	}

	async function handleCheckUpdate() {
		if (!installedInstance || checkingUpdate) return;
		checkingUpdate = true;
		updateInfo = null;
		upToDate = false;
		try {
			const info = await checkSelectionUpdate(installedInstance.name);
			if (info) {
				updateInfo = info;
			} else {
				upToDate = true;
			}
		} catch (e) {
			addNotice({
				kind: 'error',
				title: 'failed',
				message: `Error al buscar actualizaciones: ${String(e)}`
			});
		} finally {
			checkingUpdate = false;
		}
	}

	async function handleVerify() {
		if (!installedInstance || verifying) return;
		verifying = true;
		integrityReport = null;
		try {
			integrityReport = await verifySelectionIntegrity(installedInstance.name);
		} catch (e) {
			addNotice({
				kind: 'error',
				title: 'failed',
				message: `Error al verificar integridad: ${String(e)}`
			});
		} finally {
			verifying = false;
		}
	}

	async function handleRepair() {
		if (!installedInstance || repairing) return;
		repairing = true;
		repairSuccess = false;
		try {
			await repairSelectionInstance(installedInstance.name);
			repairSuccess = true;
			integrityReport = null;
			addNotice({
				kind: 'success',
				title: 'finished',
				message: 'Instancia reparada: archivos oficiales restaurados y en integridad completa.'
			});
		} catch (e) {
			addNotice({
				kind: 'error',
				title: 'failed',
				message: `Error al reparar: ${String(e)}`
			});
		} finally {
			repairing = false;
		}
	}

	const OFFICIAL_MODS = [
		{ name: 'Argentum', version: '1.0.2', desc: 'Reemplazo moderno y optimizado de OptiFine' },
		{
			name: 'Argentum Extras',
			version: '1.0.2',
			desc: 'Soporte adicional de características gráficas'
		},
		{ name: 'Cera', version: '1.0.2', desc: 'Optimizaciones de renderizado y estabilidad' },
		{ name: 'Sarcio', version: '1.1.1', desc: 'Compatibilidad de parches de entidades e interfaz' },
		{
			name: 'Ornithe Standard Libraries',
			version: '0.22.2',
			desc: 'Librerías base de Ornithe Gen2'
		},
		{ name: 'Amplio', version: '0.1.1', desc: 'Zoom suave con aceleración cinemática' },
		{ name: 'Legacy LWJGL3', version: '1.4.10', desc: 'Motor gráfico LWJGL 3 moderno para 1.8.9' },
		{ name: 'Lucerna', version: '0.1.5-tfl.2', desc: 'Iluminación dinámica suave y eficiente' },
		{ name: 'Pulse', version: '1.0.0', desc: 'Rendimiento y reducción de micro-stuttering' },
		{
			name: 'SoundFix',
			version: '1.1.0',
			desc: 'Correcciones y estabilidad del subsistema de audio'
		},
		{ name: 'TFL Core', version: '0.5.0-visual-preview.1', desc: 'Núcleo oficial de TFL Client' }
	];
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
		use:modalFocus={onClose}
		aria-modal="true"
		tabindex="-1"
	>
		<!-- Encabezado del modal -->
		<div class="panel-header">
			<div class="title-row">
				<Tfl width="20" height="20" />
				<h2>TFL Selection</h2>
			</div>
			<button type="button" class="close-btn" onclick={onClose} aria-label={t('settings.close')}>
				<X size={16} />
			</button>
		</div>

		<!-- Hero oficial -->
		<div class="selection-hero">
			<div class="hero-mark">
				<Tfl width="32" height="32" />
			</div>
			<div class="hero-text">
				<span class="eyebrow">Distribución Oficial • 1.8.9 Ornithe</span>
				<p class="subtitle">
					Versión curada, optimizada y firmada criptográficamente por TFLives. Lista para jugar con
					un solo click.
				</p>
			</div>
		</div>

		{#if loading}
			<div class="loading-state">
				<Loader2 size={24} class="spin" />
				<span>Verificando catálogo firmado con Ed25519...</span>
			</div>
		{:else if catalogError}
			<div class="error-banner">
				<AlertTriangle size={18} />
				<div>
					<strong>No se pudo cargar el catálogo de TFL Selection</strong>
					<p>{catalogError}</p>
				</div>
				<button type="button" class="retry-btn" onclick={() => loadCatalog(true)}>
					<RefreshCw size={14} /> Reintentar
				</button>
			</div>
		{:else if !activeTarget}
			<div class="empty-state">
				<p>No hay versiones publicadas de TFL Selection en este momento.</p>
			</div>
		{:else}
			{@const ch = activeTarget.channels}
			{@const activePointer =
				ch[selectedChannel as keyof typeof ch] || ch.beta || ch.stable || ch.dev}
			<div class="target-card">
				<!-- Header de la tarjeta del target -->
				<div class="target-header">
					<div class="target-identity">
						<div class="target-icon">
							<Package size={24} />
						</div>
						<div>
							<div class="target-title-row">
								<h3 class="target-title">{activeTarget.displayName || 'TFL Selection 1.8.9'}</h3>
								{#if activePointer}
									<span class="badge badge-channel">
										{selectedChannel.toUpperCase()} • v{activePointer.release}
									</span>
								{/if}
							</div>
							<p class="target-summary">
								{activeTarget.summary ||
									'Instalación optimizada de Minecraft 1.8.9 con Ornithe Gen2 y 11 mods administrados.'}
							</p>
						</div>
					</div>
				</div>

				<!-- Píldoras de especificaciones técnicas -->
				<div class="specs-grid">
					<div class="spec-pill">
						<span class="spec-label">Minecraft</span>
						<span class="spec-value">{activeTarget.minecraft_version}</span>
					</div>
					<div class="spec-pill">
						<span class="spec-label">Loader</span>
						<span class="spec-value">Ornithe Gen2 (Fabric 0.20.1)</span>
					</div>
					<div class="spec-pill">
						<span class="spec-label">Runtime</span>
						<span class="spec-value">Java 25 (Azul Zulu)</span>
					</div>
					<div class="spec-pill">
						<span class="spec-label">Seguridad</span>
						<span class="spec-value">Ed25519 + SHA-256</span>
					</div>
				</div>

				<!-- Estado / Acciones de Instalación -->
				<div class="target-main-action">
					{#if installedInstance}
						<div class="installed-panel">
							<div class="installed-header">
								<div class="status-indicator">
									<span class="status-dot"></span>
									<span>Instalada como <strong>{installedInstance.name}</strong></span>
								</div>
								{#if isRunning}
									<span class="badge badge-running">En ejecución</span>
								{/if}
							</div>

							<!-- Botonera de acciones -->
							<div class="actions-row">
								<button
									type="button"
									class="primary-btn play-btn"
									disabled={isRunning}
									onclick={handleLaunch}
								>
									<Play size={16} />
									<span>{isRunning ? 'Ejecutándose' : 'Jugar ahora'}</span>
								</button>

								<button
									type="button"
									class="secondary-btn"
									disabled={checkingUpdate}
									onclick={handleCheckUpdate}
									title="Buscar actualizaciones oficiales"
								>
									<RefreshCw size={14} class={checkingUpdate ? 'spin' : ''} />
									<span>Actualizaciones</span>
								</button>

								<button
									type="button"
									class="secondary-btn"
									disabled={verifying}
									onclick={handleVerify}
									title="Verificar integridad de los 11 mods oficiales"
								>
									<ShieldCheck size={14} class={verifying ? 'spin' : ''} />
									<span>Verificar</span>
								</button>

								<button
									type="button"
									class="secondary-btn"
									disabled={repairing}
									onclick={handleRepair}
									title="Restaurar archivos corruptos o ausentes"
								>
									<Wrench size={14} class={repairing ? 'spin' : ''} />
									<span>Reparar</span>
								</button>
							</div>

							<!-- Respuestas de comprobaciones -->
							{#if upToDate}
								<div class="notice-box notice-success">
									<Check size={15} />
									<span>Tu instancia está al día con la última versión oficial.</span>
								</div>
							{/if}

							{#if updateInfo}
								<div class="notice-box notice-info">
									<AlertTriangle size={15} />
									<span
										>Nueva versión disponible: v{updateInfo.latest_release} (actual: v{updateInfo.current_release}).</span
									>
									<button type="button" class="btn-sm" onclick={handleInstall}> Actualizar </button>
								</div>
							{/if}

							{#if integrityReport}
								<div
									class="notice-box {integrityReport.is_valid
										? 'notice-success'
										: 'notice-warning'}"
								>
									{#if integrityReport.is_valid}
										<Check size={15} />
										<span
											>Integridad perfecta: los 11 mods coinciden con los hashes SHA-256 oficiales.</span
										>
									{:else}
										<ShieldAlert size={15} />
										<div>
											<strong>Se detectaron discrepancias en la instancia:</strong>
											{#if integrityReport.missing.length > 0}
												<p>Faltantes: {integrityReport.missing.join(', ')}</p>
											{/if}
											{#if integrityReport.corrupted.length > 0}
												<p>Corruptos: {integrityReport.corrupted.join(', ')}</p>
											{/if}
											{#if integrityReport.foreign_mods.length > 0}
												<p>Archivos no autorizados: {integrityReport.foreign_mods.join(', ')}</p>
											{/if}
										</div>
										<button type="button" class="btn-sm" onclick={handleRepair}>
											Reparar ahora
										</button>
									{/if}
								</div>
							{/if}

							{#if repairSuccess}
								<div class="notice-box notice-success">
									<Check size={15} />
									<span
										>Instancia reparada exitosamente. Tus configuraciones y mundos siguen intactos.</span
									>
								</div>
							{/if}
						</div>
					{:else}
						<!-- Formulario de instalación nueva -->
						<div class="install-form">
							<div class="form-row">
								<label for="instance-name-input">Nombre de la instancia:</label>
								<input
									id="instance-name-input"
									type="text"
									bind:value={customInstanceName}
									placeholder="TFL Selection 1.8.9"
									disabled={installing}
								/>
							</div>

							<div class="form-row">
								<label for="channel-select">Canal oficial:</label>
								<select id="channel-select" bind:value={selectedChannel} disabled={installing}>
									{#if ch.stable}<option value="stable">Stable</option>{/if}
									{#if ch.beta}<option value="beta">Beta (Oficial 1.8.9)</option>{/if}
									{#if ch.dev}<option value="dev">Dev</option>{/if}
								</select>
							</div>

							{#if installError}
								<div class="notice-box notice-warning">
									<AlertTriangle size={15} />
									<span>{installError}</span>
								</div>
							{/if}

							<!-- Progreso de descarga en vivo -->
							{#if activeDownloadTask}
								<div class="progress-box">
									<div class="progress-header">
										<span class="progress-stage">{activeDownloadTask.stage}</span>
										<span class="progress-counts">
											{activeDownloadTask.itemCurrent} / {activeDownloadTask.itemTotal} archivos
										</span>
									</div>
									<div class="progress-bar-bg">
										<div
											class="progress-bar-fill"
											style="width: {activeDownloadTask.bytesTotal > 0
												? Math.min(
														100,
														Math.round(
															(activeDownloadTask.bytesCurrent / activeDownloadTask.bytesTotal) *
																100
														)
													)
												: 0}%"
										></div>
									</div>
									<div class="progress-footer">
										<span class="current-file">
											{activeDownloadTask.currentItem || 'Procesando descarga...'}
										</span>
										<span class="bytes-info">
											{(activeDownloadTask.bytesCurrent / 1048576).toFixed(1)} / {(
												activeDownloadTask.bytesTotal / 1048576
											).toFixed(1)} MB
										</span>
									</div>
								</div>
							{/if}

							<button
								type="button"
								class="primary-btn install-btn"
								disabled={installing}
								onclick={handleInstall}
							>
								{#if installing}
									<Loader2 size={16} class="spin" />
									<span>Instalando TFL Selection...</span>
								{:else}
									<Download size={16} />
									<span>Instalar TFL Selection 1.8.9</span>
								{/if}
							</button>
						</div>
					{/if}
				</div>

				<!-- Sección colapsable: 11 mods y seguridad -->
				<div class="components-section">
					<button
						type="button"
						class="toggle-components-btn"
						onclick={() => (showComponents = !showComponents)}
					>
						<span>Componentes incluidos (11 mods oficiales) y seguridad</span>
						{#if showComponents}
							<ChevronUp size={16} />
						{:else}
							<ChevronDown size={16} />
						{/if}
					</button>

					{#if showComponents}
						<div class="components-content anim-fade-in">
							<p class="protection-notice">
								🛡️ <strong>Instancia administrada y protegida:</strong> Los 11 mods oficiales están verificados
								por hashes SHA-256 e integrados con Ornithe Gen2 + Java 25. Para asegurar estabilidad
								competitiva y prevenir incompatibilidades, no se pueden modificar mods individuales en
								esta instancia. Tus mundos y configuraciones personales siempre están preservados.
							</p>

							<div class="mods-list">
								{#each OFFICIAL_MODS as mod (mod.name)}
									<div class="mod-item">
										<div class="mod-meta">
											<span class="mod-name">{mod.name}</span>
											<span class="mod-version">v{mod.version}</span>
										</div>
										<p class="mod-desc">{mod.desc}</p>
									</div>
								{/each}
							</div>
						</div>
					{/if}
				</div>
			</div>
		{/if}
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
		width: min(680px, calc(100vw - 32px));
		max-height: 85vh;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 24px;
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 16px;
		overflow-y: auto;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.title-row {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.title-row :global(svg) {
		color: var(--accent);
	}

	.panel-header h2 {
		font-size: 1.15rem;
		font-family: var(--font-brand);
		margin: 0;
		color: var(--text-primary);
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition:
			background 0.15s,
			color 0.15s;
	}
	.close-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.selection-hero {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 16px;
		border: 1px solid color-mix(in srgb, var(--accent) 30%, var(--border));
		border-radius: var(--border-radius);
		background:
			linear-gradient(130deg, color-mix(in srgb, var(--accent) 14%, transparent), transparent 65%),
			var(--bg-input);
	}

	.hero-mark {
		display: grid;
		place-items: center;
		width: 52px;
		height: 52px;
		border-radius: 14px;
		flex-shrink: 0;
		color: var(--accent);
		background: color-mix(in srgb, var(--accent) 18%, transparent);
		border: 1px solid color-mix(in srgb, var(--accent) 35%, transparent);
	}

	.eyebrow {
		display: block;
		color: var(--accent);
		font-size: 0.68rem;
		letter-spacing: 0.14em;
		font-weight: 800;
		text-transform: uppercase;
		margin-bottom: 4px;
	}

	.subtitle {
		font-size: 0.82rem;
		color: var(--text-secondary);
		line-height: 1.45;
		margin: 0;
	}

	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 36px 0;
		color: var(--text-muted);
		font-size: 0.85rem;
	}

	.error-banner {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 14px;
		border-radius: var(--border-radius);
		background: color-mix(in srgb, var(--color-error, #f44336) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-error, #f44336) 30%, transparent);
		color: var(--text-primary);
		font-size: 0.8rem;
	}

	.error-banner div {
		flex: 1;
	}

	.retry-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-card);
		color: var(--text-primary);
		font-size: 0.76rem;
		cursor: pointer;
	}

	.target-card {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 18px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		background: color-mix(in srgb, var(--bg-input) 50%, var(--bg-card));
	}

	.target-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
	}

	.target-identity {
		display: flex;
		align-items: center;
		gap: 14px;
	}

	.target-icon {
		display: grid;
		place-items: center;
		width: 44px;
		height: 44px;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--accent) 15%, transparent);
		color: var(--accent);
		flex-shrink: 0;
	}

	.target-title-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.target-title {
		font-size: 1.05rem;
		font-weight: 700;
		margin: 0;
		color: var(--text-primary);
	}

	.badge {
		font-size: 0.65rem;
		font-weight: 700;
		padding: 2px 7px;
		border-radius: 999px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.badge-channel {
		background: color-mix(in srgb, var(--accent) 20%, transparent);
		color: var(--accent);
		border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
	}

	.badge-running {
		background: color-mix(in srgb, #4caf50 20%, transparent);
		color: #4caf50;
		border: 1px solid color-mix(in srgb, #4caf50 40%, transparent);
	}

	.target-summary {
		font-size: 0.78rem;
		color: var(--text-muted);
		margin: 4px 0 0 0;
	}

	.specs-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
		gap: 8px;
	}

	.spec-pill {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 12px;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
	}

	.spec-label {
		font-size: 0.65rem;
		color: var(--text-muted);
		text-transform: uppercase;
		font-weight: 700;
		letter-spacing: 0.05em;
	}

	.spec-value {
		font-size: 0.75rem;
		color: var(--text-primary);
		font-weight: 600;
	}

	.target-main-action {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		padding: 16px;
	}

	.installed-panel {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.installed-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.status-indicator {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.82rem;
		color: var(--text-primary);
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #4caf50;
	}

	.actions-row {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.primary-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 9px 18px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: var(--accent);
		color: var(--accent-text);
		font-size: 0.82rem;
		font-weight: 700;
		cursor: pointer;
		transition: filter 0.15s;
	}
	.primary-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}
	.primary-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.play-btn {
		background: #2e7d32;
		color: #ffffff;
	}
	.play-btn:hover:not(:disabled) {
		background: #388e3c;
	}

	.secondary-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.78rem;
		font-weight: 600;
		cursor: pointer;
		transition: background 0.15s;
	}
	.secondary-btn:hover:not(:disabled) {
		background: var(--bg-hover);
	}
	.secondary-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.notice-box {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 14px;
		border-radius: var(--border-radius-sm);
		font-size: 0.78rem;
	}
	.notice-box div {
		flex: 1;
	}
	.notice-box p {
		margin: 2px 0 0 0;
		font-size: 0.74rem;
	}

	.notice-success {
		background: color-mix(in srgb, #4caf50 12%, transparent);
		border: 1px solid color-mix(in srgb, #4caf50 30%, transparent);
		color: var(--text-primary);
	}
	.notice-success :global(svg) {
		color: #4caf50;
	}

	.notice-info {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent);
		color: var(--text-primary);
	}

	.notice-warning {
		background: color-mix(in srgb, #ff9800 12%, transparent);
		border: 1px solid color-mix(in srgb, #ff9800 30%, transparent);
		color: var(--text-primary);
	}
	.notice-warning :global(svg) {
		color: #ff9800;
	}

	.btn-sm {
		padding: 4px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: var(--accent);
		color: var(--accent-text);
		font-size: 0.72rem;
		font-weight: 700;
		cursor: pointer;
	}

	.install-form {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.form-row {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.form-row label {
		font-size: 0.76rem;
		font-weight: 600;
		color: var(--text-secondary);
	}

	.form-row input,
	.form-row select {
		padding: 8px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.82rem;
	}

	.progress-box {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 12px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		border: 1px solid var(--border);
	}

	.progress-header,
	.progress-footer {
		display: flex;
		justify-content: space-between;
		font-size: 0.72rem;
	}

	.progress-stage {
		font-weight: 700;
		color: var(--text-primary);
	}

	.progress-counts,
	.bytes-info {
		color: var(--text-muted);
	}

	.current-file {
		color: var(--text-secondary);
		max-width: 320px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.progress-bar-bg {
		height: 6px;
		border-radius: 999px;
		background: var(--bg-card);
		overflow: hidden;
	}

	.progress-bar-fill {
		height: 100%;
		border-radius: 999px;
		background: var(--accent);
		transition: width 0.15s ease-out;
	}

	.install-btn {
		width: 100%;
		padding: 12px;
		font-size: 0.88rem;
	}

	.components-section {
		display: flex;
		flex-direction: column;
		border-top: 1px solid var(--border);
		padding-top: 12px;
	}

	.toggle-components-btn {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 0;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 0.78rem;
		font-weight: 600;
		cursor: pointer;
	}
	.toggle-components-btn:hover {
		color: var(--text-primary);
	}

	.components-content {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-top: 10px;
	}

	.protection-notice {
		margin: 0;
		padding: 10px 12px;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--accent) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--accent) 20%, transparent);
		font-size: 0.74rem;
		line-height: 1.45;
		color: var(--text-secondary);
	}

	.mods-list {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 8px;
		max-height: 200px;
		overflow-y: auto;
		padding-right: 4px;
	}

	.mod-item {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		background: var(--bg-card);
		border: 1px solid var(--border);
	}

	.mod-meta {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.mod-name {
		font-size: 0.76rem;
		font-weight: 700;
		color: var(--text-primary);
	}

	.mod-version {
		font-size: 0.68rem;
		color: var(--accent);
		font-weight: 600;
	}

	.mod-desc {
		margin: 0;
		font-size: 0.7rem;
		color: var(--text-muted);
		line-height: 1.35;
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
