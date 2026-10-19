<script lang="ts">
	import '../styles/global.css';

	let { children } = $props();

	// El launcher no usa el menú nativo del navegador (Atrás/Refrescar/
	// Guardar como/Imprimir) en ningún lado — donde hace falta un click
	// derecho hay un menú propio (ver Sidebar.svelte) que ya hace su
	// propio preventDefault. En el resto de la app, bloquear acá para que
	// no aparezca ese menú "de sitio web" y rompa la estética nativa.
	//
	// Excepción: texto que el propio CSS ya marca como seleccionable
	// (inputs, el código de inicio de sesión de Microsoft, etc. — ver
	// `user-select: text` en global.css y en los componentes que lo
	// necesitan). Bloquear el menú nativo ahí también dejaba sin forma de
	// copiar ese texto con el click derecho — bug real reportado: "el
	// código para iniciar sesión no deja copiar ni pegar".
	function blockNativeContextMenu(e: MouseEvent) {
		const target = e.target as HTMLElement | null;
		if (target && getComputedStyle(target).userSelect !== 'none') return;
		e.preventDefault();
	}
</script>

<svelte:window oncontextmenu={blockNativeContextMenu} />

{@render children()}
