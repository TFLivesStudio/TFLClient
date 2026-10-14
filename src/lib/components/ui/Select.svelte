<script lang="ts">
	import { ChevronDown } from 'lucide-svelte';

	/** `<select>` nativo estilizado con los tokens — Entrega A. A propósito
	 * no reinventa un dropdown custom (el encargo pide foco en que se vea
	 * coherente, no un componente de posicionamiento nuevo); el ChevronDown
	 * es solo decorativo, superpuesto sobre el `<select>` real. */
	let {
		value = $bindable(''),
		options,
		label,
		error,
		disabled = false,
		id,
		...rest
	}: {
		value?: string;
		options: { value: string; label: string }[];
		label?: string;
		error?: string;
		disabled?: boolean;
		id?: string;
		[key: string]: unknown;
	} = $props();

	const autoId = $props.id();
	const selectId = $derived(id ?? autoId);
	const errorId = $derived(`${selectId}-error`);
</script>

<div class="tfl-field">
	{#if label}
		<label class="tfl-field-label" for={selectId}>{label}</label>
	{/if}
	<div class="tfl-select-wrap">
		<select
			id={selectId}
			class="tfl-select"
			class:error={!!error}
			{disabled}
			bind:value
			aria-invalid={!!error}
			aria-describedby={error ? errorId : undefined}
			{...rest}
		>
			{#each options as option (option.value)}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
		<ChevronDown size={16} class="tfl-select-chevron" aria-hidden="true" />
	</div>
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

	.tfl-select-wrap {
		position: relative;
		display: flex;
	}

	.tfl-select {
		width: 100%;
		height: var(--control-size-normal);
		padding: 0 34px 0 12px;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--border);
		background: var(--bg-input);
		color: var(--text-primary);
		font-family: inherit;
		font-size: var(--text-body);
		line-height: var(--line-height-body);
		appearance: none;
		cursor: pointer;
		transition:
			border-color var(--duration-micro) var(--easing-standard),
			background-color var(--duration-micro) var(--easing-standard);
	}

	.tfl-select:hover:not(:disabled) {
		border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
	}

	.tfl-select:focus-visible {
		border-color: var(--accent);
	}

	.tfl-select:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.tfl-select.error {
		border-color: var(--color-error);
	}

	:global(.tfl-select-chevron) {
		position: absolute;
		right: 10px;
		top: 50%;
		transform: translateY(-50%);
		color: var(--text-muted);
		pointer-events: none;
	}

	.tfl-field-error {
		font-size: var(--text-caption);
		line-height: var(--line-height-caption);
		color: var(--color-error);
	}
</style>
