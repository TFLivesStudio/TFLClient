<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import Mascot from '$lib/components/ui/Mascot.svelte';
	import { IconButton, Tooltip } from '$lib/components/ui';
	import { getMascotFor } from '$lib/mascots';
	import { network } from '$lib/state/network.svelte';
	import { iconVersions, goHome } from '$lib/state/instanceState.svelte';
	import { appState } from '$lib/state/state.svelte';
	import {
		SIDEBAR_COLLAPSED_WIDTH,
		SIDEBAR_EXPANDED_WIDTH,
		isSidebarEffectiveCollapsed,
		isSidebarForcedCompact
	} from '$lib/state/viewport.svelte';
	import { updateSettings } from '$lib/api';
	import { t } from '$lib/i18n/index.svelte';
	import { getInstanceIconPath } from '$lib/api';
	import type { InstanceData, MinecraftUser } from '$lib/types/types';
	import {
		Plus,
		User as UserIcon,
		LogOut,
		Settings,
		Search,
		Boxes,
		Sparkles,
		WifiOff,
		Globe,
		Shirt,
		Home,
		PanelLeftClose,
		PanelLeftOpen
	} from 'lucide-svelte';

	let {
		instances,
		selected,
		user,
		onSelect,
		onCreate,
		onLogout,
		onOpenSettings,
		onOpenTflSelection,
		onJoinServer,
		onOpenSkinManager,
		onInstanceContextMenu
	}: {
		instances: InstanceData[];
		selected: InstanceData | null;
		user: MinecraftUser | null;
		onSelect: (i: InstanceData) => void;
		onCreate: () => void;
		onLogout: () => void;
		onOpenSettings: () => void;
		onOpenTflSelection: () => void;
		onJoinServer: () => void;
		onOpenSkinManager: () => void;
		onInstanceContextMenu: (instance: InstanceData, x: number, y: number) => void;
	} = $props();

	// Modo compacto (solo íconos) — persistido en Ajustes, mismo patrón
	// optimista que `multi_instance_warning_dismissed` en InstanceDetail.svelte:
	// se actualiza el estado local ya mismo y se guarda en paralelo; si falla
	// el guardado no vale la pena bloquear la UI por eso, se reintenta solo.
	// NO se reimplementa este mecanismo (Entrega B, sección 6.3) — `manualCollapsed`
	// sigue siendo la única fuente de verdad persistida, `toggleCollapsed` el
	// único lugar que la escribe.
	const manualCollapsed = $derived(appState.settings?.sidebar_collapsed === true);
	function toggleCollapsed() {
		if (!appState.settings) return;
		const next = !manualCollapsed;
		appState.settings.sidebar_collapsed = next;
		updateSettings({ ...appState.settings, sidebar_collapsed: next }).catch(() => {
			// revertir si no se pudo guardar, para no mostrar un estado que
			// después se pierde solo al reabrir el launcher
			if (appState.settings) appState.settings.sidebar_collapsed = !next;
		});
	}

	// Responsive (Entrega B, sección 4) — `collapsed` es el colapso EFECTIVO
	// que de verdad se renderiza: preferencia manual OR falta de espacio en
	// la ventana. `forcedCompact` distingue el segundo caso del primero para
	// poder deshabilitar el toggle manual cuando no tendría efecto visible
	// (ver comentario largo en viewport.svelte.ts) — así nunca se termina
	// grabando sin querer una preferencia que el usuario no pidió.
	const collapsed = $derived(isSidebarEffectiveCollapsed());
	const forcedCompact = $derived(isSidebarForcedCompact());

	const LOADER_COLOR: Record<string, string> = {
		vanilla: 'var(--loader-vanilla)',
		fabric: 'var(--loader-fabric)',
		forge: 'var(--loader-forge)',
		neoforge: 'var(--loader-neoforge)',
		quilt: 'var(--loader-quilt)'
	};

	let query = $state('');

	// Hint de la paleta de comandos (CommandPalette.svelte) — Cmd+K en
	// macOS, Ctrl+K en Windows/Linux. Solo texto, la tecla real la maneja
	// CommandPalette con e.metaKey || e.ctrlKey (agnóstico de plataforma).
	const shortcutLabel =
		typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform) ? '⌘K' : 'Ctrl K';

	// Cabeza real de cuentas premium — se pide a Mojang directo (session
	// server oficial), no a un CDN de terceros de renderizado de avatares
	// (Crafatar/mc-heads.net/etc probaron estar caídos en distintos
	// momentos de la misma semana). Se renderiza con CSS (recorte del PNG
	// de la skin real vía background-position), no hace falta canvas.
	let skinUrl = $state<string | null>(null);
	$effect(() => {
		skinUrl = null;
		if (user && user.user_type !== 'Cracked') {
			invoke<string | null>('get_skin_texture_url', { uuid: user.uuid })
				.then((url) => {
					skinUrl = url;
				})
				.catch(() => {
					skinUrl = null;
				});
		}
	});

	const sorted = $derived([...instances].sort((a, b) => b.last_played - a.last_played));
	const filtered = $derived(
		query.trim()
			? sorted.filter((i) => i.name.toLowerCase().includes(query.trim().toLowerCase()))
			: sorted
	);

	// La lista mostraba solo la inicial — nunca pedía el ícono real de cada
	// instancia. Se pide una vez por instancia y se vuelve a pedir cuando
	// cambia su versión de ícono (ver `bumpInstanceIcon`), para que un ícono
	// nuevo se vea sin reiniciar el launcher.
	let iconUrls = $state<Record<string, string | null>>({});
	const iconLoaded: Record<string, string> = {};
	$effect(() => {
		for (const inst of instances) {
			const version = iconVersions[inst.name] ?? 0;
			const key = `${inst.name}:${version}`;
			if (iconLoaded[inst.uuid] === key) continue;
			iconLoaded[inst.uuid] = key;
			getInstanceIconPath(inst.name)
				.then((path) => {
					iconUrls[inst.uuid] = path ? `${convertFileSrc(path)}?v=${version}` : null;
				})
				.catch(() => {
					iconUrls[inst.uuid] = null;
				});
		}
	});
