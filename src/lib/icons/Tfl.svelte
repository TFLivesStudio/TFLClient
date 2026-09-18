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

<!-- Logo oficial: 4 diamantes (cuadrados rotados 45°) alrededor de un
	 centro compartido, con un pequeño espacio entre ellos — vectorizado a
	 mano a partir del PNG que pasó el cliente (proporciones medidas por
	 muestreo de canal alfa, no a ojo). currentColor para heredar el acento
	 activo, igual que el resto de los íconos de la app. -->
<svg viewBox="0 0 64 64" {width} {height} class={className} xmlns="http://www.w3.org/2000/svg">
	<rect x="10.7" y="24.2" width="15.6" height="15.6" rx="2" fill="currentColor" transform="rotate(45 18.5 32)" />
	<rect x="37.7" y="24.2" width="15.6" height="15.6" rx="2" fill="currentColor" transform="rotate(45 45.5 32)" />
	<rect x="24.2" y="10.7" width="15.6" height="15.6" rx="2" fill="currentColor" transform="rotate(45 32 18.5)" />
	<rect x="24.2" y="37.7" width="15.6" height="15.6" rx="2" fill="currentColor" transform="rotate(45 32 45.5)" />
</svg>
