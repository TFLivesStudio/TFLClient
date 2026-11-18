<script lang="ts">
	import { Server, Terminal, Network, Folder } from 'lucide-svelte';
	import type { InstanceData, ServerConnectionInfo, ServerStatus } from '$lib/types/types';
	import { vr } from '$lib/i18n/refresh';
	import { tr } from '$lib/i18n/experience';
	import { pingServer } from '$lib/api/servers';
	let {
		instance,
		running,
		connection,
		onConsole,
		onFiles
	}: {
		instance: InstanceData;
		running: boolean;
		connection: ServerConnectionInfo | null;
		onConsole: () => void;
		onFiles: () => void;
	} = $props();

	let live = $state<ServerStatus | null>(null);
	$effect(() => {
		const port = connection?.port;
		const lan = connection?.local_ip;
		live = null;
		if (!running || !port) return;
		let stale = false;
		let timer: ReturnType<typeof setTimeout> | undefined;
		async function poll() {
			try {
				let status = await pingServer(`127.0.0.1:${port}`);
				if (!stale && !status.online && lan && lan !== '127.0.0.1')
					status = await pingServer(`${lan}:${port}`);
				if (!stale) live = status.online ? status : null;
			} catch {
				if (!stale) live = null;
			} finally {
				if (!stale) timer = setTimeout(poll, 15000);
			}
		}
		void poll();
		return () => {
			stale = true;
			clearTimeout(timer);
		};
	});
</script>

<section class="server-dashboard">
	<header>
		<span class="vr-eyebrow"><Server size={17} />{vr('session')}</span>
		<h3>{instance.name}</h3>
		<p class="vr-muted">{vr('serverHint')}</p>
	</header>
	<div class="server-facts">
		<div>
			<span>{vr('status')}</span><strong class:online={running}
				>{running ? vr('running') : vr('stopped')}</strong
			>
		</div>
		<div>
			<span>{vr('server')}</span><strong>{instance.server_type} · {instance.mc_version}</strong>
		</div>
		<div>
			<span><Network size={13} /> LAN</span><strong
				>{connection?.local_ip ? `${connection.local_ip}:${connection.port}` : '—'}</strong
			>
		</div>
		{#if running}
			<div>
				<span>{tr('Jugadores', 'Players')}</span>
				<strong
					>{live?.players_online != null && live.players_max != null
						? `${live.players_online} / ${live.players_max}`
						: tr('Sin datos', 'No data')}</strong
				>
			</div>
		{/if}
	</div>
	<div class="server-links">
		<button class="vr-control" onclick={onConsole}><Terminal size={17} />{vr('console')}</button
		><button class="vr-control" onclick={onFiles}><Folder size={17} />{vr('technical')}</button>
	</div>
</section>

<style>
	.server-dashboard {
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		background: var(--surface-card);
		padding: 24px;
		margin-bottom: 20px;
	}
	.server-dashboard h3 {
		font-size: 1.6rem;
		letter-spacing: -0.03em;
		margin: 14px 0 8px;
	}
	.server-facts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(135px, 1fr));
		gap: 18px;
		padding: 24px 0;
		margin: 18px 0;
		border-block: 1px solid var(--border);
	}
	.server-facts > div {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.server-facts span {
		display: flex;
		align-items: center;
		gap: 5px;
		color: var(--text-muted);
		font-size: var(--text-caption);
	}
	.server-facts strong {
		font-size: var(--text-small);
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.online {
		color: var(--color-success);
	}
	.server-links {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
	}
	@media (max-width: 850px) {
		.server-facts {
			grid-template-columns: 1fr 1fr;
			gap: 16px;
		}
		.server-facts > div:last-child {
			grid-column: 1/-1;
		}
	}
</style>
