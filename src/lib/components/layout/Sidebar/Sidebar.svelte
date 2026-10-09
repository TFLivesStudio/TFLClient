<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import Mascot from '$lib/components/ui/Mascot.svelte';
	import { getMascotFor } from '$lib/mascots';
	import { network } from '$lib/state/network.svelte';
	import { iconVersions, goHome } from '$lib/state/instanceState.svelte';
	import { appState } from '$lib/state/state.svelte';
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
	const collapsed = $derived(appState.settings?.sidebar_collapsed === true);
	function toggleCollapsed() {
		if (!appState.settings) return;
		const next = !collapsed;
		appState.settings.sidebar_collapsed = next;
		updateSettings({ ...appState.settings, sidebar_collapsed: next }).catch(() => {
			// revertir si no se pudo guardar, para no mostrar un estado que
			// después se pierde solo al reabrir el launcher
			if (appState.settings) appState.settings.sidebar_collapsed = !next;
		});
	}

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

<aside class="sidebar" class:collapsed>
	<div class="sidebar-top">
		<!-- Arriba y no al pie: el banner de actualización lista
			 (UpdateBadge.svelte) es "position: fixed; bottom" y tapa esa franja
			 del sidebar cuando hay una actualización — abajo quedaba inútil. -->
		<button
			type="button"
			class="collapse-toggle"
			onclick={toggleCollapsed}
			title={collapsed ? t('sidebar.expand') : t('sidebar.collapse')}
			aria-label={collapsed ? t('sidebar.expand') : t('sidebar.collapse')}
		>
			{#if collapsed}
				<PanelLeftOpen size={15} />
			{:else}
				<PanelLeftClose size={15} />
			{/if}
		</button>
	</div>

	<button
		type="button"
		class="home-btn"
		class:active={selected === null}
		onclick={goHome}
		title={t('sidebar.home')}
		aria-label={t('sidebar.home')}
	>
		<Home size={15} />
		{#if !collapsed}<span>{t('sidebar.home')}</span>{/if}
	</button>

	<button
		type="button"
		class="tfl-selection-btn"
		onclick={onOpenTflSelection}
		title={collapsed ? 'TFL Selection' : undefined}
	>
		<Sparkles size={15} />
		{#if !collapsed}TFL Selection{/if}
	</button>

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
				<button
					type="button"
					class="create-btn"
					onclick={onJoinServer}
					title={t('sidebar.joinServer')}
					aria-label={t('sidebar.joinServer')}
				>
					<Globe size={14} strokeWidth={2.25} />
				</button>
				<button
					type="button"
					class="create-btn"
					onclick={onCreate}
					title={t('sidebar.createInstance')}
					aria-label={t('sidebar.createInstance')}
				>
					<Plus size={14} strokeWidth={2.25} />
				</button>
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
					<button
						type="button"
						class="empty-create"
						onclick={onCreate}
						title={collapsed ? t('sidebar.createFirst') : undefined}
					>
						<Plus size={13} strokeWidth={2.5} />
						{#if !collapsed}{t('sidebar.createFirst')}{/if}
					</button>
				</div>
			{:else if filtered.length === 0}
				{#if !collapsed}<p class="empty-search">{t('sidebar.noResultsFor', { query })}</p>{/if}
			{:else}
				{#each filtered as instance (instance.uuid)}
					<button
						type="button"
						class="instance-item"
						class:active={selected?.uuid === instance.uuid}
						style="--loader-color: {LOADER_COLOR[instance.loader]}"
						title={collapsed ? instance.name : undefined}
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
					<button
						type="button"
						class="logout-btn"
						onclick={onOpenSkinManager}
						title={t('skinManager.title')}
						aria-label={t('skinManager.title')}
					>
						<Shirt size={14} />
					</button>
				{/if}
				<button
					type="button"
					class="logout-btn"
					onclick={onOpenSettings}
					title={t('sidebar.openSettings')}
					aria-label={t('sidebar.openSettings')}
				>
					<Settings size={14} />
				</button>
				<button
					type="button"
					class="logout-btn"
					onclick={onLogout}
					title={t('sidebar.logout')}
					aria-label={t('sidebar.logout')}
				>
					<LogOut size={14} />
				</button>
			</div>
		</div>
	{/if}
</aside>

<style>
	.sidebar {
		position: relative;
		z-index: 1;
		width: var(--sidebar-width);
		flex-shrink: 0;
		background: var(--bg-sidebar);
		border-right: 1px solid var(--border);
		display: flex;
		flex-direction: column;
		padding: 14px 12px;
		gap: 12px;
		transition: width 0.16s ease;
	}

	/* Modo compacto: solo íconos, pedido por el cliente para no perder
	   espacio de pantalla con una lista larga de instancias. El ancho fijo
	   acá (no una var global) porque solo afecta a este componente —
	   --sidebar-width la sigue usando TitleBar.svelte para alinear su
	   propio layout con el ancho normal. */
	.sidebar.collapsed {
		width: 76px;
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

	.home-btn,
	.tfl-selection-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid color-mix(in srgb, var(--accent) 35%, var(--border));
		background: color-mix(in srgb, var(--accent) 12%, var(--bg-card));
		color: var(--accent);
		font-size: 0.8rem;
		font-weight: 700;
		cursor: pointer;
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.home-btn {
		border-color: var(--border);
		background: var(--bg-card);
		color: var(--text-secondary);
	}

	.home-btn.active {
		border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
		background: color-mix(in srgb, var(--accent) 14%, var(--bg-card));
		color: var(--accent);
	}

	.home-btn:hover,
	.tfl-selection-btn:hover {
		background: color-mix(in srgb, var(--accent) 20%, var(--bg-card));
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

	.create-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: var(--accent);
		color: var(--accent-text);
		cursor: pointer;
	}

	.search-box {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
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
		font-size: 0.78rem;
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
		gap: 6px;
		margin-top: 4px;
		padding: 6px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.72rem;
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
		background: var(--bg-card);
		color: var(--text-primary);
		cursor: pointer;
		text-align: left;
		/* La barrita de color (::before, ver abajo) es un rectángulo recto
		   asomando desde el borde izquierdo — sin esto sobresale de las
		   esquinas redondeadas de la card, se ve "cortada" en vez de
		   completa. */
		overflow: hidden;
		transition:
			background 0.15s,
			border-color 0.15s,
			transform 0.12s;
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
		transition: opacity 0.15s;
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
		background: var(--bg-item-active);
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
		font-size: 0.83rem;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.instance-version {
		font-size: 0.68rem;
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
		background: var(--bg-card);
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

	.logout-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.logout-btn:hover {
		color: var(--color-error);
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

	.collapse-toggle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--border-radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}

	.collapse-toggle:hover {
		color: var(--text-primary);
	}
</style>
