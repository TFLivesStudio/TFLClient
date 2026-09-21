<script lang="ts">
	// Cubre dos casos con el mismo mecanismo (localStorage['tfl-last-seen-version']
	// vs __APP_VERSION__, la versión del build actual):
	//   - Nunca se vio ningún valor guardado → primera vez que se abre el
	//     launcher de una, muestra tips rápidos (versión simplificada de
	//     "tour guiado" — no un spotlight interactivo con posicionamiento
	//     sobre elementos reales, eso es mucho más trabajo de UI; esto son
	//     4 tips en una lista).
	//   - Había un valor guardado pero es distinto al actual → se acaba de
	//     actualizar, muestra "qué hay de nuevo" con link al Release.
	// Si coincide, no muestra nada — no molesta en cada apertura.
	import { openExternalUrl } from '$lib/api/tflApi';
	import { Sparkles, X, ExternalLink } from 'lucide-svelte';

	const STORAGE_KEY = 'tfl-last-seen-version';

	type Mode = 'none' | 'first-run' | 'update';

	function detectMode(): Mode {
		if (typeof localStorage === 'undefined') return 'none';
		const seen = localStorage.getItem(STORAGE_KEY);
		if (seen === null) return 'first-run';
		if (seen !== __APP_VERSION__) return 'update';
		return 'none';
	}

	let mode = $state<Mode>(detectMode());

	function dismiss() {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(STORAGE_KEY, __APP_VERSION__);
		}
		mode = 'none';
	}

	function openRelease() {
		openExternalUrl(`https://github.com/TFLivesStudio/TFLClient/releases/tag/v${__APP_VERSION__}`);
	}

	const TIPS = [
		'Ctrl+K (⌘K en Mac) abre una paleta para saltar entre instancias o acciones sin tocar el mouse.',
		'"TFL Selection" trae modpacks curados y los que publique el equipo — un click y quedan listos.',
		'Cada instancia tiene su propia pestaña de Mods, Shaders y Modpacks — nada se mezcla entre instancias.',
		'En Ajustes → Estilo podés cambiar acento, superficie, densidad y hasta el fondo animado.'
	];
</script>

<svelte:window onkeydown={(e) => mode !== 'none' && e.key === 'Escape' && dismiss()} />

{#if mode !== 'none'}
	<div class="overlay" onclick={dismiss} onkeydown={(e) => e.key === 'Escape' && dismiss()} role="button" tabindex="-1">
		<div class="panel anim-fade-in" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
			<div class="panel-header">
				<div class="title-row"><Sparkles size={16} /><h2>{mode === 'first-run' ? 'Bienvenido a TFL Client' : `Novedades en v${__APP_VERSION__}`}</h2></div>
				<button type="button" class="close-btn" onclick={dismiss} aria-label="Cerrar"><X size={16} /></button>
			</div>

			{#if mode === 'first-run'}
				<ul class="tips">
					{#each TIPS as tip}
						<li>{tip}</li>
					{/each}
				</ul>
			{:else}
				<p class="update-text">
					Se actualizó solo a la versión <strong>{__APP_VERSION__}</strong>. El detalle completo de
					qué cambió está en el Release de GitHub.
				</p>
				<button type="button" class="release-link" onclick={openRelease}>
					<ExternalLink size={13} /> Ver el Release completo
				</button>
			{/if}

			<button type="button" class="got-it-btn" onclick={dismiss}>Entendido</button>
		</div>
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
		z-index: 250;
	}

	.panel {
		width: min(420px, calc(100vw - 32px));
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 20px;
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.title-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.title-row :global(svg) {
		color: var(--accent);
	}

	.panel-header h2 {
		font-size: 0.95rem;
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

	.tips {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding-left: 18px;
		font-size: 0.8rem;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.update-text {
		font-size: 0.82rem;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.release-link {
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
		cursor: pointer;
	}

	.got-it-btn {
		padding: 9px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: var(--accent);
		color: var(--accent-text);
		font-size: 0.82rem;
		font-weight: 700;
		cursor: pointer;
	}
</style>
