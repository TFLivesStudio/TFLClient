<script lang="ts">
	/** Input de texto compartido — Entrega A. Label opcional (asociado por
	 * `id`, generado automáticamente con `$props.id()` si no se pasa uno),
	 * estado de error con `aria-invalid`/`aria-describedby`. */
	let {
		value = $bindable(''),
		label,
		error,
		disabled = false,
		id,
		type = 'text',
		placeholder,
		...rest
	}: {
		value?: string;
		label?: string;
		error?: string;
		disabled?: boolean;
		id?: string;
		type?: 'text' | 'password' | 'email' | 'number' | 'search';
		placeholder?: string;
		[key: string]: unknown;
	} = $props();

	const autoId = $props.id();
	const inputId = $derived(id ?? autoId);
	const errorId = $derived(`${inputId}-error`);
</script>

<div class="tfl-field">
	{#if label}
		<label class="tfl-field-label" for={inputId}>{label}</label>
	{/if}
	<input
		id={inputId}
		class="tfl-field-input"
		class:error={!!error}
		{type}
		{placeholder}
		{disabled}
		bind:value
		aria-invalid={!!error}
		aria-describedby={error ? errorId : undefined}
		{...rest}
	/>
	{#if error}
		<p class="tfl-field-error" id={errorId}>{error}</p>
	{/if}
</div>

<style>
	.tfl-field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		width: 100%;
	}

	.tfl-field-label {
		font-size: var(--text-small);
		line-height: var(--line-height-small);
		font-weight: 600;
		color: var(--text-secondary);
	}

	.tfl-field-input {
		height: var(--control-size-normal);
		padding: 0 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-family: inherit;
		font-size: var(--text-body);
		line-height: var(--line-height-body);
		transition:
			border-color var(--duration-micro) var(--easing-standard),
			background-color var(--duration-micro) var(--easing-standard);
	}

	.tfl-field-input::placeholder {
		color: var(--text-muted);
	}

	.tfl-field-input:hover:not(:disabled) {
		border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
	}

	.tfl-field-input:focus-visible {
		border-color: var(--accent);
	}

	.tfl-field-input:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.tfl-field-input.error {
		border-color: var(--color-error);
	}

	.tfl-field-error {
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		color: var(--color-error);
	}
</style>
