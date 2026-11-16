# TFL Client 0.15.4-beta.3: escenas, Pixel y Addons

## Base y alcance

Base comprobada: `main` en `99e704c4f1e5eacfdad79261f99427361bc08413`,
sin diferencias con la referencia del encargo al iniciar. Se conserva el código
de Beta 2 y sus datos. La versión se alinea en package.json, tauri.conf.json,
Cargo.toml y Cargo.lock mediante el script del proyecto.

La publicación fue autorizada expresamente al retomar el trabajo el 16 de
noviembre. Debe completar las compuertas existentes antes del tag; no se omiten
pruebas para forzar una release. Las notas públicas están en
[v0.15.4-beta.3.md](../releases/v0.15.4-beta.3.md).

## Escenas y arte

- Se retira el parallax del puntero en escenarios, vistas previas y el banner
  del Hero. El seguimiento del personaje conserva su comportamiento propio.
- Las capas viajan hacia la derecha en píxeles lógicos por segundo. Cada fase
  está acotada al periodo; el delta se limita al reanudar. Se conservan los
  límites de FPS, pausas de foco/visibilidad/modal/juego, Lite y las dos fuentes
  de reducción de movimiento.
- Geometría periódica de 768 píxeles, sprites envueltos y cobertura por repetición.
  La conversión de preferencias conserva el valor de profundidad anterior.
- Canvas y SVG comparten siluetas y cuantización del cielo. El dithering queda
  restringido al último 30% de cada transición de paleta para reducir ruido.
- Los pósteres originales son una opción estática explícita. Los vídeos propios
  e importados mantienen su reproducción sin traslación que revele bordes; no se
  ejecutan a la vez renderizadores animados ocultos detrás del vídeo.

Contratos, autoría original y licencia: [SCENE_MOTION.md](../experience/SCENE_MOTION.md).

## Pixel y modo claro

Se retira el velo global de LivingScene. Las superficies de lectura y navegación
se mantienen estables frente a paisajes o vídeos variables. Inicio, Biblioteca,
Explorar y el área técnica del detalle emplean tratamientos locales; la escena
conserva su color entre bloques. Se refuerzan los tokens de texto secundario,
metadatos y estados en claro, y el foco usa un color con contraste propio.

Pixel conserva tipografía de títulos, acento TFL y texto de lectura. Se reducen
rejilla decorativa, textura de navegación, sombras y biseles repetidos; búsquedas
compuestas mantienen un único marco. Modern conserva su geometría.

## Addons declarativos

Se añade Ajustes → Addons, separado de plugins de servidores Minecraft. Incluye
inspección local, consentimiento, instalación desactivada, activación, actualización
confirmada, rollback, desinstalación, recuperación de paquetes dañados y modo
seguro. Los temas y escenas se integran con los selectores existentes. Desactivar
una contribución seleccionada restaura un valor integrado.

El instalador Rust comprueba esquema, SemVer, compatibilidad, IDs, permisos vacíos,
rutas, entradas ZIP, límites, ratios y PNG con CRC y decodificación. El hash de la
inspección se vuelve a comprobar al instalar. Generaciones inmutables y un
registro atómico conservan la versión anterior ante fallo; la recuperación activa
modo seguro. La limpieza exige una frontera absoluta canonicalizada y no sigue
enlaces. Los recursos de una escena se validan y entregan en un único lote.

No se introduce ejecución de código de terceros, WebViews de Addons, CSS libre,
acceso a archivos arbitrarios ni ampliación de CSP o asset protocol. Los comandos
permanecen en el anfitrión y el ACL excluye la ventana de log. El nombre del autor
y SHA-256 no acreditan identidad ni firma de editor. No hay marketplace.

Formato, ejemplo instalable y arquitectura futura:
[Addons](../addons/README.md), [seguridad y SDK](../addons/security-and-sdk.md),
[orange-dunes.tfl](../../examples/addons/orange-dunes.tfl).

## Comprobaciones

Los resultados finales de CI y publicación se registrarán al completar el tag.
La evidencia local generada vive en `artifacts/`, ignorado por Git.

- 54 tests unitarios Node pasaron, incluidos migración, política, movimiento,
  continuidad geométrica y correspondencia de texels entre Canvas y SVG.
- `bun run check`: cero errores; cinco advertencias previas de referencias
  iniciales en ContextMenu, InstanceDetail y ContentExplorerView.
- Build de producción del frontend completado. Los avisos de tamaño de chunks
  no bloquean la compilación.
- Matriz de interfaz compilada con IPC simulado: 25 capturas/casos, contraste
  de tokens sobre superficies estables, límites de ventana y escala lógica.
- Addons con IPC simulado: cuatro variantes Pixel/Modern × claro/oscuro, desde
  inspección y consentimiento hasta actualización, recuperación y eliminación.
- Raster de Chromium: continuidad al cruzar el periodo en los tres paisajes y
  liberación de recursos. No equivale a certificar GPU o WebView nativos.

Entorno local: Windows, cuatro procesadores lógicos y aproximadamente 7.45 GiB
de RAM. Rust 1.94.1 fijado por el proyecto; Node 26.13.0 y Bun 1.4.1 locales.
CI usa las versiones de Node y Bun declaradas en el workflow. DPI del navegador
es emulación; no acredita el panel de escalado de Windows, otras GPU, una cuenta
Microsoft real ni una partida de Minecraft. No se inventan métricas de FPS o
compatibilidad universal.

## Herramientas y mantenimiento

Vite excluye el directorio `target` del workspace para evitar EBUSY al vigilar
ejecutables de Cargo en Windows. Prettier respeta los finales de línea del
checkout, evitando reescribir archivos históricos. CI incorpora las tres pruebas
nuevas de Beta 3 sin ampliar permisos, secretos ni políticas de publicación.
