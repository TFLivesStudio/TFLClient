<script lang="ts">
	import { scenes } from './preferences.svelte';
	import { appearance } from '$lib/state/appearanceState.svelte';
	import { runtime } from '$lib/visual/runtime.svelte';
	import { appState } from '$lib/state/state.svelte';
	import { gameSession } from '$lib/state/gameSession.svelte';
	import { reaction } from './reactions.svelte';
	import { effects } from '$lib/visual/runtime.svelte';
	import SceneStage from './SceneStage.svelte';
	import type { InstanceData } from '$lib/types/types';
	let {
		instance = null,
		placement = 'hero'
	}: { instance?: InstanceData | null; placement?: 'hero' | 'window' } = $props();
	const target = $derived(instance ?? appState.selectedInstance);
	const config = $derived((target && scenes.instances[target.uuid]) || scenes.global);
</script>

{#if config.preset !== 'legacy' && config.scope === placement}<div
		class="living-scene"
		class:window-scene={placement === 'window'}
		data-scene-placement={placement}
		aria-hidden="true"
	>
		<SceneStage
			{config}
			quality={appearance.quality}
			reduced={appearance.reduceMotion || appearance.noInfiniteFx || runtime.systemReducedMotion}
			active={runtime.visible && runtime.focused && !runtime.overlay && runtime.modalDepth === 0}
			gameRunning={gameSession.running.length > 0}
		/>
		<div class="veil"></div>
		{#if config.reactive && effects.current.cinematic && reaction.kind !== 'idle'}{#key reaction.sequence}<div
					class="reaction-light"
					class:error={reaction.kind === 'error'}
				></div>{/key}{/if}
	</div>{/if}

<style>
	.living-scene {
		position: absolute;
		inset: 0;
		pointer-events: none;
		overflow: hidden;
		z-index: 0;
	}
	.window-scene .veil {
		background: color-mix(in srgb, var(--surface-canvas) 64%, transparent);
		opacity: 1;
	}
	.veil {
		position: absolute;
		inset: 0;
		background:
			linear-gradient(90deg, var(--surface-canvas) 0%, transparent 62%),
			linear-gradient(0deg, var(--surface-canvas), transparent 80%);
		opacity: 0.64;
	}
	:global([data-theme='light']) .veil {
		opacity: 0.84;
	}
	.reaction-light {
		position: absolute;
		inset: 0;
		background: radial-gradient(ellipse at 75% 30%, #81ffc62b, transparent 65%);
		animation: scene-response 1.8s ease-out both;
	}
	.reaction-light.error {
		background: radial-gradient(ellipse at 75% 30%, #ff8f8f25, transparent 65%);
	}
	@keyframes scene-response {
		0%,
		100% {
			opacity: 0;
		}
		30% {
			opacity: 1;
		}
	}
</style>
