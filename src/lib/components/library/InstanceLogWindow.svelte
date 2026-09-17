<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { invoke } from '@tauri-apps/api/core';
	import { Square, X, Loader2 } from 'lucide-svelte';

	const instanceName =
		(window as unknown as { __TFL_LOG_INSTANCE__?: string }).__TFL_LOG_INSTANCE__ ?? '';

	interface LogLine {
		stream: 'stdout' | 'stderr';
		line: string;
	}

	const MAX_LINES = 5000;

	let lines = $state<LogLine[]>([]);
	let exitCode = $state<number | null | undefined>(undefined);
	let stopping = $state(false);
	let autoScroll = $state(true);
	let logEl: HTMLDivElement;
	let unlisten: UnlistenFn | undefined;

	async function handleStop() {
		stopping = true;
		try {
			await invoke('stop_running_instance');
		} catch (e) {
			lines = [...lines, { stream: 'stderr', line: `No se pudo detener: ${e}` }];
		} finally {
			stopping = false;
		}
	}

	function handleClose() {
		getCurrentWindow().close();
	}

	function handleScroll() {
		if (!logEl) return;
		autoScroll = logEl.scrollHeight - logEl.scrollTop - logEl.clientHeight < 40;
	}

	onMount(async () => {
		unlisten = await listen<{ type: string; data: Record<string, unknown> }>('app-event', (event) => {
			const payload = event.payload;
			if (payload.type === 'InstanceLogLine') {
				const d = payload.data as { instance: string; stream: string; line: string };
				if (d.instance !== instanceName) return;
				const next = [...lines, { stream: d.stream as 'stdout' | 'stderr', line: d.line }];
				lines = next.length > MAX_LINES ? next.slice(next.length - MAX_LINES) : next;
				tick().then(() => {
					if (autoScroll && logEl) logEl.scrollTop = logEl.scrollHeight;
				});
			} else if (payload.type === 'InstanceExited') {
				const d = payload.data as { instance: string; code: number | null };
				if (d.instance !== instanceName) return;
				exitCode = d.code;
			}
		});
	});

	onDestroy(() => {
		unlisten?.();
	});
</script>

<div class="log-window">
	<header>
		<div>
			<h1>{instanceName}</h1>
			<span class="status" class:exited={exitCode !== undefined}>
				{#if exitCode === undefined}
					Corriendo…
				{:else}
					Proceso finalizado{exitCode !== null ? ` (código ${exitCode})` : ''}
				{/if}
			</span>
		</div>
		<div class="actions">
			<button type="button" disabled={exitCode !== undefined || stopping} onclick={handleStop}>
				{#if stopping}<Loader2 size={13} class="spin" />{:else}<Square size={13} />{/if}
				Detener
			</button>
			<button type="button" onclick={handleClose}><X size={13} /> Cerrar</button>
		</div>
	</header>

	<div class="log-body" bind:this={logEl} onscroll={handleScroll}>
		{#each lines as l, i (i)}
			<div class="line" class:err={l.stream === 'stderr'}>{l.line}</div>
		{/each}
		{#if lines.length === 0}
			<p class="empty">Esperando salida del juego…</p>
		{/if}
	</div>
</div>

<style>
	:global(html),
	:global(body) {
		margin: 0;
		height: 100%;
		background: #0b0d12;
	}

	.log-window {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: #0b0d12;
		color: #e4e6eb;
		font-family:
			-apple-system,
			BlinkMacSystemFont,
			'Segoe UI',
			system-ui,
			sans-serif;
	}

	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.08);
		flex-shrink: 0;
	}

	h1 {
		font-size: 0.95rem;
		margin: 0 0 2px;
	}

	.status {
		font-size: 0.72rem;
		color: #8b93a7;
	}

	.status.exited {
		color: #f59e0b;
	}

	.actions {
		display: flex;
		gap: 8px;
	}

	.actions button {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border-radius: 6px;
		border: 1px solid rgba(255, 255, 255, 0.12);
		background: rgba(255, 255, 255, 0.04);
		color: #e4e6eb;
		font-size: 0.75rem;
		cursor: pointer;
	}

	.actions button:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.1);
	}

	.actions button:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.log-body {
		flex: 1;
		overflow-y: auto;
		padding: 10px 14px;
		font-family: 'SF Mono', Consolas, Menlo, monospace;
		font-size: 0.72rem;
		line-height: 1.5;
	}

	.line {
		white-space: pre-wrap;
		word-break: break-word;
		color: #c7cbd4;
	}

	.line.err {
		color: #ff8080;
	}

	.empty {
		color: #5c6270;
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