</script>

<aside
	class="sidebar"
	class:collapsed
	style="width: {collapsed ? SIDEBAR_COLLAPSED_WIDTH : SIDEBAR_EXPANDED_WIDTH}px"
>
	<div class="sidebar-top">
		<!-- Arriba y no al pie: el banner de actualización lista
			 (UpdateBadge.svelte) es "position: fixed; bottom" y tapa esa franja
			 del sidebar cuando hay una actualización — abajo quedaba inútil. -->
		<IconButton
			label={collapsed ? t('sidebar.expand') : t('sidebar.collapse')}
			variant="ghost"
			size="compact"
			disabled={forcedCompact}
			onclick={toggleCollapsed}
		>
			{#if collapsed}
				<PanelLeftOpen size={16} />
			{:else}
				<PanelLeftClose size={16} />
			{/if}
		</IconButton>
	</div>

	{#if collapsed}
		<Tooltip content={t('sidebar.home')} position="right">
			<button
				type="button"
				class="home-btn"
				class:active={selected === null}
				onclick={goHome}
				aria-label={t('sidebar.home')}
			>
				<Home size={16} />
			</button>
		</Tooltip>
	{:else}
		<button
			type="button"
			class="home-btn"
			class:active={selected === null}
			onclick={goHome}
			aria-label={t('sidebar.home')}
		>
			<Home size={16} />
			<span>{t('sidebar.home')}</span>
		</button>
	{/if}

	{#if collapsed}
		<Tooltip content="TFL Selection" position="right">
			<button
				type="button"
				class="tfl-selection-btn"
				onclick={onOpenTflSelection}
				aria-label="TFL Selection"
			>
				<Sparkles size={16} />
			</button>
		</Tooltip>
	{:else}
		<button
			type="button"
			class="tfl-selection-btn"
			onclick={onOpenTflSelection}
			aria-label="TFL Selection"
		>
			<Sparkles size={16} />
			TFL Selection
		</button>
	{/if}

	{#if !network.online && !collapsed}
		<div class="offline-badge">
			<WifiOff size={12} />
			<span>{t('sidebar.offline')}</span>
		</div>
	{/if}

	<div class="instances">
		<div class="section-label">
			{#if !collapsed}<span>{t('sidebar.yourInstances')}</span>{/if}
			<div class="section-label-actions">
				<IconButton
					label={t('sidebar.joinServer')}
					variant="secondary"
					size="compact"
					onclick={onJoinServer}
				>
					<Globe size={16} strokeWidth={2.25} />
				</IconButton>
				<IconButton
					label={t('sidebar.createInstance')}
					variant="secondary"
					size="compact"
					onclick={onCreate}
				>
					<Plus size={16} strokeWidth={2.25} />
				</IconButton>
			</div>
		</div>

		{#if instances.length > 0 && !collapsed}
			<div class="search-box">
				<Search size={13} />
				<input type="text" placeholder={t('sidebar.search')} bind:value={query} />
				<kbd class="shortcut-hint">{shortcutLabel}</kbd>
			</div>
		{/if}

		<div class="instance-list">
			{#if instances.length === 0}
				<div class="empty">
					<Boxes size={28} />
					{#if !collapsed}<p>{t('sidebar.noInstancesYet')}</p>{/if}
					{#if collapsed}
						<Tooltip content={t('sidebar.createFirst')} position="right">
							<button
								type="button"
								class="empty-create"
								onclick={onCreate}
								aria-label={t('sidebar.createFirst')}
							>
								<Plus size={13} strokeWidth={2.5} />
							</button>
						</Tooltip>
					{:else}
						<button
							type="button"
							class="empty-create"
							onclick={onCreate}
							aria-label={t('sidebar.createFirst')}
						>
							<Plus size={13} strokeWidth={2.5} />
							{t('sidebar.createFirst')}
						</button>
					{/if}
				</div>
			{:else if filtered.length === 0}
				{#if !collapsed}<p class="empty-search">{t('sidebar.noResultsFor', { query })}</p>{/if}
			{:else}
				{#each filtered as instance (instance.uuid)}
					<Tooltip content={instance.name} position="right" disabled={!collapsed}>
						<button
							type="button"
							class="instance-item"
							class:active={selected?.uuid === instance.uuid}
							style="--loader-color: {LOADER_COLOR[instance.loader]}"
							aria-label={instance.name}
							onclick={() => onSelect(instance)}
							oncontextmenu={(e) => {
								e.preventDefault();
								onInstanceContextMenu(instance, e.clientX, e.clientY);
							}}
							onmousedown={(e) => {
								// El evento "contextmenu" del click derecho no dispara de
								// forma confiable en esta webview (se probó en vivo: ni
								// siquiera aparece el menú nativo del sistema, algo se lo
								// come antes de llegar al JS). "mousedown" con botón
								// derecho es más bajo nivel y sí llega siempre — queda
								// como camino principal, oncontextmenu como refuerzo si
								// en algún entorno sí dispara.
								if (e.button === 2) {
									e.preventDefault();
									onInstanceContextMenu(instance, e.clientX, e.clientY);
								}
							}}
						>
							<span class="instance-avatar">
								{#if iconUrls[instance.uuid]}
									<img src={iconUrls[instance.uuid]} alt="" />
								{:else}
									{instance.name.charAt(0).toUpperCase()}
								{/if}
							</span>
							{#if !collapsed}
								<span class="instance-text">
									<span class="instance-name">{instance.name}</span>
									<span class="instance-version">{instance.mc_version} · {instance.loader}</span>
								</span>
							{/if}
						</button>
					</Tooltip>
				{/each}
			{/if}
		</div>
	</div>

	{#if user}
		<div class="user-chip">
			<span class="user-head" title={collapsed ? user.username : undefined}>
				{#if user.user_type === 'Cracked'}
					<!-- Cuentas offline no tienen skin real — mascota simple en
						 vez del Steve genérico que devolvería Crafatar por
						 defecto. Elegida al azar (determinística por UUID) al
						 crear la cuenta, cambiable desde Ajustes. -->
					<Mascot id={getMascotFor(user.uuid)} size={32} />
				{:else}
					<UserIcon size={16} class="user-head-fallback" />
					{#if skinUrl}
						<span class="skin-head">
							<span class="skin-layer base" style="background-image: url({skinUrl})"></span>
							<span class="skin-layer overlay" style="background-image: url({skinUrl})"></span>
						</span>
					{/if}
				{/if}
			</span>
			{#if !collapsed}
				<div class="user-info">
					<span class="user-name">{user.username}</span>
					<span class="user-type"
						>{user.user_type === 'Cracked' ? t('sidebar.offlineAccountType') : user.user_type}</span
					>
				</div>
			{/if}
			<div class="user-actions">
				{#if user.user_type !== 'Cracked'}
					<IconButton
						label={t('skinManager.title')}
						variant="ghost"
						size="compact"
						onclick={onOpenSkinManager}
					>
						<Shirt size={16} />
					</IconButton>
				{/if}
				<IconButton
					label={t('sidebar.openSettings')}
					variant="ghost"
					size="compact"
					onclick={onOpenSettings}
				>
					<Settings size={16} />
				</IconButton>
				<IconButton label={t('sidebar.logout')} variant="ghost" size="compact" onclick={onLogout}>
					<LogOut size={16} />
				</IconButton>
			</div>
		</div>
	{/if}
</aside>

<style>
	.sidebar {
		position: relative;
		z-index: var(--z-content);
		/* El ancho real lo pone el style="width: ...px" inline del <aside>
		   (SIDEBAR_COLLAPSED_WIDTH/SIDEBAR_EXPANDED_WIDTH, viewport.svelte.ts)
		   en vez de una regla CSS fija acá — TitleBar.svelte importa las
		   MISMAS constantes para su bloque de marca, así los dos nunca pueden
		   desalinearse al cambiar de modo (Entrega B, sección 6.2). No le
		   agregues un `width` acá de nuevo. */
		flex-shrink: 0;
		overflow-x: hidden;
		background: var(--surface-workspace);
		border-right: 1px solid var(--border);
		display: flex;
		flex-direction: column;
		padding: 14px 12px;
		gap: 12px;
		transition: width var(--duration-panel) var(--easing-standard);
	}

	/* Modo compacto: solo íconos — tanto por preferencia manual
	   (appState.settings.sidebar_collapsed) como forzado por falta de
	   espacio en la ventana (ver `collapsed`/`forcedCompact` arriba). */
	.sidebar.collapsed {
		padding-left: 10px;
		padding-right: 10px;
		/* El banner de actualización lista es "position: fixed; bottom: 16px"
		   sobre TODA la ventana, no solo sobre el contenido — en este modo
		   angosto los íconos de usuario (skin/ajustes/cerrar sesión) quedan
		   apilados justo en esa franja. Se les deja aire abajo para que no
		   queden tapados mientras el banner esté visible. */
		padding-bottom: 88px;
		align-items: center;
	}

	/* El wrapper de <Tooltip> (Entrega A) es `display:inline-flex;
	   flex-direction:row` por defecto — como flex item de un contenedor
	   columna (.sidebar/.instance-list) se ESTIRA en ancho (align-items:
	   stretch, el default), pero su propio hijo (el botón trigger) NO se
	   estira solo porque el wrapper es row: el eje principal de un row no
	   estira, solo el cruzado. Sin esto el botón envuelto queda del ancho de
	   su contenido en vez de llenar la fila como antes de agregar el
	   Tooltip. Afecta solo a los triggers que envolvemos con Tooltip en modo
	   compacto (home, TFL Selection, instancias, "crear primera instancia");
	   la burbuja del tooltip (si está abierta) es "position: absolute" así
	   que no la toca este flex. */
	.sidebar :global(.tfl-tooltip-wrap) {
		display: flex;
		width: 100%;
	}
	.sidebar :global(.tfl-tooltip-wrap > *) {
		flex: 1;
		min-width: 0;
	}

	.home-btn,
	.tfl-selection-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		min-height: var(--control-size-normal);
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid color-mix(in srgb, var(--accent) 35%, var(--border));
		background: color-mix(in srgb, var(--accent) 12%, var(--surface-card));
		color: var(--accent);
		font-size: var(--text-small);
		line-height: var(--line-height-small);
		font-weight: 700;
		cursor: pointer;
		transition:
			background var(--duration-micro) var(--easing-standard),
			border-color var(--duration-micro) var(--easing-standard);
	}

	.home-btn {
		border-color: var(--border);
		background: var(--surface-card);
		color: var(--text-secondary);
	}

	.home-btn.active {
		border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
		background: color-mix(in srgb, var(--accent) 14%, var(--surface-card));
		color: var(--accent);
	}

	.home-btn:hover,
	.tfl-selection-btn:hover {
		background: color-mix(in srgb, var(--accent) 20%, var(--surface-card));
		border-color: var(--accent);
	}

	.sidebar.collapsed .home-btn,
	.sidebar.collapsed .tfl-selection-btn {
		justify-content: center;
		padding: 8px;
	}

	.offline-badge {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid color-mix(in srgb, #f59e0b 35%, var(--border));
		background: color-mix(in srgb, #f59e0b 12%, var(--bg-card));
		color: #f59e0b;
		font-size: 0.68rem;
		line-height: 1.3;
	}

	.offline-badge :global(svg) {
		flex-shrink: 0;
	}

	.instances {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
		gap: 8px;
	}

	.section-label {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.5px;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.sidebar.collapsed .section-label {
		justify-content: center;
	}

	.section-label-actions {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	/* Los IconButton de Entrega A son --control-size-compact (32px) — dos uno
	   al lado del otro no entran en el ancho interior del riel colapsado
	   (SIDEBAR_COLLAPSED_WIDTH 76px menos padding). Se apilan en vez de
	   quedar lado a lado, para no desbordar horizontalmente (sección 4 del
	   encargo: "nada de overflow horizontal en ningún tamaño"). */
	.sidebar.collapsed .section-label-actions {
		flex-direction: column;
	}

	.search-box {
		display: flex;
		align-items: center;
		gap: 6px;
		height: var(--control-size-compact);
		padding: 0 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-muted);
	}

	.search-box input {
		flex: 1;
		background: transparent;
		border: none;
		outline: none;
		font-size: var(--text-small);
		color: var(--text-primary);
	}

	.shortcut-hint {
		font-size: 0.62rem;
		font-weight: 700;
		color: var(--text-muted);
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 2px 5px;
		flex-shrink: 0;
	}

	.instance-list {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 6px;
		/* Mejor hipótesis para la "barra que parpadea arriba de cuentas" al
		   pasar el cursor rápido por varias instancias, sin poder confirmar
		   con una captura: la scrollbar nativa de esta lista (justo arriba
		   de .user-chip) aparece/desaparece en el hover del contenedor en
		   algunos navegadores/SO. Se oculta visualmente, el scroll sigue
		   andando igual — mismo patrón que .tabs en InstanceDetail.svelte. */
		scrollbar-width: none;
	}

	.instance-list::-webkit-scrollbar {
		display: none;
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: 8px;
		color: var(--text-muted);
		padding: 28px 8px;
	}

	.empty p {
		font-size: 0.75rem;
	}

	.empty-create {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		margin-top: 4px;
		min-height: var(--control-size-compact);
		padding: 6px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		font-weight: 600;
		cursor: pointer;
	}

	.empty-search {
		font-size: 0.75rem;
		color: var(--text-muted);
		padding: 8px 2px;
	}

	.instance-item {
		position: relative;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px 8px 12px;
		border-radius: var(--border-radius);
		border: 1px solid transparent;
		background: var(--surface-card);
		color: var(--text-primary);
		cursor: pointer;
		text-align: left;
		/* La barrita de color (::before, ver abajo) es un rectángulo recto
		   asomando desde el borde izquierdo — sin esto sobresale de las
		   esquinas redondeadas de la card, se ve "cortada" en vez de
		   completa. */
		overflow: hidden;
		transition:
			background var(--duration-micro) var(--easing-standard),
			border-color var(--duration-micro) var(--easing-standard),
			transform var(--duration-micro) var(--easing-standard);
	}

	.instance-item::before {
		content: '';
		position: absolute;
		left: 0;
		top: 8px;
		bottom: 8px;
		width: 3px;
		border-radius: 3px;
		background: var(--loader-color);
		opacity: 0;
		transition: opacity var(--duration-micro) var(--easing-standard);
	}

	.instance-item:hover {
		border-color: var(--border);
		/* Antes tenía "transform: translateX(1px)" acá — combinado con el
		   ícono recortado en esquinas redondeadas (overflow: hidden más
		   arriba), algunos motores (Chromium/WebView2) repintan mal el
		   compositing al mover el elemento: la imagen "pierde" un pedazo y
		   se ve el fondo oscuro de atrás en su lugar. Era solo un detalle
		   cosmético, no vale la pena el riesgo. */
	}

	.instance-item.active {
		background: var(--surface-interactive);
		border-color: color-mix(in srgb, var(--loader-color) 40%, var(--border));
	}

	.instance-item.active::before {
		opacity: 1;
	}

	.sidebar.collapsed .instance-item {
		justify-content: center;
		padding: 8px;
	}

	.instance-avatar {
		flex-shrink: 0;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		font-size: 0.8rem;
		font-weight: 800;
		color: var(--loader-color);
		background: color-mix(in srgb, var(--loader-color) 18%, transparent);
	}

	.instance-avatar img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.instance-text {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}

	.instance-name {
		font-size: var(--text-small);
		line-height: var(--line-height-small);
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.instance-version {
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.user-chip {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--surface-card);
	}

	.user-head {
		position: relative;
		flex-shrink: 0;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		overflow: hidden;
		background: var(--bg-input);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
	}

	.user-head :global(.user-head-fallback) {
		position: absolute;
	}

	.skin-head {
		position: relative;
		display: block;
		width: 32px;
		height: 32px;
	}

	/* Recorte de la textura de skin real (64x64) a la cara, escalada 4x
	   (256/64) — base primero, capa "hat" overlay encima. Sin canvas. */
	.skin-layer {
		position: absolute;
		inset: 0;
		background-repeat: no-repeat;
		background-size: 256px 256px;
		image-rendering: pixelated;
	}

	.skin-layer.base {
		background-position: -32px -32px;
	}

	.skin-layer.overlay {
		background-position: -160px -32px;
	}

	.user-info {
		display: flex;
		flex-direction: column;
		min-width: 0;
		flex: 1;
	}

	.user-name {
		font-size: 0.8rem;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.user-type {
		font-size: 0.62rem;
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.user-actions {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.sidebar.collapsed .user-chip {
		flex-direction: column;
		padding: 8px 4px;
		gap: 8px;
	}

	.sidebar.collapsed .user-actions {
		flex-wrap: wrap;
		justify-content: center;
	}

	.sidebar-top {
		display: flex;
		justify-content: flex-end;
	}

	.sidebar.collapsed .sidebar-top {
		justify-content: center;
	}
</style>
