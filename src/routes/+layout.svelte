<script lang="ts">
	import '../styles/global.css';

	let { children } = $props();

	// El launcher no usa el menú nativo del navegador (Atrás/Refrescar/
	// Guardar como/Imprimir) en ningún lado — donde hace falta un click
	// derecho hay un menú propio (ver Sidebar.svelte) que ya hace su
	// propio preventDefault. En el resto de la app, bloquear acá para que
	// no aparezca ese menú "de sitio web" y rompa la estética nativa.
	function blockNativeContextMenu(e: MouseEvent) {
		e.preventDefault();
	}
</script>

<svelte:window oncontextmenu={blockNativeContextMenu} />

{@render children()}
