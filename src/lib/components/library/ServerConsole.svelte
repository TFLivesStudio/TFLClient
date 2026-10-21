<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { InstanceData } from '$lib/types/types';
	import { sendServerCommand } from '$lib/api';
	import { serverSessions } from '$lib/state/serverSessions.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import type { ProcessStats } from '$lib/console/model';
	import ConsoleView from '$lib/components/console/ConsoleView.svelte';
	import { Send } from 'lucide-svelte';

	let { instance }: { instance: InstanceData } = $props();

	type LogEntry = { stream: 'stdout' | 'stderr'; line: string };
	let lines = $state<LogEntry[]>([]);
	let command = $state('');
	let disposed = false;
	function readStats() {
		return invoke<ProcessStats | null>('get_running_server_stats', { instanceName: instance.name });
	}

	const running = $derived(serverSessions.running.has(instance.name));

	let unlisten: UnlistenFn | undefined;
	onMount(async () => {
		const stopListening = await listen<{ type: string; data: Record<string, unknown> }>(
			'app-event',
			(event) => {
				const payload = event.payload;
				if (payload.type !== 'InstanceLogLine') return;
				const d = payload.data as { instance: string; stream: string; line: string };
				if (d.instance !== instance.name) return;
				lines.push({ stream: d.stream === 'stderr' ? 'stderr' : 'stdout', line: d.line });
				if (lines.length > 2000) lines.splice(0, lines.length - 2000);
			}
		);
		if (disposed) stopListening();
		else unlisten = stopListening;
	});

	onDestroy(() => {
		disposed = true;
		unlisten?.();
	});

	async function handleSend() {
		const cmd = command.trim();
		if (!cmd || !running) return;
		lines.push({ stream: 'stdout', line: `> ${cmd}` });
		command = '';
		try {
			await sendServerCommand(instance.name, cmd);
		} catch (error) {
			lines.push({ stream: 'stderr', line: `[ERROR] ${String(error)}` });
		}
	}
</script>

<div class="console">
	<ConsoleView
		instanceName={instance.name}
		{readStats}
		{running}
		{lines}
		empty={running ? t('serverConsole.waitingOutput') : t('serverConsole.notRunning')}
	/>
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
		<button
			type="submit"
			class="send-btn"
			disabled={!running || !command.trim()}
			aria-label={t('serverConsole.send')}
		>
			<Send size={14} />
		</button>
	</form>
</div>

<style>
	.console {
		display: flex;
		flex-direction: column;
		gap: 8px;
		height: min(680px, calc(100vh - 230px));
		min-height: 300px;
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
