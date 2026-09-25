<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { invoke } from '@tauri-apps/api/core';
	import { t, type TranslationKey } from '$lib/i18n/index.svelte';
	import { Square, X, Loader2 } from 'lucide-svelte';

	const instanceName =
		(window as unknown as { __TFL_LOG_INSTANCE__?: string }).__TFL_LOG_INSTANCE__ ?? '';

	interface LogLine {
		stream: 'stdout' | 'stderr';
		line: string;
	}

	const MAX_LINES = 5000;

	interface ProcessStats {
		cpu_percent: number;
		memory_mb: number;
	}

	let lines = $state<LogLine[]>([]);
	let exitCode = $state<number | null | undefined>(undefined);
	let stopping = $state(false);
	let autoScroll = $state(true);
	let stats = $state<ProcessStats | null>(null);
	let logEl: HTMLDivElement;
	let unlisten: UnlistenFn | undefined;
	let statsInterval: ReturnType<typeof setInterval> | undefined;

	async function pollStats() {
		try {
			stats = await invoke<ProcessStats | null>('get_running_instance_stats');
		} catch {
			stats = null;
		}
	}

	// Deja de pollear apenas el proceso termina — no tiene sentido seguir
	// pidiendo stats de un PID que ya no existe.
	$effect(() => {
		if (exitCode !== undefined && statsInterval) {
			clearInterval(statsInterval);
			statsInterval = undefined;
			stats = null;
		}
	});

	async function handleStop() {
		stopping = true;
		try {
			await invoke('stop_running_instance');
		} catch (e) {
			lines = [...lines, { stream: 'stderr', line: t('instanceLogWindow.stopFailed', { error: String(e) }) }];
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

	// Traducción de errores conocidos — el stacktrace crudo de Java sigue
	// abajo para quien lo necesite, esto es un cartel arriba con la
	// explicación en criollo para los casos más comunes. Best-effort, no
	// cubre absolutamente todo lo que puede fallar.
	// Guarda la key de traducción (no el texto resuelto) para que, sin
	// importar cuándo dispare el patrón, el mensaje salga en el idioma
	// activo en ese momento — no en el que estaba activo cuando se montó
	// el componente.
	const KNOWN_ERRORS: { pattern: RegExp; key: TranslationKey }[] = [
		{
			pattern: /OutOfMemoryError|Java heap space/i,
			key: 'instanceLogWindow.errors.outOfMemory'
		},
		{
			pattern: /UnsupportedClassVersionError/i,
			key: 'instanceLogWindow.errors.unsupportedJavaVersion'
		},
		{
			pattern: /Could not reserve enough space for( the)? object heap/i,
			key: 'instanceLogWindow.errors.insufficientSystemRam'
		},
		{
			pattern: /A JNI error has occurred/i,
			key: 'instanceLogWindow.errors.jniError'
		},
		{
			pattern: /Mixin apply .*failed|MixinApplicatorStandard/i,
			key: 'instanceLogWindow.errors.mixinConflict'
		},
		{
			pattern: /NoClassDefFoundError|ClassNotFoundException/i,
			key: 'instanceLogWindow.errors.missingDependency'
		},
		{
			pattern: /DuplicateModsFoundException|duplicate mod/i,
			key: 'instanceLogWindow.errors.duplicateMod'
		},
		{
			pattern: /A fatal error has been detected by the Java Runtime Environment/i,
			key: 'instanceLogWindow.errors.nativeCrash'
		}
	];

	let friendlyError = $state<string | null>(null);

	function checkForKnownError(line: string) {
		if (friendlyError) return; // ya hay uno mostrado, no lo piso con otro
		for (const { pattern, key } of KNOWN_ERRORS) {
			if (pattern.test(line)) {
				friendlyError = t(key);
				return;
			}
		}
	}

	// Si terminó mal (código distinto de 0, o sin código porque el sistema
	// operativo mató el proceso) y ningún patrón conocido de stderr avisó
	// nada — el caso típico es el SO cortando el proceso por falta de RAM
	// (OOM killer) sin que Java llegue a loguear un OutOfMemoryError propio,
	// así el log corta de golpe sin ninguna excepción visible. Antes de caer
	// al mensaje genérico, se intenta el resumen real del hs_err_pid*.log
	// que la JVM deja al crashear nativo — mucho más útil que adivinar.
	async function handleExit(code: number | null) {
		if (code === 0 || friendlyError) return;
		try {
			const crashSummary = await invoke<string | null>('get_crash_report', {
				name: instanceName
			});
			if (crashSummary) {
				friendlyError = t('instanceLogWindow.crashSummary', { summary: crashSummary });
				return;
			}
		} catch {
			// sin acceso al reporte, seguimos con el mensaje genérico de abajo
		}
		friendlyError =
			code === -9
				? t('instanceLogWindow.exitOom')
				: code === null
					? t('instanceLogWindow.exitNullCode')
					: t('instanceLogWindow.exitOtherCode', { code });
	}

	onMount(async () => {
		unlisten = await listen<{ type: string; data: Record<string, unknown> }>('app-event', (event) => {
			const payload = event.payload;
			if (payload.type === 'InstanceLogLine') {
				const d = payload.data as { instance: string; stream: string; line: string };
				if (d.instance !== instanceName) return;
				const next = [...lines, { stream: d.stream as 'stdout' | 'stderr', line: d.line }];
				lines = next.length > MAX_LINES ? next.slice(next.length - MAX_LINES) : next;
				if (d.stream === 'stderr') checkForKnownError(d.line);
				tick().then(() => {
					if (autoScroll && logEl) logEl.scrollTop = logEl.scrollHeight;
				});
			} else if (payload.type === 'InstanceExited') {
				const d = payload.data as { instance: string; code: number | null };
				if (d.instance !== instanceName) return;
				exitCode = d.code;
				void handleExit(d.code);
			}
		});
		pollStats();
		statsInterval = setInterval(pollStats, 2000);
	});

	onDestroy(() => {
		unlisten?.();
		clearInterval(statsInterval);
	});
</script>

<div class="log-window">
	<header>
		<div>
			<h1>{instanceName}</h1>
			<span class="status" class:exited={exitCode !== undefined}>
				{#if exitCode === undefined}
					{t('instanceLogWindow.running')}
				{:else}
					{t('instanceLogWindow.exited', {
						codeSuffix: exitCode !== null ? t('instanceLogWindow.exitedCodeSuffix', { code: exitCode }) : ''
					})}
				{/if}
				{#if stats}
					{t('instanceLogWindow.stats', { cpu: stats.cpu_percent.toFixed(0), ram: stats.memory_mb })}
				{/if}
			</span>
		</div>
		<div class="actions">
			<button type="button" disabled={exitCode !== undefined || stopping} onclick={handleStop}>
				{#if stopping}<Loader2 size={13} class="spin" />{:else}<Square size={13} />{/if}
				{t('instanceLogWindow.stop')}
			</button>
			<button type="button" onclick={handleClose}><X size={13} /> {t('settings.close')}</button>
		</div>
	</header>

	{#if friendlyError}
		<div class="friendly-error">
			<span>{friendlyError}</span>
			<button type="button" onclick={() => (friendlyError = null)} aria-label={t('instanceLogWindow.closeNotice')}>✕</button>
		</div>
	{/if}

	<div class="log-body" bind:this={logEl} onscroll={handleScroll}>
		{#each lines as l, i (i)}
			<div class="line" class:err={l.stream === 'stderr'}>{l.line}</div>
		{/each}
		{#if lines.length === 0}
			<p class="empty">{t('instanceLogWindow.waitingOutput')}</p>
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

	.friendly-error {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 10px;
		padding: 8px 14px;
		background: rgba(245, 158, 11, 0.14);
		border-bottom: 1px solid rgba(245, 158, 11, 0.35);
		color: #f5c518;
		font-size: 0.74rem;
	}

	.friendly-error span {
		white-space: pre-wrap;
		max-height: 200px;
		overflow-y: auto;
	}

	.friendly-error button {
		flex-shrink: 0;
		border: none;
		background: transparent;
		color: inherit;
		cursor: pointer;
		font-size: 0.8rem;
		opacity: 0.75;
	}

	.friendly-error button:hover {
		opacity: 1;
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
