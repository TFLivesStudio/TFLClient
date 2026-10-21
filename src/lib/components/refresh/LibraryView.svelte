<script lang="ts">
	import {
		Search,
		Plus,
		List,
		Grid2X2,
		Star,
		MoreHorizontal,
		Folder,
		X,
		ChevronRight
	} from 'lucide-svelte';
	import { appState } from '$lib/state/state.svelte';
	import { selectInstance } from '$lib/state/instanceState.svelte';
	import { openModal, openInstanceContextMenu } from '$lib/state/uiState.svelte';
	import { gameSession } from '$lib/state/gameSession.svelte';
	import { serverSessions } from '$lib/state/serverSessions.svelte';
	import {
		visual,
		instanceVisual,
		setInstanceVisual,
		setVisual,
		addGroup,
		removeGroup
	} from '$lib/visual/preferences.svelte';
	import { vr } from '$lib/i18n/refresh';
	import { i18nState } from '$lib/i18n/index.svelte';
	import InstanceMark from './InstanceMark.svelte';
	import LaunchControl from './LaunchControl.svelte';
	import type { InstanceData } from '$lib/types/types';
	let query = $state(''),
		filter = $state('all'),
		groupName = $state(''),
		creating = $state(false);
	const filtered = $derived(
		[...appState.instances]
			.filter((i) => {
				const meta = instanceVisual(i.uuid);
				return (
					`${i.name} ${i.mc_version} ${i.loader}`
						.toLowerCase()
						.includes(query.toLowerCase().trim()) &&
					(filter === 'all' ||
						(filter === 'recent' && i.last_played > 0) ||
						(filter === 'favorites' && meta.favorite) ||
						(filter.startsWith('group:') && meta.group === filter.slice(6)))
				);
			})
			.sort((a, b) => b.last_played - a.last_played || a.name.localeCompare(b.name))
	);
	function running(i: InstanceData) {
		return i.server_type
			? serverSessions.running.has(i.name)
			: gameSession.running.some((r) => r.name === i.name);
	}
	function createGroup() {
		addGroup(groupName);
		groupName = '';
		creating = false;
	}
</script>

