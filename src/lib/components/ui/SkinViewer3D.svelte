<script lang="ts">
	import { subscribeFrame } from '$lib/scenes/frameClock';
	import { scenes } from '$lib/scenes/preferences.svelte';
	import { reaction } from '$lib/scenes/reactions.svelte';
	import type { SkinViewer } from 'skinview3d';
	import { effects, runtime } from '$lib/visual/runtime.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { Box, UserRound } from 'lucide-svelte';
	let {
		skinUrl = null,
		capeUrl = null,
		width,
		height,
		autoRotate = true,
		autoRotateSpeed = 0.7,
		zoom = 0.9,
		onSkinError,
		onCapeError,
		priority = 'studio',
		launching = false
	}: {
		skinUrl?: string | null;
		capeUrl?: string | null;
		width: number;
		height: number;
		autoRotate?: boolean;
		autoRotateSpeed?: number;
		zoom?: number;
		onSkinError?: (error: unknown) => void;
		onCapeError?: (error: unknown) => void;
		priority?: 'hero' | 'studio';
		launching?: boolean;
	} = $props();
	let canvas = $state<HTMLCanvasElement>(),
		container = $state<HTMLDivElement>(),
		viewer = $state.raw<SkinViewer | null>(null);
	let manual = $state(false),
		intersecting = $state(true),
		failed = $state(false);
	const enabled = $derived(!!skinUrl && !failed && (effects.current.viewer || manual));
	const visible = $derived(
		intersecting &&
			runtime.visible &&
			runtime.focused &&
			!(priority === 'hero' && (runtime.overlay || runtime.modalDepth > 0))
	);
	$effect(() => {
		void skinUrl;
		failed = false;
	});
	$effect(() => {
		if (!container) return;
		const observer = new IntersectionObserver(
			(entries) => {
				intersecting = entries[0]?.isIntersecting ?? false;
			},
			{ threshold: 0.05 }
		);
		observer.observe(container);
		return () => observer.disconnect();
	});
	$effect(() => {
		if (!canvas || !enabled || !visible) return;
		const node = canvas,
			w = width,
			h = height;
		const lost = () => {
			failed = true;
		};
		node.addEventListener('webglcontextlost', lost);
		let disposed = false;
		let created: SkinViewer | null = null;
		import('skinview3d')
			.then(({ SkinViewer }) => {
				if (disposed) return;
				try {
					created = new SkinViewer({
						canvas: node,
						width: w,
						height: h,
						pixelRatio: Math.min(window.devicePixelRatio || 1, effects.current.dpr),
						renderPaused: true
					});
					created.zoom = zoom;
					created.controls.enableZoom = false;
					created.controls.enablePan = false;
					viewer = created;
				} catch (error) {
					failed = true;
					onSkinError?.(error);
				}
			})
			.catch((error) => {
				if (!disposed) {
					failed = true;
					onSkinError?.(error);
				}
			});
		return () => {
			disposed = true;
			node.removeEventListener('webglcontextlost', lost);
			if (viewer === created) viewer = null;
			created?.dispose();
		};
	});
	$effect(() => {
		const current = viewer,
			url = skinUrl;
		if (!current) return;
		let stale = false;
		Promise.resolve(url ? current.loadSkin(url) : current.loadSkin(null))
			.then(() => {
				if (!stale) current.render();
			})
			.catch((error) => {
				if (!stale) {
					failed = true;
					onSkinError?.(error);
				}
			});
		return () => {
			stale = true;
		};
	});
	$effect(() => {
		const current = viewer,
			url = capeUrl;
		if (!current) return;
		let stale = false;
		Promise.resolve(url ? current.loadCape(url) : current.loadCape(null))
			.then(() => {
				if (!stale) current.render();
			})
			.catch((error) => {
				if (!stale) onCapeError?.(error);
			});
		return () => {
			stale = true;
		};
	});
	$effect(() => {
		const current = viewer;
		if (!current) return;
		const animate = visible && effects.current.animateViewer;
		current.renderPaused = true;
		current.autoRotate = animate && autoRotate;
		current.autoRotateSpeed = autoRotateSpeed;
		current.pixelRatio = Math.min(window.devicePixelRatio || 1, effects.current.dpr);
		current.zoom = zoom;
		const cinematic = effects.current.cinematic,
			launch =
				(priority !== 'hero' || scenes.global.reactive) &&
				(launching || reaction.kind === 'launch'),
			result = priority !== 'hero' || scenes.global.reactive ? reaction.kind : 'idle',
			fps = Math.min(scenes.global.fps, cinematic ? 60 : 24);
		let stop = () => {};
		let stale = false;
		import('skinview3d').then(
			({ IdleAnimation, WalkingAnimation, WaveAnimation, HitAnimation }) => {
				if (stale || viewer !== current) return;
				current.animation =
					animate && cinematic
						? launch
							? new WalkingAnimation()
							: result === 'success'
								? new WaveAnimation()
								: result === 'error'
									? new HitAnimation()
									: new IdleAnimation()
						: null;
				if (!animate) current.render();
				else
					stop = subscribeFrame((delta) => {
						current.animation?.update(current.playerObject, delta);
						if (autoRotate) current.playerWrapper.rotation.y += delta * autoRotateSpeed;
						current.controls.update();
						current.render();
					}, fps);
			}
		);
		return () => {
			stale = true;
			stop();
		};
	});
	function look(e: PointerEvent) {
		if (priority === 'hero' && effects.current.cinematic && viewer) {
			const rect =
				e.currentTarget instanceof HTMLElement ? e.currentTarget.getBoundingClientRect() : null;
			if (rect) {
				viewer.playerObject.skin.head.rotation.y = Math.max(
					-0.35,
					Math.min(0.35, (e.clientX - rect.left) / rect.width - 0.5)
				);
				viewer.playerObject.skin.head.rotation.x = Math.max(
					-0.18,
					Math.min(0.18, (e.clientY - rect.top) / rect.height - 0.5)
				);
			}
		}
		draw();
	}
	function draw() {
		if (viewer?.renderPaused) viewer.render();
	}
