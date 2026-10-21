<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import type { ModSearchHit, ContentProjectDetail, ModVersionSummary } from '$lib/types/types';
	import { openContentProjectLink } from '$lib/api/content';
	import { t } from '$lib/i18n/index.svelte';
	import {
		ArrowLeft,
		Download,
		Check,
		Loader2,
		ExternalLink,
		Github,
		BookOpen,
		MessageCircle,
		AlertCircle
	} from 'lucide-svelte';

	/** Ficha completa de un proyecto — pedido explícito del cliente:
	 * descripción completa en Markdown SEGURO (sin HTML sin sanear),
	 * categorías, versiones de MC, loaders, versiones publicadas,
	 * changelog, galería, enlaces oficiales (navegador externo — instalar
	 * sigue siendo acá adentro, nunca el navegador). Se pide a demanda, ya
	 * fetcheada por el composable — este componente solo renderiza. */
	let {
		kind,
		projectId,
		hit,
		detail,
		loading,
		error,
		installed,
		installing,
		onBack,
		onInstall,
		loadVersions
	}: {
		kind: 'mod' | 'shader' | 'resourcepack' | 'plugin' | 'modpack';
		projectId: string;
		hit: ModSearchHit | null;
		detail: ContentProjectDetail | null;
		loading: boolean;
		error: string | null;
		installed: boolean;
		installing: boolean;
		onBack: () => void;
		onInstall: (versionId?: string) => void;
		loadVersions: (projectId: string) => Promise<ModVersionSummary[]>;
	} = $props();

	// marked.parse puede devolver una Promise si hay extensiones async
	// configuradas — no es el caso acá (sin `use()` custom), pero se
	// tipea explícito para no depender de esa inferencia.
	const descriptionHtml = $derived(
		detail ? DOMPurify.sanitize(marked.parse(detail.body, { async: false }) as string) : ''
	);

	let versions = $state<ModVersionSummary[]>([]);
	let loadingVersions = $state(false);
	let versionsToken = 0;
	$effect(() => {
		if (kind === 'modpack') return;
		const token = ++versionsToken;
		versions = [];
		loadingVersions = true;
		loadVersions(projectId).then((v) => {
			if (token === versionsToken) {
				versions = v;
				loadingVersions = false;
			}
		});
	});

	function formatDate(iso: string): string {
		if (!iso) return '';
		try {
			return new Date(iso).toLocaleDateString();
		} catch {
			return iso;
		}
	}
</script>

