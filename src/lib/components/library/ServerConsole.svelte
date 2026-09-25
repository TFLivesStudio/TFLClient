<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { InstanceData } from '$lib/types/types';
	import { sendServerCommand } from '$lib/api/tflApi';
	import { serverSessions } from '$lib/state/serverSessions.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { Send } from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();

	type LogEntry = { stream: 'stdout' | 'stderr'; line: string };
	let lines = $state<LogEntry[]>([]);
	let command = $state('');
	let logEl = $state<HTMLDivElement | undefined>(undefined);
	let stickToBottom = $state(true);

	const running = $derived(serverSessions.running.has(instance.name));

	let unlisten: UnlistenFn | undefined;
	onMount(async () => {
		unlisten = await listen<{ type: string; data: Record<string, unknown> }>('app-event', (event) => {
			const payload = event.payload;
			if (payload.type !== 'InstanceLogLine') return;
			const d = payload.data as { instance: string; stream: string; line: string };
			if (d.instance !== instance.name) return;
			lines.push({ stream: d.stream === 'stderr' ? 'stderr' : 'stdout', line: d.line });
			if (lines.length > 2000) lines.splice(0, lines.length - 2000);
			if (stickToBottom) {
				tick().then(() => {
					if (logEl) logEl.scrollTop = logEl.scrollHeight;
				});
			}
		});
	});

	onDestroy(() => unlisten?.());

	function handleScroll() {
		if (!logEl) return;
		stickToBottom = logEl.scrollHeight - logEl.scrollTop - logEl.clientHeight < 40;
	}

	async function handleSend() {
		const cmd = command.trim();
		if (!cmd || !running) return;
		lines.push({ stream: 'stdout', line: `> ${cmd}` });
		command = '';
		try {
			await sendServerCommand(instance.name, cmd);
		} catch {
			// el error real (si lo hay) va a aparecer como línea de log del
			// propio proceso — no hace falta duplicar el aviso acá
		}
	}
</script>

<div class="console">
	<div class="log" bind:this={logEl} onscroll={handleScroll}>
		{#if lines.length === 0}
			<p class="empty">{running ? t('serverConsole.waitingOutput') : t('serverConsole.notRunning')}</p>
		{:else}
			{#each lines as entry, i (i)}
				<div class="line" class:stderr={entry.stream === 'stderr'}>{entry.line}</div>
			{/each}
		{/if}
	</div>
	<form
		class="command-row"
		onsubmit={(e) => {
			e.preventDefault();
			handleSend();
		}}
	>
		<input
			type="text"
			bind:value={command}
			disabled={!running}
			placeholder={running ? t('serverConsole.commandPlaceholder') : t('serverConsole.notRunning')}
			autocomplete="off"
		/>
		<button type="submit" class="send-btn" disabled={!running || !command.trim()} aria-label={t('serverConsole.send')}>
			<Send size={14} />
		</button>
	</form>
</div>

<style>
	.console {
		display: flex;
		flex-direction: column;
		gap: 8px;
		height: 420px;
	}

	.log {
		flex: 1;
		overflow-y: auto;
		padding: 10px 12px;
		border-radius: var(--border-radius);
		border: 1px solid var(--border);
		background: #0a0a0a;
		font-family: ui-monospace, 'SF Mono', 'Cascadia Code', monospace;
		font-size: 0.74rem;
		line-height: 1.5;
	}

	.empty {
		color: var(--text-muted);
	}

	.line {
		white-space: pre-wrap;
		word-break: break-word;
		color: #d4d4d8;
	}

	.line.stderr {
		color: #f87171;
	}

	.command-row {
		display: flex;
		gap: 8px;
	}

	.command-row input {
		flex: 1;
		padding: 9px 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-family: ui-monospace, 'SF Mono', 'Cascadia Code', monospace;
		font-size: 0.8rem;
	}

	.command-row input:disabled {
		opacity: 0.6;
	}

	.send-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 38px;
		flex-shrink: 0;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--accent);
		color: var(--accent-text);
		cursor: pointer;
	}

	.send-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		background: var(--bg-input);
		color: var(--text-primary);
	}
</style>
