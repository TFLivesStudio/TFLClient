<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import Tfl from '$lib/icons/Tfl.svelte';
	import Mascot from '$lib/components/ui/Mascot.svelte';
	import { getMascotFor } from '$lib/mascots';
	import { network } from '$lib/state/network.svelte';
	import type { InstanceData, MinecraftUser } from '$lib/types/types';
	import { Plus, User as UserIcon, LogOut, Settings, Search, Boxes, Sparkles, WifiOff } from 'lucide-svelte';

	let {
		instances,
		selected,
		user,
		onSelect,
		onCreate,
		onLogout,
		onOpenSettings,
		onOpenTflSelection
	}: {
		instances: InstanceData[];
		selected: InstanceData | null;
		user: MinecraftUser | null;
		onSelect: (i: InstanceData) => void;
		onCreate: () => void;
		onLogout: () => void;
		onOpenSettings: () => void;
		onOpenTflSelection: () => void;
	} = $props();

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
</script>

<aside class="sidebar">
	<div class="sidebar-header">
		<div class="brand-mark">
			<Tfl width="18" height="18" />
			<h1>TFL Client</h1>
		</div>
	</div>

	<button type="button" class="tfl-selection-btn" onclick={onOpenTflSelection}>
		<Sparkles size={15} />
		TFL Selection
	</button>

	{#if !network.online}
		<div class="offline-badge">
			<WifiOff size={12} />
			<span>Sin conexión — jugá lo que ya tenés instalado</span>
		</div>
	{/if}

	<div class="instances">
		<div class="section-label">
			<span>Tus instancias</span>
			<button type="button" class="create-btn" onclick={onCreate} aria-label="Crear instancia">
				<Plus size={14} strokeWidth={2.25} />
			</button>
		</div>

		{#if instances.length > 0}
			<div class="search-box">
				<Search size={13} />
				<input type="text" placeholder="Buscar…" bind:value={query} />
				<kbd class="shortcut-hint">{shortcutLabel}</kbd>
			</div>
		{/if}

		<div class="instance-list">
			{#if instances.length === 0}
				<div class="empty">
					<Boxes size={28} />
					<p>Todavía no creaste ninguna instancia</p>
					<button type="button" class="empty-create" onclick={onCreate}>
						<Plus size={13} strokeWidth={2.5} /> Crear la primera
					</button>
				</div>
			{:else if filtered.length === 0}
				<p class="empty-search">Sin resultados para "{query}"</p>
			{:else}
				{#each filtered as instance (instance.uuid)}
					<button
						type="button"
						class="instance-item"
						class:active={selected?.uuid === instance.uuid}
						style="--loader-color: {LOADER_COLOR[instance.loader]}"
						onclick={() => onSelect(instance)}
					>
						<span class="instance-avatar">
							{instance.name.charAt(0).toUpperCase()}
						</span>
						<span class="instance-text">
							<span class="instance-name">{instance.name}</span>
							<span class="instance-version">{instance.mc_version} · {instance.loader}</span>
						</span>
					</button>
				{/each}
			{/if}
		</div>
	</div>

	{#if user}
		<div class="user-chip">
			<span class="user-head">
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
			<div class="user-info">
				<span class="user-name">{user.username}</span>
				<span class="user-type"
					>{user.user_type === 'Cracked' ? 'Offline — solo singleplayer' : user.user_type}</span
				>
			</div>
			<button type="button" class="logout-btn" onclick={onOpenSettings} aria-label="Ajustes">
				<Settings size={14} />
			</button>
			<button type="button" class="logout-btn" onclick={onLogout} aria-label="Cerrar sesión">
				<LogOut size={14} />
			</button>
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
	}

	.sidebar-header {
		padding-bottom: 10px;
		border-bottom: 1px solid var(--border);
	}

	.tfl-selection-btn {
		display: flex;
		align-items: center;
		gap: 8px;
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

	.tfl-selection-btn:hover {
		background: color-mix(in srgb, var(--accent) 20%, var(--bg-card));
		border-color: var(--accent);
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

	.brand-mark {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.brand-mark :global(svg) {
		color: var(--accent);
	}

	.sidebar-header h1 {
		font-family: var(--font-brand);
		font-size: 0.78rem;
		color: var(--text-primary);
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
		transform: translateX(1px);
	}

	.instance-item.active {
		background: var(--bg-item-active);
		border-color: color-mix(in srgb, var(--loader-color) 40%, var(--border));
	}

	.instance-item.active::before {
		opacity: 1;
	}

	.instance-avatar {
		flex-shrink: 0;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 0.8rem;
		font-weight: 800;
		color: var(--loader-color);
		background: color-mix(in srgb, var(--loader-color) 18%, transparent);
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
</style>
