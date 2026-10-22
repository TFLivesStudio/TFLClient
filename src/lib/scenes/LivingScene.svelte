<script lang="ts">
	import { onMount } from 'svelte';
	import { scenes, watchScenes } from './preferences.svelte';
	import { appearance } from '$lib/state/appearanceState.svelte';
	import { runtime } from '$lib/visual/runtime.svelte';
	import { appState } from '$lib/state/state.svelte';
	import { gameSession } from '$lib/state/gameSession.svelte';
	import { reaction } from './reactions.svelte';
	import { effects } from '$lib/visual/runtime.svelte';
	import SceneStage from './SceneStage.svelte';
	const config = $derived(
		(appState.selectedInstance && scenes.instances[appState.selectedInstance.uuid]) || scenes.global
	);
	onMount(watchScenes);
	$effect(() => {
		document.documentElement.toggleAttribute('data-living-scene', config.preset !== 'legacy');
		return () => document.documentElement.removeAttribute('data-living-scene');
	});
</script>

{#if config.preset !== 'legacy'}<div class="living-scene" aria-hidden="true">
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
