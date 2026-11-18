<script lang="ts">
	import { Globe2, RotateCcw } from 'lucide-svelte';
	import {
		i18nState,
		languageOptions,
		languageState,
		setLocale,
		translateMessage
	} from '$lib/i18n/index.svelte';
	import { tr } from '$lib/i18n/experience';
	const options = $derived(languageOptions());
</script>

<div class="language-picker">
	<div
		class="language-options"
		role="group"
		aria-label={tr('Idioma de la aplicación', 'Application language')}
	>
		{#each options as option (option.locale)}
			<button
				type="button"
				class="language-option"
				class:chosen={i18nState.locale === option.locale}
				aria-pressed={i18nState.locale === option.locale}
				disabled={languageState.saving}
				onclick={() => void setLocale(option.locale)}
			>
				{#if option.flag}<img src={option.flag} alt="" width="28" height="19" />{:else}<Globe2
						size={22}
						aria-hidden="true"
					/>{/if}
				<span
					><strong lang={option.locale}>{option.name}</strong><small
						>{option.locale}{option.packageId ? ' · Addon' : ''}</small
					></span
				>
			</button>
		{/each}
	</div>
	<p class="language-hint">
		{translateMessage(
			'language.available',
			options.length === 1 ? '{{count}} idioma disponible' : '{{count}} idiomas disponibles',
			'{{count}} languages available',
			{ count: options.length }
		)}
	</p>
	<p class="language-hint">
		{tr(
			'Instala y activa un paquete de idioma en Addons para añadir otra lengua. Las claves que falten usan su idioma de respaldo.',
			'Install and enable a language package in Addons to add a language. Missing keys use its fallback language.'
		)}
	</p>
	{#if languageState.saving}<p role="status">{tr('Guardando idioma…', 'Saving language…')}</p>{/if}
	{#if languageState.error}<div class="language-error" role="alert">
			<p>{languageState.error}</p>
			{#if languageState.retryLocale}
				{@const retryLocale = languageState.retryLocale}
				<button type="button" onclick={() => void setLocale(retryLocale)}
					><RotateCcw size={14} />{tr('Reintentar', 'Retry')}</button
				>
			{/if}
		</div>{/if}
</div>

<style>
	.language-options {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 230px), 1fr));
		gap: 10px;
	}
	.language-option {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		text-align: start;
		color: var(--text-primary);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm, 8px);
		background: var(--bg-input);
		cursor: pointer;
		font: inherit;
	}
	.language-option.chosen {
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 10%, var(--bg-input));
	}
	.language-option span {
		display: grid;
		gap: 3px;
		min-width: 0;
	}
	strong {
		font-size: 13px;
		overflow-wrap: anywhere;
	}
	small,
	.language-hint {
		color: var(--text-secondary);
		font-size: 12px;
		line-height: 1.5;
	}
	img {
		object-fit: contain;
		flex: none;
	}
	.language-option:focus-visible,
	.language-error button:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 3px;
	}
	.language-error {
		color: var(--color-error);
		font-size: 12px;
	}
	.language-error button {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px;
		color: var(--text-primary);
		background: var(--bg-input);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm, 8px);
		cursor: pointer;
	}
	button:disabled {
		opacity: 0.6;
		cursor: wait;
	}
	:global([data-visual='pixel']) .language-option {
		border-radius: 0;
	}
</style>
