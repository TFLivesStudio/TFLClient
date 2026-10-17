<script lang="ts">
	/** Anillo que se llena/vacía en tiempo real — reemplaza al número/%
	 * suelto que pedía la ventana de log (ver InstanceLogWindow.svelte).
	 * `value` ya viene clampeado 0-100 por quien lo usa (acá solo dibuja);
	 * `displayValue` es el texto libre a mostrar adentro (puede ser "42%",
	 * "1.9 / 4 GB", "2.3 MB/s" — cada caller decide su propio formato). */
	let {
		value,
		label,
		displayValue,
		icon: Icon
	}: {
		value: number;
		label: string;
		displayValue: string;
		icon?: typeof import('lucide-svelte').Boxes;
	} = $props();

	const RADIUS = 26;
	const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
	const clamped = $derived(Math.min(100, Math.max(0, value)));
	const offset = $derived(CIRCUMFERENCE * (1 - clamped / 100));
</script>

<div class="gauge">
	<div class="gauge-ring-wrap">
		<svg viewBox="0 0 64 64" width="56" height="56">
			<circle class="gauge-track" cx="32" cy="32" r={RADIUS} />
			<circle
				class="gauge-fill"
				cx="32"
				cy="32"
				r={RADIUS}
				stroke-dasharray={CIRCUMFERENCE}
				stroke-dashoffset={offset}
			/>
		</svg>
		{#if Icon}
			<div class="gauge-center"><Icon size={13} /></div>
		{/if}
	</div>
	<div class="gauge-text">
		<span class="gauge-label">{label}</span>
		<span class="gauge-value">{displayValue}</span>
	</div>
</div>

<style>
	.gauge {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.gauge-ring-wrap {
		position: relative;
		flex-shrink: 0;
	}

	svg {
		/* El anillo arranca arriba (12 en punto) y crece en sentido horario,
		   como cualquier medidor — por defecto SVG lo dibuja desde las 3 en
		   punto. */
		transform: rotate(-90deg);
		display: block;
	}

	.gauge-track,
	.gauge-fill {
		fill: none;
		stroke-width: 5;
	}

	.gauge-track {
		stroke: var(--border, rgba(255, 255, 255, 0.14));
	}

	.gauge-fill {
		stroke: var(--accent, #ff7a2e);
		stroke-linecap: round;
		transition: stroke-dashoffset 0.5s ease;
	}

	@media (prefers-reduced-motion: reduce) {
		.gauge-fill {
			transition: none;
		}
	}

	.gauge-center {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted, #8b93a7);
		pointer-events: none;
	}

	.gauge-text {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}

	.gauge-label {
		font-size: 0.62rem;
		font-weight: 700;
		letter-spacing: 0.4px;
		text-transform: uppercase;
		color: var(--text-muted, #8b93a7);
	}

	.gauge-value {
		font-size: 0.8rem;
		font-weight: 700;
		color: var(--text-primary, #e4e6eb);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
</style>