</script>

<div class="skin-view" bind:this={container} style:width={`${width}px`}>
	{#if enabled}<canvas
			bind:this={canvas}
			aria-label={vr('studio')}
			onpointermove={look}
			style:height={`${height}px`}
		></canvas>
	{:else}<div
			class="static-view"
			style:height={`${height}px`}
			role="img"
			aria-label={skinUrl ? vr('skinStatic') : vr('noSkin')}
		>
			{#if skinUrl}<div class="pixel-person" style={`--skin:url("${skinUrl}")`}>
					<span class="head"></span><span class="body"></span><span class="arm left"></span><span
						class="arm right"
					></span><span class="leg left"></span><span class="leg right"></span>
				</div>{:else}<UserRound size={64} strokeWidth={1} />{/if}
		</div>{/if}
	{#if !effects.current.viewer && skinUrl}<button
			class="viewer-toggle"
			onclick={() => (manual = !manual)}
			><Box size={14} />{manual ? vr('disable3d') : vr('enable3d')}</button
		>{/if}
	{#if failed}<p class="vr-muted">{vr('noSkin')}</p>{/if}
</div>

<style>
	.skin-view {
		display: flex;
		flex-direction: column;
		align-items: center;
		max-width: 100%;
	}
	.skin-view canvas {
		max-width: 100%;
		object-fit: contain;
		cursor: grab;
	}
	.static-view {
		display: grid;
		place-items: center;
		width: 100%;
		color: var(--text-muted);
	}
	.viewer-toggle {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 7px;
		padding: 8px 10px;
		background: var(--surface-card);
		color: var(--text-primary);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		font-size: var(--text-caption);
	}
	.pixel-person {
		position: relative;
		width: 128px;
		height: 256px;
		image-rendering: pixelated;
	}
	.pixel-person span {
		position: absolute;
		background-image: var(--skin);
		background-size: 512px auto;
		image-rendering: pixelated;
	}
	.head {
		width: 64px;
		height: 64px;
		left: 32px;
		top: 0;
		background-position: -64px -64px;
	}
	.body {
		width: 64px;
		height: 96px;
		left: 32px;
		top: 64px;
		background-position: -160px -160px;
	}
	.arm {
		width: 32px;
		height: 96px;
		top: 64px;
		background-position: -352px -160px;
	}
	.arm.left {
		left: 0;
	}
	.arm.right {
		right: 0;
		transform: scaleX(-1);
	}
	.leg {
		width: 32px;
		height: 96px;
		top: 160px;
		background-position: -32px -160px;
	}
	.leg.left {
		left: 32px;
	}
	.leg.right {
		left: 64px;
		transform: scaleX(-1);
	}
</style>