<div class="project-detail">
	<button type="button" class="back-btn" onclick={onBack}>
		<ArrowLeft size={14} />
		{t('contentManager.backToResults')}
	</button>

	{#if loading}
		<div class="centered"><Loader2 size={20} class="spin" /></div>
	{:else if error}
		<div class="centered error">
			<AlertCircle size={16} />
			<span>{t('projectDetail.loadFailed')}</span>
		</div>
	{:else if detail}
		<div class="header">
			{#if detail.icon_url}
				<img class="icon" src={detail.icon_url} alt="" />
			{:else}
				<div class="icon icon-fallback"></div>
			{/if}
			<div class="header-info">
				<h2>{detail.title}</h2>
				{#if hit?.author}<span class="author">{hit.author}</span>{/if}
				<div class="stats">
					<span>{t('projectDetail.downloads', { count: detail.downloads })}</span>
					<span>{t('projectDetail.followers', { count: detail.followers })}</span>
				</div>
			</div>
			{#if installed}
				<span class="installed-badge" title={t('contentManager.alreadyInstalled')}
					><Check size={16} /></span
				>
			{:else}
				<button type="button" class="install-btn" disabled={installing} onclick={() => onInstall()}>
					{#if installing}<Loader2 size={16} class="spin" />{:else}<Download size={16} />{/if}
					{t('common.install')}
				</button>
			{/if}
		</div>

		{#if detail.categories.length > 0}
			<div class="chips">
				{#each detail.categories as cat (cat)}<span class="chip">{cat}</span>{/each}
			</div>
		{/if}

		<p class="dates">
			{t('projectDetail.published', { date: formatDate(detail.published) })} ·
			{t('projectDetail.updated', { date: formatDate(detail.updated) })}
		</p>

		{#if detail.gallery.length > 0}
			<div class="gallery">
				{#each detail.gallery as img (img.url)}
					<img src={img.url} alt={img.title ?? ''} loading="lazy" />
				{/each}
			</div>
		{/if}

		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="description">{@html descriptionHtml}</div>

		{#if kind !== 'modpack'}
			<section class="versions-section">
				<h3>{t('projectDetail.versions')}</h3>
				{#if loadingVersions}
					<Loader2 size={14} class="spin" />
				{:else if versions.length === 0}
					<p class="hint">{t('contentManager.noCompatibleVersions')}</p>
				{:else}
					<div class="versions-list">
						{#each versions as v (v.id)}
							<button
								type="button"
								class="version-row"
								disabled={installing}
								onclick={() => onInstall(v.id)}
							>
								<span>{v.version_number || v.name}</span>
								<span class="version-date">{formatDate(v.date_published)}</span>
							</button>
						{/each}
					</div>
				{/if}
			</section>
		{/if}

		{#if detail.source_url || detail.wiki_url || detail.discord_url || detail.issues_url}
			<section class="links-section">
				<h3>{t('projectDetail.links')}</h3>
				<div class="links">
					{#if detail.source_url}
						<button
							type="button"
							class="link-btn"
							onclick={() => openContentProjectLink(detail.source_url!)}
						>
							<Github size={13} />{t('projectDetail.sourceLink')}<ExternalLink size={11} />
						</button>
					{/if}
					{#if detail.wiki_url}
						<button
							type="button"
							class="link-btn"
							onclick={() => openContentProjectLink(detail.wiki_url!)}
						>
							<BookOpen size={13} />{t('projectDetail.wikiLink')}<ExternalLink size={11} />
						</button>
					{/if}
					{#if detail.discord_url}
						<button
							type="button"
							class="link-btn"
							onclick={() => openContentProjectLink(detail.discord_url!)}
						>
							<MessageCircle size={13} />{t('projectDetail.discordLink')}<ExternalLink size={11} />
						</button>
					{/if}
					{#if detail.issues_url}
						<button
							type="button"
							class="link-btn"
							onclick={() => openContentProjectLink(detail.issues_url!)}
						>
							<AlertCircle size={13} />{t('projectDetail.issuesLink')}<ExternalLink size={11} />
						</button>
					{/if}
					<button
						type="button"
						class="link-btn"
						onclick={() => openContentProjectLink(`https://modrinth.com/${kind}/${projectId}`)}
					>
						{t('projectDetail.openOnModrinth')}<ExternalLink size={11} />
					</button>
				</div>
			</section>
		{/if}
	{/if}
</div>

<style>
	.project-detail {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.back-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		align-self: flex-start;
		padding: 5px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.72rem;
		font-weight: 600;
		cursor: pointer;
	}

	.centered {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 30px 0;
	}

	.centered.error {
		color: var(--color-error);
	}

	.header {
		display: flex;
		align-items: flex-start;
		gap: 12px;
	}

	.icon,
	.icon-fallback {
		width: 56px;
		height: 56px;
		border-radius: var(--border-radius-sm);
		flex-shrink: 0;
		background: var(--bg-input);
		object-fit: cover;
	}

	.header-info {
		flex: 1;
		min-width: 0;
	}

	.header-info h2 {
		margin: 0;
		font-size: 1.05rem;
	}

	.author {
		font-size: 0.74rem;
		color: var(--text-muted);
	}

	.stats {
		display: flex;
		gap: 10px;
		margin-top: 4px;
		font-size: 0.72rem;
		color: var(--text-muted);
	}

	.install-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, var(--bg-input));
		color: var(--accent);
		font-weight: 700;
		font-size: 0.78rem;
		cursor: pointer;
		flex-shrink: 0;
	}

	.install-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.installed-badge {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		border: 1px solid color-mix(in srgb, #22c55e 45%, var(--border));
		background: color-mix(in srgb, #22c55e 16%, var(--bg-input));
		color: #22c55e;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
	}

	.chip {
		padding: 2px 9px;
		border-radius: 999px;
		border: 1px solid var(--border);
		font-size: 0.66rem;
		color: var(--text-secondary);
	}

	.dates {
		font-size: 0.68rem;
		color: var(--text-muted);
		margin: 0;
	}

	.gallery {
		display: flex;
		gap: 8px;
		overflow-x: auto;
		padding-bottom: 4px;
	}

	.gallery img {
		height: 120px;
		border-radius: var(--border-radius-sm);
		flex-shrink: 0;
	}

	.description {
		font-size: 0.8rem;
		line-height: 1.55;
		color: var(--text-secondary);
		max-width: 100%;
		overflow-wrap: break-word;
	}

	.description :global(img) {
		max-width: 100%;
	}

	.description :global(a) {
		color: var(--accent);
	}

	.description :global(pre) {
		overflow-x: auto;
		padding: 8px;
		background: var(--bg-input);
		border-radius: var(--border-radius-sm);
	}

	h3 {
		font-size: 0.78rem;
		margin: 0 0 6px;
		color: var(--text-primary);
	}

	.versions-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 220px;
		overflow-y: auto;
	}

	.version-row {
		display: flex;
		justify-content: space-between;
		padding: 6px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.72rem;
		cursor: pointer;
	}

	.version-row:hover:not(:disabled) {
		border-color: var(--accent);
		color: var(--accent);
	}

	.version-row:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.version-date {
		color: var(--text-muted);
	}

	.links {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.link-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-secondary);
		font-size: 0.7rem;
		cursor: pointer;
	}

	.link-btn:hover {
		border-color: var(--accent);
		color: var(--accent);
	}

	.hint {
		color: var(--text-secondary);
		font-size: 0.78rem;
	}

	:global(.spin) {
		animation: tfl-spin 0.8s linear infinite;
	}

	@keyframes tfl-spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
