<script lang="ts">
	let {
		width = '100%',
		height = '100%',
		class: className = ''
	}: {
		width?: string | number;
		height?: string | number;
		class?: string;
	} = $props();
</script>

<!-- Marca geométrica T/F/L — construida con barras superpuestas, cada una
	 con su propia opacidad sobre currentColor (así hereda el acento activo
	 como el resto de los íconos de la app en vez de traer su propio color
	 fijo). Reemplaza el hexágono placeholder original. -->
<svg viewBox="0 0 64 64" {width} {height} class={className} xmlns="http://www.w3.org/2000/svg">
	<!-- T bar -->
	<rect x="12" y="14" width="40" height="7" rx="2" fill="currentColor" />
	<!-- T stem -->
	<rect x="12" y="21" width="10" height="29" rx="2" fill="currentColor" fill-opacity="0.9" />
	<!-- F mid-bar -->
	<rect x="22" y="25" width="18" height="6" rx="2" fill="currentColor" fill-opacity="0.75" />
	<!-- L foot -->
	<rect x="34" y="35" width="18" height="7" rx="2" fill="currentColor" fill-opacity="0.6" />
	<!-- Corner accent dot -->
	<circle cx="53" cy="11" r="3" fill="currentColor" fill-opacity="0.5" />
</svg>
