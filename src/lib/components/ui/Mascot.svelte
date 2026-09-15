<script lang="ts">
	import { MASCOTS, type MascotId } from '$lib/mascots';

	let {
		id,
		size = 32
	}: {
		id: MascotId;
		size?: number;
	} = $props();

	const m = $derived(MASCOTS.find((x) => x.id === id) ?? MASCOTS[0]);
</script>

<!-- Mascotas simples estilo "ip-as-logo": silueta redondeada dominante,
	 4-7 formas grandes, 2-3 colores, sin detalles finos. Reemplazan el
	 Steve genérico de Crafatar para cuentas offline/cracked (que no
	 tienen skin real) — cada cuenta arranca con una al azar y se puede
	 cambiar por otra del mismo set. -->
<svg viewBox="0 0 64 64" width={size} height={size} xmlns="http://www.w3.org/2000/svg">
	<rect width="64" height="64" rx="16" fill={m.bg} />

	{#if m.id === 'fox'}
		<path d="M14 30 22 16 30 30Z" fill={m.primary} />
		<path d="M34 30 42 16 50 30Z" fill={m.primary} />
		<circle cx="32" cy="36" r="18" fill={m.primary} />
		<path d="M32 40 22 48 32 54 42 48Z" fill={m.secondary} />
		<circle cx="25" cy="34" r="3.4" fill="#1a1a1a" />
		<circle cx="39" cy="34" r="3.4" fill="#1a1a1a" />
	{:else if m.id === 'bear'}
		<circle cx="18" cy="20" r="8" fill={m.primary} />
		<circle cx="46" cy="20" r="8" fill={m.primary} />
		<circle cx="32" cy="36" r="20" fill={m.primary} />
		<circle cx="32" cy="42" r="8" fill={m.secondary} />
		<circle cx="24" cy="32" r="3.4" fill="#1a1a1a" />
		<circle cx="40" cy="32" r="3.4" fill="#1a1a1a" />
	{:else if m.id === 'cat'}
		<path d="M16 14 26 26 14 30Z" fill={m.primary} />
		<path d="M48 14 38 26 50 30Z" fill={m.primary} />
		<circle cx="32" cy="36" r="19" fill={m.primary} />
		<ellipse cx="32" cy="43" rx="7" ry="5" fill={m.secondary} />
		<circle cx="24" cy="34" r="3.2" fill="#1a1a1a" />
		<circle cx="40" cy="34" r="3.2" fill="#1a1a1a" />
	{:else if m.id === 'owl'}
		<circle cx="32" cy="34" r="20" fill={m.primary} />
		<circle cx="23" cy="30" r="8" fill="#ffffff" />
		<circle cx="41" cy="30" r="8" fill="#ffffff" />
		<circle cx="23" cy="30" r="3.6" fill="#1a1a1a" />
		<circle cx="41" cy="30" r="3.6" fill="#1a1a1a" />
		<path d="M32 38 28 46 36 46Z" fill={m.secondary} />
	{:else if m.id === 'panda'}
		<circle cx="17" cy="18" r="8" fill="#1a1a1a" />
		<circle cx="47" cy="18" r="8" fill="#1a1a1a" />
		<circle cx="32" cy="36" r="20" fill="#f4f4f5" />
		<ellipse cx="23" cy="34" rx="6" ry="7.5" fill="#1a1a1a" />
		<ellipse cx="41" cy="34" rx="6" ry="7.5" fill="#1a1a1a" />
		<circle cx="23" cy="35" r="2.6" fill="#f4f4f5" />
		<circle cx="41" cy="35" r="2.6" fill="#f4f4f5" />
		<ellipse cx="32" cy="44" rx="4" ry="3" fill={m.primary} />
	{:else if m.id === 'frog'}
		<circle cx="21" cy="22" r="9" fill={m.primary} />
		<circle cx="43" cy="22" r="9" fill={m.primary} />
		<circle cx="21" cy="22" r="3.6" fill="#1a1a1a" />
		<circle cx="43" cy="22" r="3.6" fill="#1a1a1a" />
		<rect x="14" y="30" width="36" height="22" rx="11" fill={m.primary} />
		<path d="M22 42 Q32 48 42 42" stroke={m.secondary} stroke-width="3" fill="none" stroke-linecap="round" />
	{:else if m.id === 'bee'}
		<circle cx="32" cy="34" r="19" fill={m.secondary} />
		<rect x="13" y="26" width="38" height="7" fill={m.primary} />
		<rect x="13" y="40" width="38" height="7" fill={m.primary} />
		<path d="M16 16 24 24 M48 16 40 24" stroke="#1a1a1a" stroke-width="3" stroke-linecap="round" />
		<circle cx="26" cy="34" r="3" fill="#1a1a1a" />
		<circle cx="38" cy="34" r="3" fill="#1a1a1a" />
	{:else}
		<circle cx="32" cy="38" r="19" fill={m.primary} />
		<path d="M13 30 A19 22 0 0 1 51 30 Z" fill={m.secondary} />
		<circle cx="25" cy="38" r="3.4" fill="#1a1a1a" />
		<circle cx="39" cy="38" r="3.4" fill="#1a1a1a" />
		<ellipse cx="32" cy="46" rx="5" ry="3" fill="#ffffff" />
	{/if}
</svg>
