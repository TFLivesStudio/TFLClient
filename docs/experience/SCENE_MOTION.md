# Movimiento de escenas — 0.15.4-beta.3

El paisaje se desplaza hacia la derecha con velocidades por plano. No escucha
el puntero. El Hero conserva su banner editorial fijo; el seguimiento del
personaje sigue siendo una interacción independiente.

## Contrato de tiempo y cobertura

`motion.ts` expresa la velocidad en píxeles lógicos por segundo:
`0.6 + profundidad × profundidadDelPlano × 5.4`. Profundidad cero conserva el
viaje de todos los planos a una velocidad común; no equivale a «Estático».
Cada plano mantiene una fase independiente, acotada a `[0, periodo)`. Los deltas
no finitos o negativos se descartan; el máximo es 100 ms. Un intervalo largo
de suspensión no se recupera acelerando la escena. El reloj compartido deja de
pedir frames cuando se retira su último consumidor.

Los paisajes Canvas se componen en 384×216. Sus cachés de terreno son franjas
de 768×216: perfiles periódicos que coinciden en altura y pendiente, árboles
que atraviesan ambos lados y dos copias adyacentes por frame. La posición se
redondea hacia abajo al píxel lógico; la fase conserva su fracción para evitar
deriva. El escalado CSS es entero y usa `image-rendering: pixelated`, con recorte
centrado. La franja repetible también cubre ventanas panorámicas: el factor
de escala toma el máximo necesario por ancho y alto, sin estirar el arte.
Cambios de FPS o restauración del contexto conservan las fases de los planos;
las cachés descartadas liberan sus backing stores.

Las siete escenas con capas SVG utilizan geometría periódica generada por
`layeredArt.ts`, con paletas y valores por profundidad. Su cielo es un recurso
vectorial local existente. `SceneFrame` repite las capas horizontalmente y
actualiza su posición desde el mismo reloj y contrato de fase; no oscila de
ida y vuelta ni traslada una fotografía para simular profundidad.

## Póster, vídeo y políticas

El selector «Arte original (estático)» conserva el acceso al póster
cinematográfico anterior. Los wallpapers y estos pósteres permanecen fijos:
su geometría plana no ofrece parallax. El selector de profundidad explica esa
limitación y se desactiva para esos materiales o para vídeo.

Los vídeos incluidos e importados se reproducen en su superficie completa,
con sus FPS codificados, sin transformaciones ni falsas capas. Al reproducirse
un vídeo se suspenden el terreno, shader, partículas y clima del fondo. Si
falla el medio, vuelve la representación del paisaje. La disponibilidad de
códecs continúa dependiendo del WebView y del sistema operativo.

`sceneCapabilities` conserva Lite/Balanced/Experience, movimiento del sistema,
«Detener efectos continuos», foco, visibilidad, overlays/vista previa y las
políticas de juego. Lite, movimiento desactivado y reducción muestran un fondo
estático coherente. Balanced limita los efectos a 24 FPS; Experience respeta el
máximo configurado hasta 60. Estas cifras son límites, no rendimiento medido.

## Compatibilidad y persistencia

La clave `tfl-scenes-v1`, el esquema v2 y el campo numérico `parallax` se
conservan. El control se llama «Profundidad automática»: el valor antiguo se
interpreta como separación de velocidades, sin sobrescribirlo ni volver a
crear preferencias por instancia. Guardar continúa pasando por `persistAuthority`
y por los eventos de almacenamiento de las ventanas. Vista previa/Cancelar
no aplican el borrador. `poster: "cinematic"` es una opción aditiva.

Los IDs `addon:<namespace.id>:<escena>` pueden persistirse, pero no son URLs ni
rutas. El catálogo de Addons habilitados resuelve sus recursos validados. Un
addon eliminado o desactivado restaura la escena predeterminada. Su renderer
está separado del código privilegiado y conserva los límites de movimiento.

## Autoría y licencia

La geometría y las fórmulas nuevas de `landscape.ts`, `layeredArt.ts` y `motion.ts`
son código original creado para TFL Client en esta entrega, bajo la licencia
GPL-3.0-only del repositorio. No se añaden sprites, fotos, texturas de Minecraft
ni imágenes de terceros. Se reutilizan los cielos SVG y pósteres ya versionados;
su procedencia y sus límites anteriores siguen en [ASSETS.md](ASSETS.md).
Este cambio no concede derechos nuevos sobre materiales existentes.

## Verificación reproducible

- `node --test tests/unit/scene-motion.test.mjs tests/unit/landscape.test.mjs tests/unit/living-preferences.test.mjs`
  comprueba dirección, velocidad, fase modular, pausa, deltas inválidos, periodos
  espaciales, cobertura de tamaños, ausencia de puntero y persistencia/políticas.
- `node tests/visual/scene-motion.mjs` importa el renderer de producción en
  Chromium y compara píxeles de terreno al cruzar 767.99→0: la imagen debe
  equivaler a viajar un texel hacia la derecha. Genera PNG de antes/después y
  `results.json` en `artifacts/beta3-motion`. Es un harness Canvas de navegador,
  no una ejecución nativa del launcher.
- `node tests/visual/landscapes.mjs` cubre escenas/calidades, cambio de píxeles,
  pérdida/restauración de contexto, cachés, desmontaje, vídeo y fallo de medios
  en el frontend completo con IPC simulado.

La matriz real de WebView, escalado de Windows y dispositivos sigue requiriendo
evidencia propia; estas pruebas no certifican GPU, consumo energético ni todos
los controladores. Las capturas históricas de betas anteriores no acreditan
el resultado de esta beta.
