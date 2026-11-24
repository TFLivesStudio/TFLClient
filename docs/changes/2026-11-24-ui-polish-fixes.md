# UI polish: carpeta de perfil, detección de GPU y visor de capturas

Estado de trabajo: implementación incorporada y validada localmente (tests,
reproducción visual en vivo). No certifica una release ni verificación nativa
en Windows/macOS/Linux reales — eso corresponde a build de prueba.

## Reporte original

Cuatro items reportados juntos, con capturas:

1. Carpeta de datos del perfil se llama `tflclient` (todo minúscula); debería
   ser `TFLClient` por defecto.
2. Detección de hardware en el panel "Estado del sistema" — GPU mostraba "No
   disponible" en una sesión Windows real (CPU con iGPU real, AMD A10-8750B R7).
3. Etiquetas de instancia posiblemente ilegibles en modo claro (capturas
   mostraban varios "stat-chip" en blanco junto a un badge de loader visible).
4. Vista de captura en tamaño completo no se ajusta correctamente según el
   estado (desplegado/colapsado) del panel de instancias.

## 1. Carpeta de perfil `.tflclient` → `.TFLClient`

`src-tauri/src/core/path_manager.rs`. Windows y macOS (sistema de archivos
insensible a mayúsculas) resuelven ambos nombres a la misma carpeta real, así
que ahí alcanza con usar el string nuevo — es exactamente donde se veía el
nombre viejo (panel de Estado del sistema, que solo imprime el `PathBuf` que
arma el código, no consulta el nombre real guardado en disco). Linux (sensible
a mayúsculas) tiene carpetas de verdad distintas: si existe la vieja y no la
nueva, se renombra en el lugar (operación de metadatos en el mismo volumen, no
una copia) antes de usarla, para no perder instancias/ajustes de nadie. Si el
renombrado falla (permisos, disco de solo lectura), se seguye usando el nombre
anterior en vez de perder acceso a los datos.

4 tests nuevos (`cargo test -p tflclient --lib core::path_manager`): instalación
limpia usa el nombre nuevo, migración real preserva archivos, migrar dos veces
es no-op, y el test preexistente de aislamiento de build de prueba (ajustado al
nombre nuevo).

No se tocó el identificador de bundle de Tauri (`com.tflives.tflclient`) ni la
carpeta `.tflclient-auth-tests` del carril de pruebas de auth — fuera de alcance
de este reporte y sensible para firma/actualización automática.

## 2. Detección de GPU en sesión remota (RDP)

`src-tauri/src/commands/system_status.rs`. `Win32_VideoController` vía CIM
informa el adaptador que está renderizando la sesión ACTUAL — en una sesión de
Escritorio Remoto eso suele ser el driver virtual de RDP, no la GPU física
instalada, incluso cuando la GPU real existe y tiene drivers correctos. Ahora
se filtran nombres conocidos como virtuales (`remote`, `rdp`, `basic render
driver`); si tras filtrar no queda ningún adaptador real, se consulta
`Get-PnpDevice -Class Display -PresentOnly`, que lista hardware de video
instalado independientemente de cuál esté activo en la sesión actual.

No se pudo probar contra una sesión RDP real (sin acceso a Windows desde este
entorno); la implementación sigue el patrón documentado para este problema
conocido de `Win32_VideoController` sobre RDP. 2 tests nuevos cubren el
filtrado de nombres virtuales y el parseo de JSON de PowerShell (objeto único
vs. array, BOM, entradas vacías) — lógica pura, sin depender de Windows.

## 3. Etiquetas de instancia en modo claro — no reproducido

Se armó un caso de prueba estático con el CSS real del proyecto
(`global.css` + `refresh.css`) bajo `html[data-visual='pixel'][data-theme='light']`,
y también la app real corriendo con datos de fixture: instancia Fabric con y
sin arte de fondo, con el panel de instancias expandido y colapsado. En los
cuatro casos el badge de loader y los 4 "stat-chip" (versión, versión de
loader, última vez jugado, tiempo jugado) se renderizan con contraste correcto.

No se encontró ni se tocó código para este punto — no se pudo confirmar la
causa sin una captura más específica (qué instancia exacta, versión de la app,
si el launcher estaba usando el estilo Pixel o Modern). Pendiente de una
reproducción más clara antes de tocar nada acá.

## 4. Visor de capturas en tamaño completo no cubre el panel de instancias

`src/lib/components/library/ScreenshotsPanel.svelte`. Causa real, confirmada
en vivo: `.main-content` (`AppShell.svelte`) tiene `position: relative` +
`z-index: var(--z-content)` (1), lo que arma su propio stacking context. El
Sidebar vive como hermano de `.main-content` con `z-index: calc(var(--z-content) + 1)`
(2). Como 2 > 1, el Sidebar entero pinta por encima de TODO el subárbol de
`.main-content` en la comparación del padre — sin importar qué z-index tenga
algo anidado adentro (el lightbox pedía `z-index: 400`, totalmente irrelevante
porque esa comparación nunca llega a ese nivel). El `.lightbox` SÍ ocupaba la
ventana completa en tamaño (`position:fixed; inset:0` se calculaba bien,
confirmado con `getBoundingClientRect`) — el problema era de orden de pintado,
no de geometría, y solo se notaba con el panel de instancias desplegado porque
colapsado es angosto y el recorte visual pasa casi desapercibido.

Mismo patrón que ya usa `Tooltip.svelte` para este problema exacto: el
`.lightbox` se saca del árbol al `<body>` apenas se monta (`use:portalToBody`),
donde su z-index sí compite directamente contra el del Sidebar y gana. Probado
en vivo con el panel expandido y colapsado, antes y después del fix — captura
cubre toda la ventana en ambos casos ahora.

## Validación

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`
  (solo warnings preexistentes), `cargo test --workspace` (251 tests, 0 fail),
  `cargo audit` (limpio — ver más abajo).
- `bun run check` (0 errores), `bun run lint`, `bun run build`.
- Reproducción visual real: `bun run dev` + un fixture de desarrollo temporal
  (instalado y completamente revertido antes de este commit, nunca llegó a
  `main`) para cargar la app sin backend Tauri y verificar los puntos 1, 3 y 4
  con datos e interacción reales en el navegador.

## Aparte: segundo lote de avisos de `cargo audit`

Independiente de lo anterior: `cargo audit` volvió a fallar por dos avisos
nuevos publicados el 20 y 23 de noviembre (`url` 2.5.8, `cookie` 0.18.2),
ninguno relacionado con este cambio. Igual que el lote anterior (PR #18):
`cargo update -p cookie --precise 0.18.3` y `cargo update -p url --precise
2.5.9` (ambas transitivas, sin tocar ningún `Cargo.toml`), workspace completo
recompila y testea verde, `cargo audit` vuelve a quedar en 0 vulnerabilidades.
