<script lang="ts">
	import { appState } from '$lib/state/state.svelte';
	import { updateSettings } from '$lib/api/tflApi';
	import { Zap, FolderOpen, X } from 'lucide-svelte';

	let { variant, onDone }: { variant: 'modal' | 'banner'; onDone?: () => void } = $props();

	let busy = $state<'auto' | 'manual' | null>(null);

	async function choose(mode: 'auto' | 'manual') {
		if (!appState.settings) return;
		busy = mode;
		appState.settings.native_dialog_mode = mode;
		appState.settings.native_dialog_mode_prompted = true;
		try {
			await updateSettings(appState.settings);
			onDone?.();
		} finally {
			busy = null;
		}
	}
</script>

{#if variant === 'modal'}
	<div class="overlay">
		<div class="card" role="dialog" aria-modal="true">
			<h1>¿Cómo querés instalar contenido propio?</h1>
			<p class="lead">
				Esto define si vas a poder subir tu propio ícono/wallpaper o agregar mods por archivo a
				mano. Lo podés cambiar cuando quieras desde Ajustes.
			</p>
			<div class="options">
				<button type="button" class="option primary" disabled={!!busy} onclick={() => choose('auto')}>
					<span class="option-icon"><Zap size={18} /></span>
					<span class="option-text">
						<span class="option-title">Automático (recomendado)</span>
						<span class="option-sub">Todo se instala solo — sin diálogos de archivo, sin riesgo de crash.</span>
					</span>
				</button>
				<button type="button" class="option" disabled={!!busy} onclick={() => choose('manual')}>
					<span class="option-icon"><FolderOpen size={18} /></span>
					<span class="option-text">
						<span class="option-title">Manual</span>
						<span class="option-sub">
							Habilita subir tu propio ícono/wallpaper y mods por archivo — en algunos casos
							puede cerrar el launcher de golpe.
						</span>
					</span>
				</button>
			</div>
		</div>
	</div>
{:else}
	<div class="banner">
		<div class="banner-body">
			<span class="banner-title">¿Cómo querés instalar contenido propio?</span>
			<span class="banner-sub">Automático: sin diálogos de archivo. Manual: los habilita, con riesgo de crash.</span>
			<div class="banner-actions">
				<button type="button" disabled={!!busy} onclick={() => choose('auto')}>Automático</button>
				<button type="button" disabled={!!busy} onclick={() => choose('manual')}>Manual</button>
			</div>
		</div>
		<button type="button" class="banner-close" aria-label="Cerrar" disabled={!!busy} onclick={() => choose('auto')}>
			<X size={12} />
		</button>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}

	.card {
		width: min(460px, calc(100vw - 32px));
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 28px;
		box-shadow: var(--shadow-lg);
	}

	.card h1 {
		font-size: var(--text-lg);
		margin-bottom: 8px;
	}

	.lead {
		color: var(--text-secondary);
		font-size: 0.82rem;
		margin-bottom: 18px;
	}

	.options {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.option {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px 16px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
		transition:
			border-color 0.15s,
			transform 0.12s;
	}

	.option:hover:not(:disabled) {
		border-color: var(--accent);
		transform: translateY(-1px);
	}

	.option:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.option.primary {
		border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
		background: color-mix(in srgb, var(--accent) 10%, var(--bg-input));
	}

	.option-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
		color: var(--accent);
	}

	.option-text {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.option-title {
		font-size: 0.86rem;
		font-weight: 700;
	}

	.option-sub {
		font-size: 0.72rem;
		color: var(--text-muted);
		line-height: 1.4;
	}

	.banner {
		position: fixed;
		right: 16px;
		top: 16px;
		z-index: 200;
		max-width: 320px;
		background: var(--bg-card);
		border: 1px solid var(--accent);
		border-radius: var(--border-radius);
		box-shadow: var(--shadow-md);
		padding: 10px 4px 10px 12px;
		display: flex;
		align-items: flex-start;
		gap: 4px;
		animation: tfl-banner-in 0.25s ease;
	}

	.banner-body {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.banner-title {
		font-size: 0.78rem;
		font-weight: 700;
		color: var(--text-primary);
	}

	.banner-sub {
		font-size: 0.68rem;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.banner-actions {
		display: flex;
		gap: 6px;
		margin-top: 2px;
	}

	.banner-actions button {
		padding: 5px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.7rem;
		font-weight: 600;
		cursor: pointer;
	}

	.banner-actions button:hover:not(:disabled) {
		border-color: var(--accent);
	}

	.banner-close {
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		padding: 4px;
		border-radius: 4px;
	}

	.banner-close:hover:not(:disabled) {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.08);
	}

	@keyframes tfl-banner-in {
		from {
			opacity: 0;
			transform: translateY(-8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