<section class="vr-page library-view">
	<header class="library-heading">
		<div>
			<span class="vr-eyebrow">TFL / {vr('library')}</span>
			<h1 class="vr-heading">{vr('yourWorlds')}</h1>
			<p class="vr-description">{vr('librarySub')}</p>
		</div>
		<button class="vr-control primary" onclick={() => openModal('createChooser')}
			><Plus size={18} />{vr('create')}</button
		>
	</header>
	<div class="vr-toolbar">
		<label class="library-search"
			><Search size={18} /><input
				aria-label={vr('search')}
				placeholder={vr('search')}
				bind:value={query}
			/></label
		>
		<div class="view-switch">
			<button
				class="vr-control"
				aria-label={vr('list')}
				aria-pressed={visual.libraryView === 'list'}
				onclick={() => setVisual('libraryView', 'list')}><List size={18} /></button
			><button
				class="vr-control"
				aria-label={vr('grid')}
				aria-pressed={visual.libraryView === 'grid'}
				onclick={() => setVisual('libraryView', 'grid')}><Grid2X2 size={18} /></button
			>
		</div>
	</div>
	<nav class="library-filters" aria-label={vr('groups')}>
		{#each ['all', 'recent', 'favorites'] as key}<button
				class="vr-control"
				aria-pressed={filter === key}
				onclick={() => (filter = key)}>{vr(key as 'all' | 'recent' | 'favorites')}</button
			>{/each}
		{#each visual.groups as group}<div class="group-filter">
				<button
					class="vr-control"
					aria-pressed={filter === `group:${group}`}
					onclick={() => (filter = `group:${group}`)}><Folder size={14} />{group}</button
				><button
					class="delete-group"
					aria-label={`${vr('collectionDelete')}: ${group}`}
					onclick={() => {
						removeGroup(group);
						if (filter === `group:${group}`) filter = 'all';
					}}><X size={12} /></button
				>
			</div>{/each}
		<button class="vr-control" aria-expanded={creating} onclick={() => (creating = !creating)}
			><Plus size={15} />{vr('newGroup')}</button
		>
	</nav>
	{#if creating}<form
			class="group-form"
			onsubmit={(e) => {
				e.preventDefault();
				createGroup();
			}}
		>
			<input
				aria-label={vr('groupName')}
				placeholder={vr('groupName')}
				maxlength="40"
				bind:value={groupName}
			/><button class="vr-control" disabled={!groupName.trim()}>{vr('add')}</button>
		</form>{/if}
	<div class="library-count vr-muted">
		{filtered.length} / {appState.instances.length} · {vr('library')}
	</div>
	<div class="library-items vr-stagger" class:grid={visual.libraryView === 'grid'}>
		{#each filtered as instance (instance.uuid)}
			{@const meta = instanceVisual(instance.uuid)}
			<article class="vr-library-row" class:running={running(instance)}>
				<button class="instance-open" onclick={() => selectInstance(instance)}
					><InstanceMark {instance} /><span class="instance-copy"
						><strong>{instance.name}</strong><span
							>{instance.mc_version}
							<span class="loader">{instance.server_type ?? instance.loader}</span></span
						></span
					></button
				>
				<div class="instance-session">
					<span class="status" class:online={running(instance)}
						>{running(instance)
							? vr('running')
							: instance.server_type
								? vr('stopped')
								: vr('ready')}</span
					><small
						>{instance.last_played
							? new Date(instance.last_played * 1000).toLocaleDateString(i18nState.locale)
							: vr('never')}</small
					>
				</div>
				<select
					aria-label={`${vr('group')}: ${instance.name}`}
					value={meta.group}
					onchange={(e) => setInstanceVisual(instance.uuid, { group: e.currentTarget.value })}
					><option value="">{vr('ungrouped')}</option>{#each visual.groups as group}<option
							value={group}>{group}</option
						>{/each}</select
				>
				<div class="row-actions">
					<button
						class="favorite"
						aria-label={`${vr('favorite')}: ${instance.name}`}
						aria-pressed={meta.favorite}
						onclick={() => setInstanceVisual(instance.uuid, { favorite: !meta.favorite })}
						><Star size={17} fill={meta.favorite ? 'currentColor' : 'none'} /></button
					>
					{#if instance.server_type}<button
							class="vr-control"
							onclick={() => selectInstance(instance)}
							><ChevronRight size={16} />{vr('open')}</button
						>{:else}<LaunchControl {instance} />{/if}
					<button
						class="favorite"
						aria-label={`${vr('more')}: ${instance.name}`}
						onclick={(e) => {
							e.stopPropagation();
							const r = e.currentTarget.getBoundingClientRect();
							openInstanceContextMenu(instance, r.right, r.bottom);
						}}><MoreHorizontal size={18} /></button
					>
				</div>
			</article>
		{:else}<div class="library-empty">
				<Folder size={40} />
				<h2>{vr('empty')}</h2>
				<p class="vr-muted">{vr('emptyHint')}</p>
				<button class="vr-control" onclick={() => openModal('createChooser')}
					><Plus size={16} />{vr('create')}</button
				>
			</div>{/each}
	</div>
</section>

<style>
	.library-heading {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 24px;
	}
	.library-heading > .vr-control {
		flex-shrink: 0;
	}
	.library-search {
		display: flex;
		align-items: center;
		gap: 12px;
		border: 1px solid var(--border);
		background: var(--surface-card);
		border-radius: var(--border-radius);
		padding: 0 16px;
		flex: 1;
		color: var(--text-muted);
	}
	.library-search input {
		width: 100%;
		min-width: 0;
		background: none;
		border: 0;
		padding: 14px 0;
		color: var(--text-primary);
		outline-offset: 0;
	}
	.view-switch,
	.library-filters {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}
	.group-filter {
		display: flex;
		position: relative;
	}
	.group-filter .vr-control {
		padding-right: 28px;
	}
	.delete-group {
		position: absolute;
		right: 5px;
		top: 10px;
		background: none;
		color: var(--text-muted);
		border: 0;
		padding: 4px;
	}
	.group-form {
		display: flex;
		gap: 10px;
		margin: 12px 0;
	}
	.group-form input {
		background: var(--surface-card);
		color: var(--text-primary);
		border: 1px solid var(--border);
		padding: 10px;
		border-radius: var(--border-radius-sm);
	}
	.library-count {
		margin: 22px 0 12px;
	}
	.library-items {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.vr-library-row {
		display: flex;
		align-items: center;
		gap: 18px;
		min-height: var(--vr-row);
		padding: 14px 18px;
		background: var(--surface-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		transition:
			border-color var(--duration-micro),
			background-color var(--duration-micro);
	}
	.vr-library-row:hover {
		background: color-mix(in srgb, var(--accent) 4%, var(--surface-card));
		border-color: color-mix(in srgb, var(--accent) 30%, var(--border));
	}
	.instance-open {
		display: flex;
		align-items: center;
		gap: 16px;
		flex: 1;
		min-width: 0;
		text-align: left;
		color: var(--text-primary);
		border: 0;
		background: none;
		padding: 0;
	}
	.instance-copy {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-width: 0;
	}
	.instance-copy strong {
		font-size: 1.05rem;
		overflow-wrap: anywhere;
	}
	.instance-copy > span {
		font-size: var(--text-small);
		color: var(--text-secondary);
	}
	.loader {
		padding: 2px 6px;
		margin-left: 6px;
		background: var(--surface-interactive);
		border-radius: 4px;
		text-transform: capitalize;
	}
	.instance-session {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-width: 120px;
	}
	.instance-session small {
		color: var(--text-muted);
		font-size: var(--text-caption);
	}
	.status {
		font-size: var(--text-small);
		color: var(--text-secondary);
	}
	.status.online {
		color: var(--color-success);
	}
	.status:before {
		content: '';
		display: inline-block;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
		margin-right: 7px;
	}
	.row-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.favorite {
		width: 32px;
		height: 36px;
		display: grid;
		place-items: center;
		background: none;
		border: 0;
		color: var(--text-muted);
		border-radius: var(--border-radius-sm);
	}
	.favorite[aria-pressed='true'] {
		color: var(--accent);
	}
	select {
		max-width: 120px;
		min-width: 0;
		background: var(--surface-interactive);
		color: var(--text-secondary);
		border: 1px solid var(--border);
		padding: 8px;
		border-radius: var(--border-radius-sm);
		font-size: var(--text-caption);
	}
	.library-items.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(270px, 1fr));
		gap: 18px;
	}
	.grid .vr-library-row {
		flex-wrap: wrap;
		padding: 22px;
	}
	.grid .instance-open {
		flex-basis: 100%;
	}
	.grid .instance-session {
		flex: 1;
	}
	.grid .row-actions {
		width: 100%;
		justify-content: flex-end;
		border-top: 1px solid var(--border);
		padding-top: 14px;
	}
	.library-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 14px;
		padding: 60px 20px;
		color: var(--text-muted);
	}
	@media (max-width: 1200px) {
		.instance-session {
			display: none;
		}
		.vr-library-row {
			gap: 10px;
			padding: 12px;
		}
		.library-heading {
			align-items: flex-start;
			flex-direction: column;
			gap: 10px;
		}
	}
	@media (max-width: 960px) {
		.vr-library-row > select {
			display: none;
		}
		.grid .vr-library-row > select {
			display: block;
		}
		.instance-copy strong {
			font-size: 1rem;
		}
	}
</style>
