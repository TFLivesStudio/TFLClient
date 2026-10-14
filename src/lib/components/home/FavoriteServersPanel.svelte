<script lang="ts">
	import { Surface, EmptyState } from '$lib/components/ui';
	import { t } from '$lib/i18n/index.svelte';
	import { Server as ServerIcon } from 'lucide-svelte';
	import type { FavoriteServer } from '$lib/types/types';

	/** Panel "Tus servidores" de la Home — Entrega C. Markup/estilos
	 * extraídos de WelcomeState.svelte (líneas 401-421 originales); los
	 * datos (`getFavoriteServers()`) siguen cargándose en el orquestador. */
	let {
		servers,
		onSeeAll
	}: {
		servers: FavoriteServer[];
		onSeeAll: () => void;
	} = $props();
</script>

<Surface level="card" as="section">
	<div class="panel">
		<div class="panel-title">
			<ServerIcon size={13} />
			<span>{t('home.yourServers')}</span>
			<button type="button" class="panel-link" onclick={onSeeAll}>
				{t('home.seeAll')}
			</button>
		</div>
		{#if servers.length === 0}
			<EmptyState title={t('home.noServersSaved')} description={t('home.noServersSavedHint')}>
				{#snippet icon()}<ServerIcon size={22} />{/snippet}
			</EmptyState>
		{:else}
			<ul class="panel-list">
				{#each servers as server (server.id)}
					<li class="server-row">
						<span class="server-name">{server.name}</span>
						<span class="server-address">{server.address}</span>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</Surface>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px;
		min-width: 0;
	}

	.panel-title {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		font-weight: 700;
		letter-spacing: 0.4px;
		text-transform: uppercase;
		color: var(--text-muted);
	}
	.panel-title :global(svg) {
		color: var(--accent);
		flex-shrink: 0;
	}
	.panel-link {
		margin-left: auto;
		background: none;
		border: none;
		padding: 0;
		color: var(--accent);
		font-size: var(--text-caption);
		font-weight: 700;
		text-transform: none;
		letter-spacing: normal;
		cursor: pointer;
	}
	.panel-link:hover {
		text-decoration: underline;
	}

	.panel-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.server-row {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.server-name {
		font-size: var(--text-small);
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.server-address {
		font-size: var(--text-caption);
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
</style>
