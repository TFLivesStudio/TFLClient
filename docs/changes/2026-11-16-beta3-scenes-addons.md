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

El tag `v0.15.4-beta.3` apunta a `d69122c9a0574170efa733cf3f211a6fea231ea0`.
La [CI de ese commit](https://github.com/TFLivesStudio/TFLClient/actions/runs/44013565754)
pasó íntegramente antes de publicar el tag. La evidencia local generada vive en
`artifacts/`, ignorado por Git; CI conserva además su artefacto `beta-ui-review`.

- 56 tests unitarios Node pasaron, incluidos migración, política, movimiento,
  continuidad geométrica y correspondencia de texels entre Canvas y SVG.
- `bun run check`: cero errores; cinco advertencias previas de referencias
  iniciales en ContextMenu, InstanceDetail y ContentExplorerView.
- Build de producción del frontend completado. Los avisos de tamaño de chunks
  no bloquean la compilación.
- Prettier y ESLint completados. Rust: fmt, clippy, 270 tests de librería,
  aceptación nativa MCP con perfil aislado y auditoría pasaron en CI Linux.
  Los tests incluyen 18 casos del instalador de addons y sus entradas hostiles.
- Matriz de interfaz compilada con IPC simulado: 29 capturas/casos, contraste
  de tokens sobre superficies estables, límites de ventana y escala lógica.
- Addons con IPC simulado: cuatro variantes Pixel/Modern × claro/oscuro, desde
  inspección y consentimiento hasta actualización, recuperación y eliminación.
- Dos casos adicionales aplican el tema completo del ejemplo en Pixel claro y
  oscuro. Miden texto compuesto y mensajes reales de error con mínimo 4.5:1,
  rechazan metadatos ilegibles y comprueban la restauración del anfitrión.
  Cambiar Modern/Pixel, tipografía o material conserva la paleta seleccionada.
- La captura de pausa ocurre antes del desmontaje de Canvas. Se comprueba
  igualdad del último PNG, reanudación y liberación de renderizadores.
- La primera CI de preparación pasó Rust, clippy, auditoría, aceptación MCP y
  270 tests de librería (incluidos 18 del instalador de addons). Su fallo visual
  de pausa se reprodujo y corrigió antes de etiquetar la versión.
- Raster de Chromium: continuidad al cruzar el periodo en los tres paisajes y
  liberación de recursos. No equivale a certificar GPU o WebView nativos.
- Regresiones: 116 comprobaciones Pixel, 22 MCP/escenas, 11 de fondos, 216
  observaciones de sidebar, cinco escenarios del personaje y ocho casos de
  TFL Selection. La batería completa del workflow también quedó en verde.
- Comparación antes/después: ocho capturas de los builds reales de Beta 2 y
  Beta 3, Pixel/Modern × claro/oscuro, misma escena estática y ventana 1100×700.
  Es evidencia de navegador con IPC simulado, no una captura nativa.

La CI detectó además un fixture antiguo de fondos que guardaba Beta 2 como
última versión vista: abría el modal de novedades y pausaba correctamente el
paisaje. La prueba ahora obtiene la versión de package.json, sin relajar la
política de pausa del producto.

Entorno local: Windows, cuatro procesadores lógicos y aproximadamente 7.45 GiB
de RAM. Rust 1.94.1 fijado por el proyecto; Node 26.13.0 y Bun 1.4.1 locales.
CI usa las versiones de Node y Bun declaradas en el workflow. DPI del navegador
es emulación; no acredita el panel de escalado de Windows, otras GPU, una cuenta
Microsoft real ni una partida de Minecraft. No se inventan métricas de FPS o
compatibilidad universal.

El enlazado de la build nativa local se canceló al provocar presión de memoria;
no se contabiliza como prueba superada. Los resultados nativos de CI se
distinguen de la revisión del frontend con IPC simulado.

Se ejecutó una copia verificada de Beta 2 en Windows con `TFL_DATA_DIR` y
`WEBVIEW2_USER_DATA_FOLDER` separados. Se observó el Inicio nativo mediante
accesibilidad con el usuario offline de prueba. Las capturas fallaron con
`FrameArrived timed out`/`window capture timed out`; la automatización posterior
se interrumpió por ventana minimizada y detección de entrada de usuario. Esto
no acredita una revisión visual nativa ni el ciclo completo de actualización.
El perfil se preparó en `settings/settings.tfl`, que contiene JSON.

Después de verificar la firma del instalador real de Beta 3 se extrajo su
ejecutable, sin instalarlo sobre la aplicación habitual. FileVersion y
ProductVersion indican `0.15.4-beta.3`; el proceso abrió una ventana y respondió.
Con un perfil de datos y WebView separado se observó mediante accesibilidad el
panel nativo Ajustes → Addons: selección local, inspección, perfil seguro e
Instalados (0). Se guardó `artifacts/beta3-updater/native-beta3-accessibility.txt`.
Las capturas nativas volvieron a fallar con `FrameArrived timed out`. No se
certifica el ciclo nativo de instalar un addon, jugar ni aplicar una actualización.

## Publicación y actualizador

[Build TFL Client](https://github.com/TFLivesStudio/TFLClient/actions/runs/44014768149)
terminó en verde para Windows x86_64, Linux x86_64 y macOS aarch64, incluidas sus
compuertas de calidad. Solo se lanzó una ejecución de instaladores sobre el tag
validado. La build auxiliar MP Guard también pasó.

La [release v0.15.4-beta.3](https://github.com/TFLivesStudio/TFLClient/releases/tag/v0.15.4-beta.3)
se publicó el **2026-11-17T04:22:46Z**, como prerelease y sin borrador. Se
comprobaron 17 assets. El canal estable continúa en `v0.15.2`.

- `latest.json` anuncia `0.15.4-beta.3` con ocho entradas de plataforma y cinco
  paquetes distintos: aplicación macOS, AppImage, DEB, RPM y NSIS de Windows.
- Se descargaron los cinco paquetes y sus firmas. Cada entrada pasó verificación
  con la clave pública del tag y `minisign-verify 0.2.5`, la misma biblioteca del
  updater. También se rechazó alterar el contenido o el comentario firmado.
- Las firmas sueltas coinciden con las del manifest. Los tamaños y SHA-256
  coinciden con los digests publicados por GitHub. Los nombres de descarga fijos
  apuntan a bytes con el mismo digest que los archivos versionados.
- El cuerpo publicado coincide con el Markdown revisado. Se verificaron en un
  navegador real la tarjeta de Novedades y el artículo de Beta 3, ambos HTTP 200,
  con resumen y notas de Addons presentes y sin errores de página.

Evidencia local: `artifacts/beta3-updater/release-verification.json`,
`metadata-verification.json`, `github-release.json` y
`web-browser-verification.json`. El verificador temporal usa únicamente la clave
pública; no firma ni modifica paquetes, no requiere secretos y no acredita
Authenticode, notarización ni una instalación completa del updater.

Para recibirla desde el launcher: Ajustes → Actualizaciones, activar
«Versiones beta» y pulsar «Buscar actualizaciones». La selección de canal y los
bytes firmados se conservaron; no se amplió el canal estable.

## Herramientas y mantenimiento

Vite excluye el directorio `target` del workspace para evitar EBUSY al vigilar
ejecutables de Cargo en Windows. Prettier respeta los finales de línea del
checkout, evitando reescribir archivos históricos. CI incorpora las tres pruebas
nuevas de Beta 3 sin ampliar permisos, secretos ni políticas de publicación.
