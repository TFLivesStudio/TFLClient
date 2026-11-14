# Sidebar y Pixel 2.0

La barra conserva un solo componente y una sola fuente de estado para expandida
y compacta. `viewport.svelte.ts` sigue definiendo los anchos 232/72 px y el umbral
de colapso automático de 960 px. Redimensionar no modifica la preferencia manual.
`toggleCollapsed` conserva el guardado optimista y la reversión ante error.

## Borde y estructura

`sidebar-shell` posee el ancho reactivo y el sistema de coordenadas de la pestaña.
La pestaña es una capa hermana del `aside`: su centro está en el borde derecho y
en la mitad de la altura útil, con un área de interacción de 32×40 px. El `aside`
mantiene su overflow limitado y la lista conserva desplazamiento propio; el footer
de usuario permanece fuera de la lista. El contenido del workspace no tapa el
hitbox de la pestaña. La navegación mantiene la misma coordenada vertical en
ambos estados.

La geometría se investigó en los componentes
[Sidebar](https://github.com/CubicLauncherDevs/CubicLauncher/blob/develop/src/lib/components/layout/Sidebar/Sidebar.svelte)
y [SidebarCompact](https://github.com/CubicLauncherDevs/CubicLauncher/blob/develop/src/lib/components/layout/Sidebar/SidebarCompact.svelte)
de CubicLauncher. Se utiliza como referencia de posición; la implementación,
composición, dimensiones, tokens e identidad son propios de TFL Client.

Por debajo de 960 px la pestaña sigue visible y enfocada por teclado, explica que
debe ampliarse la ventana y expone `aria-disabled`. Su acción no escribe una
preferencia que no tendría efecto visual. En anchos mayores, clic o Espacio
alternan el estado y `aria-expanded` describe el resultado.

## Composición compacta

La identidad TFL de la barra de título mantiene el ancho del riel. Navegación,
acciones, instancias y cuenta ocupan grupos separados con espaciado de 4 px.
Los iconos de navegación y las instancias se centran en el mismo eje. El acceso
a la skin utiliza el avatar y evita duplicar la acción en el footer compacto;
Ajustes y Salir siguen disponibles. Los nombres largos conservan su nombre
accesible completo y su tooltip.

Las instancias oficiales utilizan `selection-identity.ts`, sin deducir identidad
por loader. Su metadato presenta release/canal y su fallback visual es el icono
empaquetado; las normales conservan nombre, icono y loader propios.

## Tooltips y accesibilidad

El tooltip compartido se monta en `document.body` con posición fija únicamente
mientras está abierto. Su ancla sigue al trigger en resize y scroll, y la burbuja
se limita al viewport. `aria-describedby` relaciona la burbuja con el control;
Escape cierra y se restaura la descripción anterior al desmontar. ResizeObserver
y listeners se liberan al cerrar. La lista conserva su clipping sin recortar
las ayudas. No hay RAF continuo de posicionamiento.

## Pruebas

`tests/visual/sidebar-refinement.mjs` utiliza el frontend real con IPC simulado.
Recorre Modern, Pixel Readable y Pixel Classic; tres temas y tres calidades;
los cinco tamaños soportados, DPR 1/2 y español/inglés. Comprueba geometría y
hitbox del borde, ciclos de clic y teclado, preferencias durante resize,
alineación compacta, listas largas, footer, overlays y tooltip portalled.
Las latencias registradas son observaciones Chromium, no tiempos nativos.

`pixel-controls.mjs` conserva su cobertura de foco de 2 px, input sin segundo
marco y colores calculados, con la nueva expectativa de marco exterior de 1 px.
