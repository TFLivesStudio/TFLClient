<script lang="ts">
	import type { InstanceData } from '$lib/types/types';
	import { installShader } from '$lib/api/tflApi';
	import ContentManager from './ContentManager.svelte';
	import { Loader2 } from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();

	// 3 opciones fijas en vez de tener que buscar y elegir un shader pack a
	// mano — project_id reales de Modrinth, verificados en vivo, no
	// inventados. Cada uno queda instalado como cualquier otro shader (se
	// puede sacar desde "Gestionar" como siempre).
	const GRAPHICS_PRESETS = [
		{ label: 'Bajo', projectId: 'izsIPI7a' }, // MakeUp - Ultra Fast
		{ label: 'Medio', projectId: 'HVnmMxH1' }, // Complementary Shaders - Reimagined
		{ label: 'Alto', projectId: 'R6NEzAwj' } // Complementary Shaders - Unbound
	];

	let installingPreset = $state<string | null>(null);
	let presetError = $state<string | null>(null);
	// ContentManager tiene su propio estado de "instalados" — este contador
	// se lo pasa como prop para que se refresque cuando un preset rápido
	// instala algo, sin que ambos componentes compartan estado directamente.
	let refreshSignal = $state(0);

	async function installPreset(projectId: string) {
		installingPreset = projectId;
		presetError = null;
		try {
			await installShader(instance.name, projectId, instance.mc_version);
			refreshSignal += 1;
		} catch (e) {
			presetError = String(e);
		} finally {
			installingPreset = null;
		}
	}
</script>

<div class="shaders-panel">
	<div class="presets-row">
		<span class="section-label">Calidad gráfica (rápido)</span>
		<div class="presets">
			{#each GRAPHICS_PRESETS as preset (preset.projectId)}
				<button
					type="button"
					class="preset-btn"
					disabled={installingPreset === preset.projectId}
					onclick={() => installPreset(preset.projectId)}
				>
					{#if installingPreset === preset.projectId}<Loader2 size={13} class="spin" />{/if}
					{preset.label}
				</button>
			{/each}
		</div>
		{#if presetError}<p class="error">{presetError}</p>{/if}
	</div>

	<ContentManager kind="shader" {instance} {refreshSignal} />
</div>

<style>
	.shaders-panel {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.presets-row {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.section-label {
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.presets {
		display: flex;
		gap: 8px;
	}

	.preset-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-size: 0.78rem;
		font-weight: 600;
		cursor: pointer;
	}

	.preset-btn:hover:not(:disabled) {
		border-color: var(--accent);
		color: var(--accent);
	}

	.preset-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.error {
		color: var(--color-error);
		font-size: 0.78rem;
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
