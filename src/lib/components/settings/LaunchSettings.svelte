<script lang="ts">
	import { appState } from '$lib/state/state.svelte';
	import { updateSettings } from '$lib/api';
	import { tr } from '$lib/i18n/experience';
	import SettingGroup from './SettingGroup.svelte';
	import type { JavaStatus } from '$lib/types/types';
	let { statuses, onSaved }: { statuses: JavaStatus[]; onSaved: () => Promise<void> } = $props();
	const majors = [8, 17, 21, 25] as const;
	type JavaKey = 'jre8_path' | 'jre17_path' | 'jre21_path' | 'jre25_path';
	let paths = $state<Record<JavaKey, string>>({
		jre8_path: appState.settings?.jre8_path ?? '',
		jre17_path: appState.settings?.jre17_path ?? '',
		jre21_path: appState.settings?.jre21_path ?? '',
		jre25_path: appState.settings?.jre25_path ?? ''
	});
	let jvm = $state(appState.settings?.jvm_args ?? '');
	let environment = $state(
		Object.entries(appState.settings?.env_vars ?? {})
			.map(([k, v]) => `${k}=${v}`)
			.join('\n')
	);
	let error = $state(''),
		busy = $state(false),
		saved = $state(false);
	async function save() {
		if (!appState.settings) return;
		const env: Record<string, string> = {};
		for (const line of environment.split(/\r?\n/).filter((l) => l.trim())) {
			const equal = line.indexOf('='),
				key = line.slice(0, equal).trim();
			if (
				equal < 1 ||
				!/^[A-Za-z_][A-Za-z0-9_]*$/.test(key) ||
				Object.hasOwn(env, key) ||
				line.includes('\0')
			) {
				error = tr(
					'Usa una variable NOMBRE=valor por línea, sin nombres repetidos.',
					'Use one NAME=value variable per line, without duplicate names.'
				);
				return;
			}
			Object.defineProperty(env, key, { value: line.slice(equal + 1), enumerable: true });
		}
		for (const value of Object.values(paths))
			if (value && !/^(?:\/|[A-Za-z]:[\\/]|\\\\)/.test(value)) {
				error = tr(
					'Introduce una ruta absoluta a Java o deja el campo vacío.',
					'Enter an absolute Java path or leave the field empty.'
				);
				return;
			}
		if (jvm.includes('\0')) {
			error = tr(
				'Los argumentos contienen un carácter no válido.',
				'Arguments contain an invalid character.'
			);
			return;
		}
		busy = true;
		error = '';
		saved = false;
		try {
			const next = { ...appState.settings, ...paths, jvm_args: jvm, env_vars: env };
			await updateSettings(next);
			appState.settings = next;
			await onSaved();
			saved = true;
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
</script>

<div class="launch-settings">
	<SettingGroup
		id="java"
		hint={tr(
			'El launcher instala automáticamente la versión necesaria. Deja las rutas vacías para usar esa selección; una ruta propia se comprobará al iniciar el juego.',
			'The launcher installs the required runtime automatically. Leave paths empty to use it; custom paths are checked when launching the game.'
		)}
	>
		<div class="java-paths">
			{#each majors as major (major)}{@const key = `jre${major}_path` as JavaKey}{@const installed =
					statuses.find((s) => s.major === major)?.installed}<label
					><span
						>Java {major}<small
							>{installed
								? tr('Disponible', 'Available')
								: tr('Se instalará cuando haga falta', 'Installed when needed')}</small
						></span
					><input
						type="text"
						bind:value={paths[key]}
						placeholder={tr('Automático', 'Automatic')}
						aria-label={`Java ${major} · ${tr('Ruta', 'Path')}`}
						spellcheck="false"
					/></label
				>{/each}
		</div>
	</SettingGroup>
	<SettingGroup
		id="jvm"
		hint={tr(
			'Opciones adicionales para la máquina virtual de Java. Se pasan directamente a Java, sin ejecutar una consola del sistema. Deja vacío si no necesitas opciones propias.',
			'Additional Java virtual machine options. Passed directly to Java without executing a system shell. Leave empty unless you need custom flags.'
		)}
		><textarea
			aria-label={tr('Argumentos de Java', 'Java arguments')}
			rows="3"
			bind:value={jvm}
			placeholder="-XX:+UseG1GC"
			spellcheck="false"></textarea></SettingGroup
	>
	<SettingGroup
		id="environment"
		hint={tr(
			'Una variable NOMBRE=valor por línea. Se aplica a los juegos que heredan la configuración global. No compartas capturas con valores privados.',
			'One NAME=value variable per line. Applies to games inheriting global settings. Avoid sharing screenshots with private values.'
		)}
		><textarea
			aria-label={tr('Variables de entorno', 'Environment variables')}
			rows="3"
			bind:value={environment}
			placeholder="NOMBRE=valor"
			spellcheck="false"></textarea></SettingGroup
	>
	<div class="launch-actions">
		<button type="button" class="vr-control primary" disabled={busy} onclick={save}
			>{busy
				? tr('Guardando…', 'Saving…')
				: tr('Guardar opciones de inicio', 'Save launch options')}</button
		>{#if saved}<span role="status">{tr('Guardado', 'Saved')}</span>{/if}
	</div>
	{#if error}<p role="alert">{error}</p>{/if}
</div>

<style>
	.launch-settings {
		display: grid;
		gap: 26px;
	}
	.java-paths {
		display: grid;
		gap: 12px;
	}
	label {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		font-size: 13px;
	}
	label span {
		min-width: 100px;
	}
	small {
		display: block;
		margin-top: 4px;
		color: var(--text-secondary);
		font-size: 11px;
	}
	input,
	textarea {
		background: var(--bg-input);
		border: 1px solid var(--border);
		color: var(--text-primary);
		padding: 10px 12px;
		border-radius: var(--border-radius-sm);
		font-size: 12px;
		font-family: ui-monospace, monospace;
		min-width: 0;
	}
	input {
		flex: 1;
		max-width: 62%;
	}
	textarea {
		width: 100%;
		resize: vertical;
		box-sizing: border-box;
	}
	.launch-actions {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 12px;
		color: var(--text-secondary);
	}
	p[role='alert'] {
		color: var(--color-error);
		font-size: 13px;
	}
</style>
