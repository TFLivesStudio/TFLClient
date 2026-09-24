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

	// Traducción de errores conocidos — el stacktrace crudo de Java sigue
	// abajo para quien lo necesite, esto es un cartel arriba con la
	// explicación en criollo para los casos más comunes. Best-effort, no
	// cubre absolutamente todo lo que puede fallar.
	const KNOWN_ERRORS: { pattern: RegExp; message: string }[] = [
		{
			pattern: /OutOfMemoryError|Java heap space/i,
			message: 'Se quedó sin memoria RAM asignada — probá subir la RAM máxima en Ajustes.'
		},
		{
			pattern: /UnsupportedClassVersionError/i,
			message: 'Esta versión de Minecraft necesita una versión de Java más nueva que la instalada.'
		},
		{
			pattern: /Could not reserve enough space for( the)? object heap/i,
			message: 'Le pediste más RAM de la que tiene disponible tu compu — bajá el máximo en Ajustes.'
		},
		{
			pattern: /A JNI error has occurred/i,
			message: 'Conflicto entre mods, o una versión de Java incompatible con esta instancia.'
		},
		{
			pattern: /Mixin apply .*failed|MixinApplicatorStandard/i,
			message: 'Un mod (mixin) chocó con otro — probá sacar el último mod que instalaste.'
		},
		{
			pattern: /NoClassDefFoundError|ClassNotFoundException/i,
			message: 'Falta una dependencia que un mod necesita — revisá si instalaste todo lo que pedía.'
		},
		{
			pattern: /DuplicateModsFoundException|duplicate mod/i,
			message: 'Tenés el mismo mod instalado dos veces — revisá la lista de mods de la instancia.'
		},
		{
			pattern: /A fatal error has been detected by the Java Runtime Environment/i,
			message:
				'La JVM crasheó de forma nativa (no es un error de un mod puntual) — puede ser drivers de video desactualizados, o falta de RAM real de la compu. Java dejó un hs_err_pid*.log con el detalle en la carpeta de la instancia.'
		}
	];

	let friendlyError = $state<string | null>(null);

	function checkForKnownError(line: string) {
		if (friendlyError) return; // ya hay uno mostrado, no lo piso con otro
		for (const { pattern, message } of KNOWN_ERRORS) {
			if (pattern.test(line)) {
				friendlyError = message;
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
				friendlyError = `La JVM crasheó de forma nativa — resumen del reporte:\n${crashSummary}`;
				return;
			}
		} catch {
			// sin acceso al reporte, seguimos con el mensaje genérico de abajo
		}
		friendlyError =
			code === -9
				? 'El sistema operativo cortó el juego por falta de memoria RAM (no solo la asignada al juego — memoria real de la compu). Cerrá otros programas o bajá la memoria máxima en Ajustes.'
				: code === null
					? 'El juego se cerró de golpe sin avisar — normalmente es el sistema operativo cortando el proceso por falta de RAM. Probá subir la memoria máxima en Ajustes.'
					: `El juego se cerró con código ${code} sin un error reconocible en el log — con varios mods instalados, suele ser falta de RAM. Probá subir la memoria máxima en Ajustes.`;
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
					Corriendo…
				{:else}
					Proceso finalizado{exitCode !== null ? ` (código ${exitCode})` : ''}
				{/if}
				{#if stats}
					· CPU {stats.cpu_percent.toFixed(0)}% · RAM {stats.memory_mb} MB
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

	{#if friendlyError}
		<div class="friendly-error">
			<span>{friendlyError}</span>
			<button type="button" onclick={() => (friendlyError = null)} aria-label="Cerrar aviso">✕</button>
		</div>
	{/if}

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
