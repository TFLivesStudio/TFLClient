<script lang="ts">
	import Tfl from '$lib/icons/Tfl.svelte';
	import type { InstanceData, MinecraftUser } from '$lib/types/types';
	import { Plus, User as UserIcon, LogOut, Settings, Search, Boxes } from 'lucide-svelte';

	let {
		instances,
		selected,
		user,
		onSelect,
		onCreate,
		onLogout,
		onOpenSettings
	}: {
		instances: InstanceData[];
		selected: InstanceData | null;
		user: MinecraftUser | null;
		onSelect: (i: InstanceData) => void;
		onCreate: () => void;
		onLogout: () => void;
		onOpenSettings: () => void;
	} = $props();

	const LOADER_COLOR: Record<string, string> = {
		vanilla: 'var(--loader-vanilla)',
		fabric: 'var(--loader-fabric)',
		forge: 'var(--loader-forge)',
		neoforge: 'var(--loader-neoforge)',
		quilt: 'var(--loader-quilt)'
	};

	let query = $state('');

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
						onclick={() => onSelect(instance)}
					>
						<span class="instance-avatar" style="--loader-color: {LOADER_COLOR[instance.loader]}">
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
			<UserIcon size={16} />
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
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		border-radius: var(--border-radius);
		border: 1px solid transparent;
		background: var(--bg-card);
		color: var(--text-primary);
		cursor: pointer;
		text-align: left;
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.instance-item:hover {
		border-color: var(--border);
	}

	.instance-item.active {
		background: var(--bg-item-active);
		border-color: var(--accent);
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
		gap: 8px;
		padding: 8px 10px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: var(--bg-card);
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
