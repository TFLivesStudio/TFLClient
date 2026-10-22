<script lang="ts">
	import { cinematicPoster } from '$lib/scenes/cinematic';
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import { invoke } from '$lib/api/invoke';
	import { appState } from '$lib/state/state.svelte';
	import { appearance } from '$lib/state/appearanceState.svelte';
	import { openSettingsAt } from '$lib/state/uiState.svelte';
	import { runtime } from '$lib/visual/runtime.svelte';
	import { SCENES, DEFAULT_SCENE, type SceneConfig } from '$lib/scenes/model';
	import { scenes, saveScene, inheritScene, sceneStatus } from '$lib/scenes/preferences.svelte';
	import SceneStage from '$lib/scenes/SceneStage.svelte';
	import SettingGroup from './SettingGroup.svelte';
	import { tr } from '$lib/i18n/experience';
	import { i18nState } from '$lib/i18n/index.svelte';
	import { Play, Pause, Upload, Trash2, Check, RotateCcw } from 'lucide-svelte';
	let { children }: { children: Snippet } = $props();
	let scope = $state('global');
	let draft = $state<SceneConfig>(structuredClone(DEFAULT_SCENE)),
		playing = $state(false),
		error = $state(''),
		busy = $state(false),
		saved = $state(false);
	type Media = { id: string; path: string; bytes: number; mime: string };
	let media = $state<Media[]>([]);
	const locale = $derived(i18nState.locale === 'en' ? 1 : 0);
	$effect(() => {
		const current = scope === 'global' ? scenes.global : (scenes.instances[scope] ?? scenes.global);
		draft = structuredClone($state.snapshot(current));
		playing = false;
		saved = false;
	});
	const dirty = $derived(
		JSON.stringify(draft) !==
			JSON.stringify(
				scope === 'global' ? scenes.global : (scenes.instances[scope] ?? scenes.global)
			)
	);
	async function load() {
		try {
			media = await invoke<Media[]>('list_scene_media');
		} catch (e) {
			error = String(e);
		}
	}
	onMount(load);
	async function importVideo() {
		busy = true;
		error = '';
		try {
			const path = await invoke<string | null>('pick_scene_video_file');
			if (!path) return;
			const item = await invoke<Media>('import_scene_video', { sourcePath: path });
			await load();
			draft.mediaId = item.id;
			if (draft.preset === 'legacy') draft.preset = 'aurora';
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
	async function remove(id: string) {
		const used = [scenes.global, ...Object.values(scenes.instances)].some((s) => s.mediaId === id);
		if (used) {
			error = tr(
				'Este vídeo está en uso. Aplica primero otra escena para eliminarlo.',
				'This video is in use. Apply another scene before deleting it.'
			);
			return;
		}
		try {
			await invoke('remove_scene_media', { id });
			if (draft.mediaId === id) draft.mediaId = null;
			await load();
		} catch (e) {
			error = String(e);
		}
	}
	function apply() {
		saveScene($state.snapshot(draft), scope === 'global' ? null : scope);
		playing = false;
		saved = !sceneStatus.error;
	}
	function cancel() {
		draft = structuredClone(
			$state.snapshot(
				scope === 'global' ? scenes.global : (scenes.instances[scope] ?? scenes.global)
			)
		);
		playing = false;
	}
</script>

<div class="scene-settings">
	<SettingGroup
		id="scene-current"
		hint={tr(
			'Previsualiza antes de aplicar. El fondo se pausa mientras editas para evitar dos escenas activas.',
			'Preview before applying. The background pauses while you edit to avoid two active scenes.'
		)}
	>
		<div class="preview">
			<SceneStage
				config={draft}
				quality={appearance.quality}
				reduced={appearance.reduceMotion || appearance.noInfiniteFx || runtime.systemReducedMotion}
				active={playing && runtime.visible && runtime.focused}
				preview
			/>
			<div class="preview-caption">
				<span>{tr('Vista previa', 'Preview')} · {appearance.quality}</span><button
					type="button"
					class="vr-control"
					onclick={() => (playing = !playing)}
					disabled={appearance.quality === 'lite'}
					>{#if playing}<Pause size={14} />{tr('Pausar', 'Pause')}{:else}<Play size={14} />{tr(
							'Reproducir',
							'Play'
						)}{/if}</button
				>
			</div>
		</div>
		<p class="hint">
			{appearance.quality === 'lite'
				? tr(
						'Lite usa la imagen estática. No crea vídeo ni WebGL.',
						'Lite uses the static image. No video or WebGL is created.'
					)
				: tr(
						'El vídeo necesita un formato compatible con el sistema. Si falla, conservamos el fondo estático.',
						'Video needs a system-supported format. If it fails, the static background remains.'
					)}
		</p>
		{#if appearance.noInfiniteFx && appearance.quality !== 'lite'}<p class="hint">
				{tr(
					'Los efectos continuos están detenidos en Apariencia. Actívalos allí para reproducir vídeo.',
					'Continuous effects are stopped in Appearance. Enable them there to play video.'
				)}
				<button type="button" class="vr-control" onclick={() => openSettingsAt('motion')}
					>{tr('Movimiento y efectos', 'Motion & effects')}</button
				>
			</p>{/if}
		<div class="actions">
			<button class="primary" type="button" disabled={!dirty} onclick={apply}
				><Check size={14} />{tr('Aplicar escena', 'Apply scene')}</button
			><button type="button" class="vr-control" disabled={!dirty} onclick={cancel}
				>{tr('Cancelar', 'Cancel')}</button
			><button
				type="button"
				class="vr-control"
				onclick={() => {
					draft = structuredClone(DEFAULT_SCENE);
					playing = false;
				}}><RotateCcw size={14} />{tr('Restablecer', 'Reset')}</button
			>{#if saved}<span role="status">{tr('Guardado', 'Saved')}</span>{/if}
		</div>
	</SettingGroup>
	<SettingGroup id="scene-gallery"
		><div class="gallery">
			{#each SCENES as scene (scene.id)}<button
					type="button"
					class="scene-tile"
					aria-pressed={draft.preset === scene.id}
					onclick={() => {
						draft.preset = scene.id;
						draft.poster = 'auto';
						draft.mediaId = null;
					}}
					><span
						class="thumb"
						style:background={`linear-gradient(145deg,${scene.colors[0]},${scene.colors[1]})`}
						>{#if scene.id !== 'legacy'}<img
								src={cinematicPoster(scene.id, true) ?? `/scenes/${scene.id}.svg`}
								alt=""
								loading="lazy"
							/>{/if}{#if draft.preset === scene.id}<b><Check size={14} /></b>{/if}</span
					><strong>{scene.name[locale]}</strong><small>{scene.description[locale]}</small></button
				>{/each}
		</div></SettingGroup
	>
	<SettingGroup
		id="scene-static"
		hint={tr(
			'Estos fondos se muestran al elegir «Tu fondo actual». Su cambio se guarda al seleccionarlo.',
			'These backgrounds appear when “Your current background” is selected. Selection saves them immediately.'
		)}
		><button
			type="button"
			class="vr-control"
			onclick={() => {
				draft.preset = 'legacy';
				draft.mediaId = null;
			}}>{tr('Usar fondo estático', 'Use static background')}</button
		>
		<div class="wallpapers">{@render children()}</div></SettingGroup
	>
	<SettingGroup
		id="scene-video"
		hint={tr(
			'Vídeos locales MP4 (H.264) y WebM. Sin sonido, en bucle. Máximo 128 MB por vídeo y 12 vídeos; no descargamos códecs.',
			'Local MP4 (H.264) and WebM videos. Muted, looping. Up to 128 MB each and 12 videos; no codec downloads.'
		)}
	>
		<label class="field"
			>{tr('Imagen de respaldo', 'Fallback image')}<select bind:value={draft.poster}
				><option value="auto">{tr('Imagen de la escena', 'Scene image')}</option><option
					value="custom">{tr('Mi fondo estático', 'My static wallpaper')}</option
				>{#each Array.from({ length: 13 }, (_, i) => `mc-${i + 1}`) as id (id)}<option value={id}
						>Minecraft · {id.slice(3)}</option
					>{/each}</select
			></label
		>
		<label class="field"
			>{tr('Fuente de vídeo', 'Video source')}<select
				value={draft.mediaId ?? ''}
				onchange={(e) => {
					draft.mediaId = e.currentTarget.value || null;
					if (draft.mediaId && draft.preset === 'legacy') draft.preset = 'aurora';
				}}
				><option value="">{tr('Sin vídeo', 'No video')}</option><option value="builtin-aurora"
					>Aurora Drift · {tr('incluido', 'included')}</option
				><option value="builtin-embers">Emberlands · {tr('incluido', 'included')}</option
				>{#each media as item (item.id)}<option value={item.id}
						>{tr('Vídeo local', 'Local video')} · {item.id.slice(-6)} · {(
							item.bytes / 1048576
						).toFixed(1)} MB</option
					>{/each}</select
			></label
		>
		<div class="actions">
			<button
				type="button"
				class="vr-control"
				disabled={busy || appState.settings?.native_dialog_mode !== 'manual'}
				onclick={importVideo}
				><Upload size={14} />{busy
					? tr('Importando…', 'Importing…')
					: tr('Importar vídeo', 'Import video')}</button
			>
		</div>
		{#if appState.settings?.native_dialog_mode !== 'manual'}<p class="hint">
				{tr(
					'Selecciona diálogos manuales en General para importar archivos.',
					'Choose manual dialogs in General to import files.'
				)}
			</p>{/if}
		{#each media as item (item.id)}<div class="media-item">
				<span>{item.id.slice(-6)} · {(item.bytes / 1048576).toFixed(1)} MB</span><button
					type="button"
					class="vr-control"
					aria-label={`${tr('Eliminar vídeo', 'Delete video')} ${item.id.slice(-6)}`}
					onclick={() => remove(item.id)}><Trash2 size={13} /></button
				>
			</div>{/each}
	</SettingGroup>
	<SettingGroup
		id="scene-atmosphere"
		hint={tr(
			'Ambiente decorativo local. La hora usa el reloj de tu equipo; no usamos tu ubicación ni datos meteorológicos.',
			'Local decorative atmosphere. Time uses your device clock; no location or weather data is used.'
		)}
	>
		<label class="field"
			>{tr('Clima decorativo', 'Decorative weather')}<select bind:value={draft.weather}
				><option value="clear">{tr('Despejado', 'Clear')}</option><option value="mist"
					>{tr('Niebla', 'Mist')}</option
				><option value="rain">{tr('Lluvia', 'Rain')}</option><option value="snow"
					>{tr('Nieve', 'Snow')}</option
				></select
			></label
		>
		{#each [['particles', tr('Partículas suaves', 'Gentle particles')], ['adaptTime', tr('Adaptar al momento del día', 'Adapt to time of day')], ['lighting', tr('Iluminación ambiental', 'Ambient lighting')], ['reactive', tr('Reaccionar a lanzamientos y descargas', 'React to launches and downloads')]] as [key, label] (key)}<label
				class="toggle"
				><span>{label}</span><input
					type="checkbox"
					checked={draft[key as 'particles' | 'adaptTime' | 'lighting' | 'reactive']}
					onchange={(e) =>
						(draft[key as 'particles' | 'adaptTime' | 'lighting' | 'reactive'] =
							e.currentTarget.checked)}
				/></label
			>{/each}
	</SettingGroup>
	<SettingGroup
		id="scene-playback"
		hint={tr(
			'Los límites de FPS se aplican a efectos procedurales y al personaje. El vídeo conserva sus FPS originales.',
			'FPS limits apply to procedural effects and the character. Video keeps its encoded frame rate.'
		)}
	>
		<label class="field"
			>{tr('Movimiento', 'Motion')}<select bind:value={draft.motion}
				><option value="off">{tr('Estático', 'Static')}</option><option value="subtle"
					>{tr('Suave', 'Subtle')}</option
				><option value="full">{tr('Completo', 'Full')}</option></select
			></label
		>
		<label class="field"
			>{tr('Límite de efectos', 'Effect limit')}<select bind:value={draft.fps}
				>{#each [15, 24, 30, 60] as fps (fps)}<option value={fps}>{fps} FPS</option>{/each}</select
			></label
		>
		<label class="field"
			>{tr('Mientras juegas', 'While playing')}<select bind:value={draft.gamePolicy}
				><option value="pause">{tr('Pausar todo', 'Pause all')}</option><option value="reduce"
					>{tr('Reducir efectos', 'Reduce effects')}</option
				><option value="keep">{tr('Mantener el perfil', 'Keep profile')}</option></select
			></label
		>
		<label class="toggle"
			><span>{tr('Permitir vídeo en Balanced', 'Allow video in Balanced')}</span><input
				type="checkbox"
				bind:checked={draft.balancedVideo}
			/></label
		><label class="toggle"
			><span
				>{tr('Luz procedural con WebGL · Experience', 'Procedural WebGL light · Experience')}</span
			><input type="checkbox" bind:checked={draft.shader} /></label
		>
	</SettingGroup>
	<SettingGroup
		id="scene-instance"
		hint={tr(
			'Las instancias heredan el fondo global hasta que guardas una escena propia. Elegir un destino descarta la vista previa sin aplicar.',
			'Instances inherit the global background until you save their own scene. Changing target discards unapplied previews.'
		)}
		><label class="field"
			>{tr('Aplicar a', 'Apply to')}<select bind:value={scope}
				><option value="global">{tr('Todo el launcher', 'Whole launcher')}</option
				>{#each appState.instances as instance (instance.uuid)}<option value={instance.uuid}
						>{instance.name}</option
					>{/each}</select
			></label
		>{#if scope !== 'global'}<p class="hint">
				{scenes.instances[scope]
					? tr('Esta instancia tiene una escena propia.', 'This instance has its own scene.')
					: tr(
							'Esta instancia hereda la escena global.',
							'This instance inherits the global scene.'
						)}
			</p>
			<button
				type="button"
				class="vr-control"
				disabled={!scenes.instances[scope]}
				onclick={() => inheritScene(scope)}>{tr('Usar escena global', 'Use global scene')}</button
			>{/if}</SettingGroup
	>
	{#if dirty}<div class="pending-scene">
			<span>{tr('Vista previa sin aplicar', 'Unapplied preview')}</span><button
				class="vr-control"
				type="button"
				onclick={cancel}>{tr('Cancelar', 'Cancel')}</button
			><button class="primary" type="button" onclick={apply}
				>{tr('Aplicar escena', 'Apply scene')}</button
			>
		</div>{/if}
	{#if error || sceneStatus.error}<p role="alert" class="error">
			{error ||
				tr(
					'No se pudo guardar la escena. Revisa el almacenamiento local.',
					'Could not save the scene. Check local storage.'
				)}
		</p>{/if}
</div>

<style>
	.scene-settings {
		display: grid;
		gap: 28px;
	}
	.preview {
		height: 200px;
		border-radius: var(--border-radius);
		position: relative;
		overflow: hidden;
		border: 1px solid var(--border);
	}
	.preview-caption {
		position: absolute;
		inset: auto 0 0;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px;
		color: white;
		background: linear-gradient(transparent, #0a1123db);
		font-size: 12px;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		align-items: center;
		margin-top: 14px;
		font-size: 12px;
	}
	.primary,
	.vr-control {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
	}
	.primary {
		border: 0;
		background: var(--accent);
		color: var(--accent-text, #111);
		border-radius: var(--border-radius-sm);
		padding: 9px 12px;
		font-weight: 650;
		cursor: pointer;
	}
	button:disabled {
		opacity: 0.45;
		cursor: default;
	}
	.gallery {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 12px;
	}
	.scene-tile {
		display: flex;
		flex-direction: column;
		text-align: left;
		gap: 6px;
		min-width: 0;
		padding: 8px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		background: var(--surface-card);
		color: var(--text-primary);
		cursor: pointer;
	}
	.scene-tile[aria-pressed='true'] {
		border-color: var(--accent);
		box-shadow: 0 0 0 1px var(--accent);
	}
	.thumb {
		display: block;
		width: 100%;
		height: 76px;
		position: relative;
		overflow: hidden;
		border-radius: var(--border-radius-sm);
	}
	.thumb img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.thumb b {
		position: absolute;
		right: 6px;
		bottom: 6px;
		border-radius: 50%;
		background: var(--accent);
		color: var(--accent-text);
		padding: 3px;
	}
	.scene-tile strong {
		font-size: 12px;
	}
	.scene-tile small,
	.hint {
		font-size: 11px;
		color: var(--text-secondary);
		line-height: 1.6;
	}
	.wallpapers {
		margin-top: 16px;
	}
	.field {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 12px;
		min-height: 48px;
		font-size: 12px;
	}
	select {
		padding: 8px;
		min-width: 130px;
		max-width: 65%;
		background: var(--bg-input);
		color: var(--text-primary);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
	}
	.toggle {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		min-height: 38px;
		font-size: 12px;
	}
	input[type='checkbox'] {
		accent-color: var(--accent);
		width: 17px;
		height: 17px;
	}
	.media-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 12px;
		border-bottom: 1px solid var(--border);
		padding: 6px;
	}
	.error {
		font-size: 12px;
		color: var(--danger, #ec7777);
	}
	@media (max-width: 900px) {
		.gallery {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
		.preview {
			height: 170px;
		}
	}
	.pending-scene {
		position: sticky;
		bottom: 0;
		z-index: 2;
		background: var(--surface-card);
		border: 1px solid var(--border);
		padding: 10px;
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
	}
	.pending-scene span {
		flex: 1;
		font-size: 11px;
		color: var(--text-secondary);
	}
</style>
