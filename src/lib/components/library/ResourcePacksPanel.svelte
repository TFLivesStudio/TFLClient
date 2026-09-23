<script lang="ts">
	import type { InstanceData } from '$lib/types/types';
	import ContentManager from './ContentManager.svelte';

	let { instance }: { instance: InstanceData } = $props();
</script>

<div class="resourcepacks-panel">
	<p class="hint">
		Los resource packs funcionan en cualquier instancia, con o sin mods — no necesitás nada extra
		instalado. Se activan desde el menú de Minecraft (Opciones → Paquetes de recursos).
	</p>
	<ContentManager kind="resourcepack" {instance} />
</div>

<style>
	.resourcepacks-panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
	}
</style>
