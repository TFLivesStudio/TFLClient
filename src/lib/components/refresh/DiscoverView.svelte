<script lang="ts">
	import { Compass, ArrowUpRight, Sparkles, Box } from 'lucide-svelte';
	import { vr } from '$lib/i18n/refresh';
	import { appState } from '$lib/state/state.svelte';
	import { openModal, openContentExplorer } from '$lib/state/uiState.svelte';
	import InstanceMark from './InstanceMark.svelte';
	const clients = $derived(appState.instances.filter((i) => !i.server_type));
</script>

<section class="vr-page discover-view vr-stagger">
	<header>
		<span class="vr-eyebrow"><Compass size={16} />{vr('discover')}</span>
		<h1 class="vr-heading">{vr('exploreTitle')}</h1>
		<p class="vr-description">{vr('exploreSub')}</p>
	</header>
	<article class="vr-editorial">
		<div>
			<span class="vr-eyebrow"><Sparkles size={16} /> TFLIVES / {vr('curated')}</span>
			<h2>{vr('selection')}</h2>
			<p>{vr('selectionSub')}</p>
			<button class="vr-control primary" onclick={() => openModal('tflSelection')}
				>{vr('selectionAction')}<ArrowUpRight size={18} /></button
			>
		</div>
		<div class="editorial-mark" aria-hidden="true"><img src="/images/tfl.svg" alt="" /></div>
	</article>
	<div class="discover-context">
		<h2>{vr('contextual')}</h2>
		<p class="vr-muted">{vr('contextualHint')}</p>
		<div class="context-list">
			{#each clients as instance (instance.uuid)}<button
					onclick={() =>
						openContentExplorer({
							instance,
							kind: instance.loader === 'vanilla' ? 'resourcepack' : 'mod'
						})}
					><InstanceMark {instance} /><span
						><strong>{instance.name}</strong><small>{instance.mc_version} · {instance.loader}</small
						></span
					><ArrowUpRight size={18} /></button
				>{:else}<div class="vr-muted">
					<Box size={22} />
					<p>{vr('noExplore')}</p>
				</div>{/each}
		</div>
	</div>
</section>

<style>
	.vr-editorial {
		display: flex;
		position: relative;
		align-items: center;
		justify-content: space-between;
		gap: 24px;
		margin: 32px 0 40px;
		padding: clamp(28px, 4vw, 52px);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		background:
			radial-gradient(
				ellipse at 95% 15%,
				color-mix(in srgb, var(--accent) 22%, transparent),
				transparent 60%
			),
			var(--surface-card);
		overflow: hidden;
	}
	.vr-editorial h2 {
		font-size: clamp(2rem, 4vw, 3.7rem);
		letter-spacing: -0.06em;
		margin: 24px 0 16px;
	}
	.vr-editorial p {
		color: var(--text-secondary);
		max-width: 46ch;
		line-height: 1.7;
		margin: 0 0 28px;
	}
	.editorial-mark {
		width: 180px;
		flex-shrink: 0;
		opacity: 0.65;
		transform: rotate(-12deg);
	}
	.editorial-mark img {
		width: 100%;
	}
	.discover-context h2 {
		font-size: var(--text-subtitle);
		margin-bottom: 10px;
	}
	.context-list {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
		gap: 10px;
		margin-top: 20px;
	}
	.context-list button {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 18px;
		background: var(--surface-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		color: var(--text-primary);
		text-align: left;
	}
	.context-list button > span {
		display: flex;
		flex: 1;
		min-width: 0;
		flex-direction: column;
		gap: 8px;
	}
	.context-list small {
		color: var(--text-muted);
	}
	@media (max-width: 960px) {
		.editorial-mark {
			width: 100px;
		}
		.vr-editorial {
			padding: 24px;
			gap: 12px;
		}
	}
</style>
