<script lang="ts">
	import { onMount } from 'svelte';
	import { FileUp, RefreshCw, Trash2, ShieldCheck, RotateCcw } from 'lucide-svelte';
	import { invoke } from '$lib/api/invoke';
	import { addonState, refreshAddons, changeAddons } from '$lib/addons/state.svelte';
	import type { AddonInspection } from '$lib/addons/model';
	import { tr } from '$lib/i18n/experience';
	import SettingGroup from './SettingGroup.svelte';
	let path = $state('');
	let inspection = $state<AddonInspection | null>(null);
	let busy = $state(false);
	let error = $state('');
	let notice = $state('');
	let removeTarget = $state<{ manifest: { id: string; name: string } } | null>(null);
	let consent = $state(false);
	onMount(() => {
		void refreshAddons();
	});
	async function run(action: () => Promise<void>) {
		if (busy) return;
		busy = true;
		error = '';
		notice = '';
		try {
			await action();
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
	async function inspect() {
		await run(async () => {
			inspection = null;
			consent = false;
			if (!path.trim()) throw new Error(tr('Selecciona un archivo .tfl.', 'Select a .tfl file.'));
			inspection = await invoke<AddonInspection>('addons_inspect', { path: path.trim() });
		});
	}
	async function pick() {
		let selected = false;
		await run(async () => {
			const chosen = await invoke<string | null>('addons_pick_file');
			if (chosen) {
				path = chosen;
				inspection = null;
				consent = false;
				selected = true;
			}
		});
		if (selected) await inspect();
	}
	async function install() {
		if (!inspection || !consent) return;
		const pending = inspection;
		await run(async () => {
			await changeAddons('addons_install', {
				path: path.trim(),
				digest: pending.digest,
				replace: Boolean(pending.installedVersion)
			});
			inspection = null;
			consent = false;
			path = '';
			notice = pending.installedVersion
				? tr(
						'Addon actualizado. Se conserva su estado de activación anterior.',
						'Addon updated. Its previous enabled state is preserved.'
					)
				: tr(
						'Instalado. Activa el addon para usar sus contribuciones.',
						'Installed. Enable the addon to use its contributions.'
					);
		});
	}
	async function action(command: string, args: Record<string, unknown>) {
		await run(() => changeAddons(command, args));
	}
</script>

<div class="addon-settings">
	<SettingGroup id="addons">
		<div class="intro">
			<ShieldCheck size={22} />
			<div>
				<h3>{tr('Personalización local', 'Local customization')}</h3>
				<p>
					{tr(
						'Instala temas y escenas declarativos desde un archivo .tfl. Funcionan sin conexión y no son plugins de servidores Minecraft.',
						'Install declarative themes and scenes from a .tfl file. They work offline and are separate from Minecraft server plugins.'
					)}
				</p>
			</div>
		</div>
		<p class="trust">
			{tr(
				'Esta beta no ejecuta código de addons. No concede acceso a cuentas, archivos arbitrarios, red ni procesos. El nombre del autor es declarado; la extensión y el hash no verifican su identidad.',
				'This beta does not execute addon code. It grants no access to accounts, arbitrary files, network or processes. Author names are self-declared; the extension and hash do not verify identity.'
			)}
		</p>
		<div class="import-row">
			<button type="button" class="vr-control" disabled={busy} onclick={pick}
				><FileUp size={16} />{tr('Seleccionar .tfl', 'Choose .tfl')}</button
			><button type="button" class="vr-control" disabled={busy} onclick={() => run(refreshAddons)}
				><RefreshCw size={15} />{tr('Actualizar lista', 'Refresh list')}</button
			>
		</div>
		<label class="path-label"
			>{tr(
				'Ruta local (también permite selección manual)',
				'Local path (also supports manual selection)'
			)}<input
				value={path}
				oninput={(e) => {
					path = e.currentTarget.value;
					inspection = null;
					consent = false;
				}}
				placeholder="C:\\…\\addon.tfl"
				disabled={busy}
			/></label
		>
		<button type="button" class="vr-control" disabled={busy || !path.trim()} onclick={inspect}
			>{tr('Inspeccionar paquete', 'Inspect package')}</button
		>
		{#if inspection}
			<section class="inspection" aria-label={tr('Inspección del addon', 'Addon inspection')}>
				<h3>{inspection.manifest.name} <small>{inspection.manifest.version}</small></h3>
				<p>{inspection.manifest.description}</p>
				<dl>
					<dt>{tr('Autor declarado', 'Declared author')}</dt>
					<dd>{inspection.manifest.author}</dd>
					<dt>ID</dt>
					<dd>{inspection.manifest.id}</dd>
					<dt>API / launcher</dt>
					<dd>{inspection.manifest.apiVersion} / {inspection.manifest.launcherVersion}</dd>
					<dt>{tr('Contribuciones', 'Contributions')}</dt>
					<dd>
						{inspection.manifest.themes
							.map((t) => t.name)
							.concat(inspection.manifest.scenes.map((s) => s.name))
							.join(' · ')}
					</dd>
					<dt>{tr('Permisos', 'Permissions')}</dt>
					<dd>
						{tr(
							'Ninguno. Solo recursos y tokens declarativos.',
							'None. Declarative resources and tokens only.'
						)}
					</dd>
					<dt>SHA-256</dt>
					<dd class="digest">{inspection.digest}</dd>
				</dl>
				{#if inspection.installedVersion}<p>
						{tr('Actualización local desde', 'Local update from')}
						{inspection.installedVersion}. {tr(
							'Se conserva una versión para volver atrás.',
							'One previous version is kept for rollback.'
						)}
					</p>{/if}
				<label class="consent"
					><input type="checkbox" bind:checked={consent} disabled={busy} />{tr(
						'Acepto instalar estas contribuciones declarativas de este autor no verificado.',
						'I agree to install these declarative contributions from this unverified author.'
					)}</label
				>
				<div class="import-row">
					<button
						type="button"
						class="vr-control primary"
						disabled={busy || !consent}
						onclick={install}
						>{tr(
							inspection.installedVersion ? 'Actualizar addon' : 'Instalar addon',
							inspection.installedVersion ? 'Update addon' : 'Install addon'
						)}</button
					><button
						type="button"
						class="vr-control"
						disabled={busy}
						onclick={() => {
							inspection = null;
							consent = false;
						}}>{tr('Cancelar', 'Cancel')}</button
					>
				</div>
			</section>
		{/if}
	</SettingGroup>
	<section class="safe-mode">
		<label
			><input
				type="checkbox"
				checked={addonState.catalog.safeMode}
				disabled={busy || !addonState.loaded}
				onchange={(e) => action('addons_safe_mode', { enabled: e.currentTarget.checked })}
			/><strong
				>{tr(
					'Perfil seguro: desactivar todos los addons',
					'Safe profile: disable all addons'
				)}</strong
			></label
		>
		<p>
			{tr(
				'Suspende todas las contribuciones sin desinstalar los paquetes. La apariencia y las escenas activas vuelven a la opción predeterminada.',
				'Suspends all contributions without uninstalling packages. Active addon themes and scenes return to their defaults.'
			)}
		</p>
	</section>
	<section aria-label={tr('Addons instalados', 'Installed addons')} class="installed">
		<h3>{tr('Instalados', 'Installed')} <small>({addonState.catalog.addons.length})</small></h3>
		<p>
			{tr(
				'Para actualizar un addon, selecciona su nuevo archivo .tfl y revisa el paquete antes de confirmar.',
				'To update an addon, choose its new .tfl file and review the package before confirming.'
			)}
		</p>
		{#if !addonState.loaded && !addonState.error}<p role="status">
				{tr('Cargando addons…', 'Loading addons…')}
			</p>{:else if addonState.loaded && !addonState.catalog.addons.length}<p class="empty">
				{tr(
					'Todavía no hay addons. Selecciona un .tfl para revisar su contenido antes de instalarlo.',
					'No addons yet. Choose a .tfl to review its contents before installing.'
				)}
			</p>{/if}
		{#each addonState.catalog.addons as addon (addon.manifest.id)}
			<article class="installed-addon">
				<div class="addon-heading">
					<div>
						<h3>{addon.manifest.name} <small>{addon.manifest.version}</small></h3>
						<p>{addon.manifest.author} · {addon.manifest.id}</p>
					</div>
					<label class="enabled"
						><input
							type="checkbox"
							checked={addon.enabled}
							disabled={busy || addonState.catalog.safeMode}
							onchange={(e) =>
								action('addons_set_enabled', {
									id: addon.manifest.id,
									enabled: e.currentTarget.checked
								})}
						/>{tr('Activo', 'Enabled')}</label
					>
				</div>
				<p>{addon.manifest.description}</p>
				<p class="counts">
					{addon.manifest.themes.length}
					{tr('temas', 'themes')} · {addon.manifest.scenes.length}
					{tr('escenas', 'scenes')} · {tr('sin código ejecutable', 'no executable code')}
				</p>
				<div class="import-row">
					{#if addon.rollbackAvailable}<button
							type="button"
							class="vr-control"
							disabled={busy}
							onclick={() => action('addons_rollback', { id: addon.manifest.id })}
							><RotateCcw size={15} />{tr(
								'Volver a la versión anterior',
								'Restore previous version'
							)}</button
						>{/if}<button
						type="button"
						class="vr-control"
						disabled={busy}
						onclick={() => (removeTarget = addon)}
						><Trash2 size={15} />{tr('Desinstalar', 'Uninstall')}</button
					>
				</div>
			</article>
		{/each}
	</section>
	{#if addonState.catalog.brokenAddons?.length}
		<section
			class="installed"
			aria-label={tr('Addons que necesitan recuperación', 'Addons needing recovery')}
		>
			<h3>{tr('Necesitan recuperación', 'Recovery needed')}</h3>
			{#each addonState.catalog.brokenAddons as addon (addon.id)}
				<article class="installed-addon">
					<h3>{addon.id}</h3>
					<p role="alert">{addon.error}</p>
					<div class="import-row">
						{#if addon.rollbackAvailable}<button
								type="button"
								class="vr-control"
								disabled={busy}
								onclick={() => action('addons_rollback', { id: addon.id })}
								><RotateCcw size={15} />{tr(
									'Recuperar versión anterior',
									'Recover previous version'
								)}</button
							>{/if}<button
							type="button"
							class="vr-control"
							disabled={busy}
							onclick={() => (removeTarget = { manifest: { id: addon.id, name: addon.id } })}
							><Trash2 size={15} />{tr('Desinstalar', 'Uninstall')}</button
						>
					</div>
				</article>
			{/each}
		</section>
	{/if}
	{#if removeTarget}<section class="inspection" role="alert">
			<h3>{tr('Desinstalar', 'Uninstall')} {removeTarget.manifest.name}?</h3>
			<p>
				{tr(
					'Se revocan sus contribuciones y se desinstala el addon. Tus instancias y mundos se conservan; la apariencia activa vuelve al valor predeterminado. Pueden conservarse archivos temporales de recuperación hasta la siguiente limpieza.',
					'Uninstalls the addon and revokes its contributions. Your instances and worlds are preserved; active appearance returns to defaults. Temporary recovery files may remain until the next cleanup.'
				)}
			</p>
			<div class="import-row">
				<button
					class="vr-control"
					type="button"
					disabled={busy}
					onclick={async () => {
						const id = removeTarget!.manifest.id;
						await action('addons_remove', { id });
						if (!error) removeTarget = null;
					}}>{tr('Confirmar desinstalación', 'Confirm uninstall')}</button
				><button
					class="vr-control"
					type="button"
					disabled={busy}
					onclick={() => (removeTarget = null)}>{tr('Cancelar', 'Cancel')}</button
				>
			</div>
		</section>{/if}
	{#if busy}<p role="status">{tr('Procesando paquete…', 'Processing package…')}</p>{/if}
	{#if notice}<p role="status">{notice}</p>{/if}
	{#if error || addonState.error}<p class="error" role="alert">{error || addonState.error}</p>{/if}
	{#each addonState.catalog.errors ?? [] as problem (problem)}<p class="error" role="alert">
			{problem}
		</p>{/each}
</div>

<style>
	.addon-settings {
		display: grid;
		gap: 24px;
	}
	.intro,
	.import-row,
	.addon-heading {
		display: flex;
		gap: 12px;
		align-items: center;
	}
	.intro {
		align-items: start;
	}
	.intro :global(svg) {
		flex: none;
		color: var(--accent);
		margin-top: 2px;
	}
	h3 {
		margin: 0;
		font-size: 14px;
		line-height: 1.5;
	}
	p {
		color: var(--text-secondary);
		font-size: 13px;
		line-height: 1.65;
		margin: 6px 0 12px;
	}
	small {
		color: var(--text-secondary);
		font-weight: 400;
	}
	.trust {
		border-left: 2px solid var(--accent);
		padding-left: 12px;
		margin: 18px 0;
	}
	.import-row {
		flex-wrap: wrap;
	}
	button {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		min-height: 36px;
	}
	button:disabled {
		opacity: 0.55;
		cursor: default;
	}
	.path-label {
		display: grid;
		gap: 8px;
		margin: 16px 0 10px;
		font-size: 12px;
		color: var(--text-secondary);
	}
	.path-label input {
		width: 100%;
		min-width: 0;
		padding: 10px;
		color: var(--text-primary);
		background: var(--bg-input);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-sm);
	}
	.inspection,
	.installed-addon,
	.safe-mode {
		min-width: 0;
		overflow-wrap: anywhere;
		padding: 16px;
		border: 1px solid var(--border);
		background: var(--surface-card);
		border-radius: var(--border-radius);
	}
	.inspection {
		margin-top: 16px;
	}
	dl {
		display: grid;
		grid-template-columns: minmax(85px, auto) minmax(0, 1fr);
		gap: 7px 16px;
		font-size: 12px;
		line-height: 1.6;
	}
	dt {
		color: var(--text-secondary);
	}
	dd {
		margin: 0;
		overflow-wrap: anywhere;
	}
	.digest {
		font-family: monospace;
		font-size: 11px;
	}
	.consent,
	.safe-mode label,
	.enabled {
		display: flex;
		gap: 10px;
		align-items: center;
		font-size: 13px;
		line-height: 1.6;
	}
	.consent {
		margin: 18px 0;
	}
	input[type='checkbox'] {
		width: 17px;
		height: 17px;
		accent-color: var(--accent);
		flex: none;
	}
	.installed {
		display: grid;
		gap: 12px;
	}
	.addon-heading {
		justify-content: space-between;
		flex-wrap: wrap;
	}
	.addon-heading > div {
		min-width: 0;
	}
	.addon-heading p {
		margin-bottom: 0;
		font-size: 11px;
	}
	.counts {
		font-size: 12px;
	}
	.empty {
		padding: 18px 0;
	}
	.error {
		color: var(--text-primary);
		border-left: 3px solid #d94a4a;
		padding-left: 12px;
		overflow-wrap: anywhere;
	}
	:global([data-visual='pixel']) .inspection,
	:global([data-visual='pixel']) .installed-addon,
	:global([data-visual='pixel']) .safe-mode {
		border-radius: 0;
	}
</style>
