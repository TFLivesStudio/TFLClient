<script lang="ts">
	import { navigate, navigation } from '$lib/visual/navigation.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { tr } from '$lib/i18n/experience';
	import {
		instanceDisplayName,
		instanceEdition,
		isSelectionInstance,
		selectionFallbackIcon
	} from '$lib/selection-identity';
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
		LogOut,
		Settings,
		Search,
		Boxes,
		WifiOff,
		Globe,
		Compass,
		Shirt,
		Home,
		ChevronLeft,
		ChevronRight
	} from 'lucide-svelte';

	let {
		instances,
		selected,
		user,
		onSelect,
		onCreate,
		onLogout,
		onOpenSettings,
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
		if (!appState.settings || forcedCompact) return;
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
	const sidebarId = $props.id();
	const collapsed = $derived(isSidebarEffectiveCollapsed());
	const forcedCompact = $derived(isSidebarForcedCompact());

	const LOADER_COLOR: Record<string, string> = {
		vanilla: 'var(--loader-vanilla)',
		fabric: 'var(--loader-fabric)',
		forge: 'var(--loader-forge)',
		neoforge: 'var(--loader-neoforge)',
		quilt: 'var(--loader-quilt)',
		'ornithe-gen2': 'var(--loader-ornithe-gen2)'
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
		const account = user;
		let cancelled = false;
		skinUrl = null;
		if (account && account.user_type !== 'Cracked')
			invoke<string | null>('get_skin_texture_url', { uuid: account.uuid })
				.then((url) => {
					if (!cancelled)
						skinUrl =
							url?.replace('http://textures.minecraft.net/', 'https://textures.minecraft.net/') ??
							null;
				})
				.catch(() => {});
		return () => {
			cancelled = true;
		};
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

<div
	class="sidebar-shell"
	class:collapsed
	style:width={(collapsed ? SIDEBAR_COLLAPSED_WIDTH : SIDEBAR_EXPANDED_WIDTH) + 'px'}
>
	<aside
		id={sidebarId}
		class="sidebar"
		class:collapsed
		aria-label={tr('Navegación principal', 'Main navigation')}
	>
		<nav class="primary-nav" aria-label={tr('Espacios de trabajo', 'Workspaces')}>
			{#each ['home', 'library', 'discover'] as section (section)}
				<Tooltip
					content={vr(section as 'home' | 'library' | 'discover')}
					position="right"
					disabled={!collapsed}
				>
					<button
						type="button"
						class="home-btn"
						class:active={selected === null && navigation.section === section}
						aria-current={selected === null && navigation.section === section ? 'page' : undefined}
						onclick={() =>
							section === 'home' ? goHome() : navigate(section as 'library' | 'discover')}
						aria-label={vr(section as 'home' | 'library' | 'discover')}
					>
						{#if section === 'home'}<Home size={18} />{:else if section === 'library'}<Boxes
								size={18}
							/>{:else}<Compass size={18} />{/if}
						{#if !collapsed}<span>{vr(section as 'home' | 'library' | 'discover')}</span>{/if}
					</button>
				</Tooltip>
			{/each}
		</nav>
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
				<div class="search-box" data-search-control>
					<Search size={13} />
					<input
						type="text"
						aria-label={t('sidebar.search')}
						placeholder={t('sidebar.search')}
						bind:value={query}
					/>
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
						<Tooltip
							content={`${instanceDisplayName(instance)} · ${instanceEdition(instance)}`}
							position="right"
							disabled={!collapsed}
						>
							<button
								type="button"
								class="instance-item"
								class:active={selected?.uuid === instance.uuid}
								style="--loader-color: {isSelectionInstance(instance)
									? 'var(--accent)'
									: (LOADER_COLOR[instance.loader] ?? 'var(--text-secondary)')}"
								aria-label={instanceDisplayName(instance)}
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
									{#if iconUrls[instance.uuid] || selectionFallbackIcon(instance.selection)}
										<img
											src={iconUrls[instance.uuid] ??
												selectionFallbackIcon(instance.selection) ??
												undefined}
											alt=""
										/>
									{:else}
										{instanceDisplayName(instance).charAt(0).toUpperCase()}
									{/if}
								</span>
								{#if !collapsed}
									<span class="instance-text">
										<span class="instance-name">{instanceDisplayName(instance)}</span>
										<span class="instance-version"
											>{instance.mc_version} · {instanceEdition(instance)}</span
										>
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
				<button
					type="button"
					class="user-profile"
					title={user.username}
					aria-label={`${user.username} · ${t('settings.tabs.accounts')}`}
					onclick={user.user_type === 'Cracked' ? onOpenSettings : onOpenSkinManager}
				>
					<span class="user-head">
						{#if user.user_type === 'Cracked'}
							<!-- Cuentas offline no tienen skin real — mascota simple en
						 vez del Steve genérico que devolvería Crafatar por
						 defecto. Elegida al azar (determinística por UUID) al
						 crear la cuenta, cambiable desde Ajustes. -->
							<Mascot id={getMascotFor(user.uuid)} size={32} />
						{:else}
							<img class="face-fallback" src="/images/player-face.svg" alt="" />
							{#if skinUrl}
								<span class="skin-head">
									<span class="skin-layer base" style="background-image: url({skinUrl})"></span>
									<span class="skin-layer overlay" style="background-image: url({skinUrl})"></span>
								</span>
							{/if}
						{/if}
					</span>
					{#if !collapsed}
						<span class="user-info">
							<span class="user-name">{user.username}</span>
							<span class="user-type"
								>{user.user_type === 'Cracked'
									? t('sidebar.offlineAccountType')
									: user.user_type}</span
							>
						</span>
					{/if}
				</button>
				<div class="user-actions">
					{#if user.user_type !== 'Cracked' && !collapsed}
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
	<Tooltip
		content={forcedCompact
			? tr(
					'Amplía la ventana para expandir la barra lateral',
					'Widen the window to expand the sidebar'
				)
			: collapsed
				? t('sidebar.expand')
				: t('sidebar.collapse')}
		position="right"
	>
		<button
			type="button"
			class="sidebar-edge-toggle"
			aria-controls={sidebarId}
			aria-expanded={!collapsed}
			aria-disabled={forcedCompact}
			aria-label={collapsed ? t('sidebar.expand') : t('sidebar.collapse')}
			onclick={toggleCollapsed}
		>
			{#if collapsed}<ChevronRight size={14} />{:else}<ChevronLeft size={14} />{/if}
		</button>
	</Tooltip>
</div>

<style>
	/* One coordinate owner for expanded/compact rails; scrolling stays inside. */
	.sidebar-shell {
		position: relative;
		z-index: calc(var(--z-content) + 1);
		flex-shrink: 0;
		height: 100%;
		min-height: 0;
		transition: width var(--duration-panel) var(--easing-standard);
	}
	.sidebar {
		width: 100%;
		height: 100%;
		min-height: 0;
		box-sizing: border-box;
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 16px 12px 12px;
		overflow: hidden;
		border-right: 1px solid var(--border);
		background: var(--surface-workspace);
	}
	.primary-nav {
		display: grid;
		gap: 4px;
		flex-shrink: 0;
	}
	.sidebar :global(.tfl-tooltip-wrap) {
		display: flex;
		width: 100%;
		min-width: 0;
	}
	.sidebar :global(.tfl-tooltip-wrap > button) {
		flex: 1;
		min-width: 0;
	}
	.home-btn {
		position: relative;
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		min-height: 40px;
		padding: 8px 12px;
		background: transparent;
		border: 1px solid transparent;
		border-radius: var(--border-radius-sm);
		color: var(--text-secondary);
		font: inherit;
		font-size: var(--text-small);
		line-height: 1.5;
		text-align: left;
		cursor: pointer;
	}
	.home-btn:hover {
		background: var(--surface-interactive);
		color: var(--text-primary);
	}
	.home-btn.active {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
		color: var(--text-primary);
		border-color: color-mix(in srgb, var(--accent) 28%, transparent);
	}
	.home-btn.active::before {
		content: '';
		position: absolute;
		left: -1px;
		top: 12px;
		bottom: 12px;
		width: 2px;
		background: var(--accent);
	}
	.home-btn :global(svg) {
		flex-shrink: 0;
	}
	.offline-badge {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px;
		font-size: 12px;
		color: var(--color-warning);
		border: 1px solid var(--border);
	}
	.instances {
		min-height: 0;
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.section-label {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		flex-shrink: 0;
		color: var(--text-muted);
		font-size: 11px;
		line-height: 1.5;
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}
	.section-label-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.section-label-actions :global(.tfl-icon-btn) {
		width: 32px;
		height: 32px;
		min-height: 32px;
	}
	.search-box {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 36px;
		padding: 4px 8px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		background: var(--bg-input);
		color: var(--text-secondary);
		flex-shrink: 0;
	}
	.search-box input {
		flex: 1;
		width: 0;
		min-width: 0;
		border: 0;
		outline: 0;
		background: transparent;
		color: var(--text-primary);
		font-size: 12px;
		line-height: 1.5;
	}
	.search-box:focus-within {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}
	.shortcut-hint {
		display: none;
	}
	.instance-list {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		gap: 4px;
		overflow-y: auto;
		overflow-x: hidden;
		scrollbar-width: thin;
		scrollbar-color: var(--border) transparent;
		padding: 4px 0;
	}
	.instance-item {
		position: relative;
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		min-height: 56px;
		flex-shrink: 0;
		padding: 8px;
		border: 1px solid transparent;
		border-radius: var(--border-radius-sm);
		background: transparent;
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
	}
	.instance-item:hover {
		background: var(--surface-interactive);
	}
	.instance-item.active {
		border-color: color-mix(in srgb, var(--loader-color) 44%, var(--border));
		background: var(--surface-interactive);
	}
	.instance-item.active::after {
		content: '';
		position: absolute;
		inset: 12px auto 12px 0;
		width: 2px;
		background: var(--loader-color);
	}
	.instance-avatar {
		display: grid;
		place-items: center;
		width: 32px;
		height: 32px;
		flex-shrink: 0;
		overflow: hidden;
		border-radius: var(--border-radius-sm);
		font-size: 14px;
		font-weight: 700;
		color: var(--loader-color);
		background: color-mix(in srgb, var(--loader-color) 12%, transparent);
	}
	.instance-avatar img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.instance-text {
		display: grid;
		gap: 4px;
		min-width: 0;
	}
	.instance-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 13px;
		line-height: 1.5;
		font-weight: 600;
	}
	.instance-version {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--text-muted);
		font-size: 11px;
		line-height: 1.5;
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 16px 0;
		color: var(--text-secondary);
		font-size: 12px;
		text-align: center;
	}
	.empty-create {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		min-height: 36px;
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
		padding: 8px;
		background: var(--surface-interactive);
		color: var(--text-primary);
	}
	.empty-search {
		font-size: 12px;
		color: var(--text-secondary);
		overflow-wrap: anywhere;
	}
	.user-chip {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: auto;
		flex-shrink: 0;
		padding: 12px 8px 0;
		border: 0;
		border-top: 1px solid var(--border);
		background: transparent;
	}
	.user-profile {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		min-width: 0;
		min-height: 36px;
		padding: 0;
		border: 0;
		background: transparent;
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
	}
	.user-head {
		position: relative;
		display: grid;
		place-items: center;
		width: 32px;
		height: 32px;
		flex-shrink: 0;
		overflow: hidden;
		border-radius: var(--border-radius-sm);
	}
	.face-fallback,
	.skin-head {
		position: absolute;
		inset: 0;
		width: 32px;
		height: 32px;
		image-rendering: pixelated;
	}
	.skin-layer {
		position: absolute;
		inset: 0;
		background-repeat: no-repeat;
		background-size: 256px auto;
		image-rendering: pixelated;
	}
	.skin-layer.base {
		background-position: -32px -32px;
	}
	.skin-layer.overlay {
		background-position: -160px -32px;
	}
	.user-info {
		display: grid;
		gap: 0;
		min-width: 0;
		flex: 1;
	}
	.user-name {
		font-size: 13px;
		line-height: 1.5;
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.user-type {
		font-size: 11px;
		line-height: 1.5;
		color: var(--text-secondary);
	}
	.user-actions {
		display: flex;
		gap: 8px;
		justify-content: space-around;
	}
	.user-actions :global(.tfl-icon-btn) {
		width: 36px;
		height: 36px;
		min-height: 36px;
	}
	/* Compact is a deliberately spaced rail, not expanded rows with hidden text. */
	.sidebar.collapsed {
		align-items: center;
		padding: 12px;
		gap: 12px;
	}
	.collapsed .primary-nav,
	.collapsed .instances,
	.collapsed .user-chip {
		width: 48px;
	}
	.collapsed .home-btn {
		width: 40px;
		height: 40px;
		min-height: 40px;
		padding: 0;
		justify-content: center;
	}
	.collapsed .primary-nav {
		justify-items: center;
	}
	.collapsed .primary-nav :global(.tfl-tooltip-wrap) {
		width: 40px;
	}
	.collapsed .instances {
		border-top: 1px solid var(--border);
		padding-top: 8px;
	}
	.collapsed .section-label {
		justify-content: center;
	}
	.collapsed .section-label-actions {
		flex-direction: column;
		gap: 4px;
	}
	.collapsed .section-label-actions :global(.tfl-icon-btn) {
		width: 36px;
		height: 32px;
		min-height: 32px;
	}
	.collapsed .instance-list {
		align-items: center;
		gap: 4px;
		padding-inline: 2px;
	}
	.collapsed .instance-item {
		width: 40px;
		height: 40px;
		min-height: 40px;
		padding: 4px;
		justify-content: center;
	}
	.collapsed .instance-list :global(.tfl-tooltip-wrap) {
		width: 40px;
		flex-shrink: 0;
	}
	.collapsed .instance-avatar {
		width: 28px;
		height: 28px;
	}
	.collapsed .user-chip {
		padding: 8px 4px 0;
		align-items: center;
		gap: 4px;
	}
	.collapsed .user-profile {
		width: 40px;
		min-height: 40px;
		justify-content: center;
	}
	.collapsed .user-actions {
		flex-direction: column;
		gap: 4px;
	}
	.collapsed .empty {
		padding: 8px 0;
	}
	.collapsed .empty > :global(svg) {
		display: none;
	}
	/* The sibling edge tab deliberately escapes only the exterior wrapper. */
	.sidebar-shell > :global(.tfl-tooltip-wrap) {
		position: absolute;
		right: -16px;
		top: 50%;
		transform: translateY(-50%);
		width: 32px;
		height: 40px;
		z-index: 1;
	}
	.sidebar-edge-toggle {
		display: grid;
		place-items: center;
		width: 32px;
		height: 40px;
		padding: 0;
		border: 1px solid var(--border);
		border-radius: 6px;
		color: var(--text-secondary);
		background: var(--surface-workspace);
		box-shadow: 1px 0 3px #0002;
		cursor: pointer;
		transition:
			color var(--duration-micro),
			background-color var(--duration-micro);
	}
	.sidebar-edge-toggle:hover {
		color: var(--text-primary);
		background: var(--surface-interactive);
	}
	.sidebar-edge-toggle:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 3px;
	}
	.sidebar-edge-toggle[aria-disabled='true'] {
		color: var(--text-muted);
		cursor: default;
	}
	:global(html[data-visual='pixel']) .home-btn {
		border: 1px solid transparent;
		box-shadow: none;
		text-shadow: none;
		background: transparent;
	}
	:global(html[data-visual='pixel']) .home-btn:hover {
		border-color: var(--pixel-rim-light);
		background: var(--pixel-stone);
		box-shadow:
			inset 0 1px var(--pixel-rim-light),
			inset 0 -1px var(--pixel-rim-dark);
	}
	:global(html[data-visual='pixel']) .home-btn.active {
		border-color: color-mix(in srgb, var(--accent) 42%, var(--border));
		background: color-mix(in srgb, var(--accent) 14%, var(--pixel-slot));
		box-shadow: inset 0 1px #0003;
	}
	:global(html[data-visual='pixel']) .home-btn:active {
		box-shadow:
			inset 1px 2px #0005,
			inset -1px -1px var(--pixel-rim-light);
	}
	:global(html[data-visual='pixel']) .user-chip {
		border: 0;
		border-top: 1px solid var(--border);
		box-shadow: none;
	}
	:global(html[data-visual='pixel']) .sidebar-edge-toggle {
		border-radius: 0;
		box-shadow:
			inset 1px 1px var(--pixel-rim-light),
			inset -1px -1px var(--pixel-rim-dark);
	}
</style>
