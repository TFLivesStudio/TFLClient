<script lang="ts">
	import { subscribeFrame } from '$lib/scenes/frameClock';
	import { untrack } from 'svelte';
	import { CharacterController } from '$lib/visual/character';
	import { scenes } from '$lib/scenes/preferences.svelte';
	import { reaction } from '$lib/scenes/reactions.svelte';
	import type { SkinViewer } from 'skinview3d';
	import { effects, runtime } from '$lib/visual/runtime.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { Box, UserRound, Hand, RotateCcw } from 'lucide-svelte';
	import { tr } from '$lib/i18n/experience';
	let {
		skinUrl = null,
		capeUrl = null,
		width,
		height,
		zoom = 0.9,
		onSkinError,
		onCapeError,
		priority = 'studio',
		launching = false,
		working = false
	}: {
		skinUrl?: string | null;
		capeUrl?: string | null;
		width: number;
		height: number;
		zoom?: number;
		onSkinError?: (error: unknown) => void;
		onCapeError?: (error: unknown) => void;
		priority?: 'hero' | 'studio';
		launching?: boolean;
		working?: boolean;
	} = $props();
	let canvas = $state<HTMLCanvasElement>(),
		container = $state<HTMLDivElement>(),
		viewer = $state.raw<SkinViewer | null>(null);
	let manual = $state(false),
		intersecting = $state(true),
		failed = $state(false);
	const character = new CharacterController();
	let dragPointer: number | null = null,
		dragX = 0,
		dragStartX = 0,
		dragged = false;
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
		character.resetView();
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
		void skinUrl;
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
					created.controls.enabled = false;
					created.autoRotate = false;
					created.playerWrapper.rotation.y = character.yaw;
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
		current.autoRotate = false;
		current.pixelRatio = Math.min(window.devicePixelRatio || 1, effects.current.dpr);
		current.zoom = zoom;
		const full = effects.current.cinematic,
			fps = Math.min(scenes.global.fps, full ? 60 : 24);
		current.animation = null;
		untrack(() => {
			character.step(0, false, false, false);
			applyPose(current);
			current.render();
		});
		return animate
			? subscribeFrame((delta) => {
					character.step(delta, true, current.playerObject.cape.visible, full);
					applyPose(current);
					current.render();
				}, fps)
			: undefined;
	});
	$effect(() => {
		const kind = priority !== 'hero' || scenes.global.reactive ? reaction.kind : 'idle';
		void reaction.sequence;
		character.setEvent(launching || kind === 'launch' ? 'launch' : working ? 'prepare' : kind);
	});
	function applyPose(current: SkinViewer) {
		const p = character.pose,
			player = current.playerObject,
			skin = player.skin;
		skin.head.rotation.set(p.headX, p.headY, 0);
		skin.body.rotation.set(p.bodyX, p.bodyY, p.bodyZ);
		skin.leftArm.rotation.set(p.leftArmX, 0, p.leftArmZ);
		skin.rightArm.rotation.set(p.rightArmX, 0, p.rightArmZ);
		skin.leftLeg.rotation.x = p.leftLegX;
		skin.rightLeg.rotation.x = p.rightLegX;
		player.position.y = p.lift;
		current.playerWrapper.rotation.y = character.yaw;
		if (player.cape.visible) {
			player.cape.rotation.x = character.cape?.pitch.value ?? 0.19;
			player.cape.rotation.z = character.cape?.roll.value ?? 0;
		}
	}
	function look(e: PointerEvent) {
		if (dragPointer === e.pointerId) {
			character.yaw += (e.clientX - dragX) * 0.012;
			dragX = e.clientX;
			dragged ||= Math.abs(e.clientX - dragStartX) > 4;
		} else if (effects.current.animateViewer && e.currentTarget instanceof HTMLElement) {
			const rect = e.currentTarget.getBoundingClientRect();
			character.look(
				(e.clientX - rect.left) / rect.width - 0.5,
				(e.clientY - rect.top) / rect.height - 0.5
			);
		}
		if (dragPointer === e.pointerId) draw();
	}
	function pointerDown(e: PointerEvent) {
		if (e.button !== 0) return;
		dragPointer = e.pointerId;
		dragX = dragStartX = e.clientX;
		dragged = false;
		character.dragging = true;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
	}
	function pointerUp(e: PointerEvent) {
		if (dragPointer !== e.pointerId) return;
		dragPointer = null;
		character.dragging = false;
	}
	function greet() {
		if (effects.current.animateViewer) character.greet();
	}
	function resetView() {
		character.resetView();
		draw();
	}
	function key(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			greet();
		} else if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
			e.preventDefault();
			character.yaw += e.key === 'ArrowLeft' ? -0.2 : 0.2;
			draw();
		} else if (e.key === 'Home') {
			e.preventDefault();
			resetView();
		}
	}
	function draw() {
		if (viewer?.renderPaused && !effects.current.animateViewer) {
			applyPose(viewer);
			viewer.render();
		}
	}
</script>

<div class="skin-view" bind:this={container} style:width={`${width}px`}>
	{#if enabled}<canvas
			bind:this={canvas}
			role="button"
			tabindex="0"
			aria-label={tr(
				'Personaje 3D: Enter para saludar, flechas para girar, Inicio para vista frontal',
				'3D character: Enter to wave, arrows to turn, Home for front view'
			)}
			onpointermove={look}
			onpointerdown={pointerDown}
			onpointerup={pointerUp}
			onpointercancel={pointerUp}
			onclick={(event) => {
				if (event.detail === 0 || !dragged) greet();
			}}
			onlostpointercapture={() => {
				dragPointer = null;
				character.dragging = false;
			}}
			onpointerenter={() => {
				if (effects.current.animateViewer) character.hover();
			}}
			onpointerleave={() => character.look(0, 0)}
			onkeydown={key}
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
	{#if enabled && viewer}<div class="character-actions">
			<button
				type="button"
				class="vr-control"
				onclick={greet}
				disabled={!effects.current.animateViewer}
				title={tr('Saludar', 'Wave')}
				aria-label={tr('Saludar', 'Wave')}><Hand size={14} /></button
			>
			<span>{tr('Arrastra para girar', 'Drag to turn')}</span>
			<button
				type="button"
				class="vr-control"
				onclick={resetView}
				title={tr('Vista frontal', 'Front view')}
				aria-label={tr('Vista frontal', 'Front view')}><RotateCcw size={14} /></button
			>
		</div>{/if}
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
		touch-action: none;
	}
	.skin-view canvas:active {
		cursor: grabbing;
	}
	.skin-view canvas:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: -3px;
	}
	.character-actions {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		width: 100%;
	}
	.character-actions span {
		font-size: 10px;
		color: var(--text-muted);
		text-align: center;
	}
	.character-actions button {
		min-width: 28px;
		min-height: 28px;
		padding: 4px;
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
