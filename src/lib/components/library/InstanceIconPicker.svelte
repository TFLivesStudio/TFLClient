<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { pickImageFile, setInstanceIcon } from '$lib/api/tflApi';
	import { X, Upload, Check, Loader2 } from 'lucide-svelte';

	let {
		instanceName,
		onClose,
		onChanged
	}: {
		instanceName: string;
		onClose: () => void;
		onChanged: () => void;
	} = $props();

	interface IconPreset {
		id: string;
		data_url: string;
	}

	let presets = $state<IconPreset[]>([]);
	let loading = $state(true);
	let busy = $state<string | null>(null);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			presets = await invoke<IconPreset[]>('list_instance_icon_presets');
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	});

	function label(id: string) {
		const name = id.replace(/^(mob|place)-/, '').replace(/_/g, ' ');
		return name.charAt(0).toUpperCase() + name.slice(1);
	}

	async function choosePreset(id: string) {
		busy = id;
		error = null;
		try {
			await invoke('set_instance_icon_from_preset', { name: instanceName, presetId: id });
			onChanged();
			onClose();
		} catch (e) {
			error = String(e);
		} finally {
			busy = null;
		}
	}

	async function uploadOwn() {
		const source = await pickImageFile();
		if (!source) return;
		busy = 'upload';
		error = null;
		try {
			await setInstanceIcon(instanceName, source);
			onChanged();
			onClose();
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
		class="panel"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div class="panel-header">
			<h2>Ícono de la instancia</h2>
			<button type="button" class="close-btn" onclick={onClose} aria-label="Cerrar">
				<X size={16} />
			</button>
		</div>

		<button type="button" class="upload-btn" disabled={busy === 'upload'} onclick={uploadOwn}>
			{#if busy === 'upload'}<Loader2 size={14} class="spin" />{:else}<Upload size={14} />{/if}
			Subir mi propia imagen
		</button>

		{#if error}<p class="error">{error}</p>{/if}

		{#if loading}
			<div class="loading-row"><Loader2 size={18} class="spin" /></div>
		{:else}
			<div class="icon-grid">
				{#each presets as p (p.id)}
					<button
						type="button"
						class="icon-swatch"
						disabled={busy === p.id}
						onclick={() => choosePreset(p.id)}
						aria-label={label(p.id)}
						title={label(p.id)}
					>
						{#if busy === p.id}
							<Loader2 size={16} class="spin" />
						{:else}
							<img src={p.data_url} alt="" />
						{/if}
					</button>
				{/each}
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
		width: min(520px, calc(100vw - 32px));
		max-height: 78vh;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		padding: 20px;
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 12px;
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

	.upload-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 9px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-input));
		color: var(--accent);
		font-size: 0.82rem;
		font-weight: 700;
		cursor: pointer;
		flex-shrink: 0;
	}

	.upload-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.error {
		color: var(--color-error);
		font-size: 0.78rem;
	}

	.loading-row {
		display: flex;
		justify-content: center;
		padding: 24px;
		color: var(--text-muted);
	}

	.icon-grid {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 6px;
		overflow-y: auto;
		padding-right: 2px;
	}

	.icon-swatch {
		aspect-ratio: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 6px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		cursor: pointer;
		transition:
			border-color 0.15s,
			transform 0.15s;
	}

	.icon-swatch:hover {
		border-color: var(--accent);
		transform: translateY(-1px);
	}

	.icon-swatch:disabled {
		cursor: not-allowed;
		opacity: 0.6;
	}

	.icon-swatch img {
		width: 100%;
		height: 100%;
		object-fit: contain;
		image-rendering: pixelated;
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
