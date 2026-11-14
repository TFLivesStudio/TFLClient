# Pixel Overhaul y Living Themes

## Base integrada

Última beta consultada: `v0.15.3-beta.7`, commit
`fa96aafb418786c3e249bd96bd3e25266cb58871`. `main` apuntaba a
`59bac84ef076dd76fb9df804d11f0077cbca524d`: 20 commits exclusivos de la
beta y cinco de main. La rama `feat/pixel-living-themes` parte de la beta;
el merge `7dc7f25da364ebd412ab5eedc563e98f5e3387f3` integra main sin
reescribir historia. La revisión del merge confirma que los servicios de
lanzamiento y ajustes solo reciben comentarios/nomenclatura; conserva Auth,
almacenamiento de credenciales y la selección de Java/macOS de beta.7.

## Contrato Pixel

La cascada anterior aplicaba bordes al input y al contenedor de búsquedas.
Biblioteca, paleta y modpacks no estaban incluidos en todas las excepciones.
`data-search-control` declara el propietario del marco; los inputs interiores
quedan fuera de la regla genérica. El foco pertenece al contenedor. Las
variantes locales de controles usan el contrato Pixel sin alterar filtros,
atajos o lógica de instalación. Los tokens de acción derivan del acento y su
primer plano contrastado. Se conservan Readable, Classic y Modern.

## Escenarios

Se añaden `midnight-forest`, `dithered-sunset` y `aurora-valley` sin cambiar
los IDs existentes ni confundir el preset `pixel` con el estilo de interfaz.
`landscape.ts` contiene paletas, geometría determinista, Bayer 4×4 y posters
SVG. `landscapeRenderer.ts` almacena cielo y capas estáticas en cachés Canvas
de tamaño fijo; solo compone las capas y dibuja nubes, auroras, bruma,
reflejos, partículas y clima por fotograma. `PixelLandscape.svelte` comparte
`frameClock`, usa escalado entero y libera cachés/listeners al desmontar.
Los textos y controles conservan la resolución nativa.

Lite y movimiento desactivado usan SVG sin contextos Canvas/WebGL. Balanced
limita a 24 FPS; Experience permite hasta 60. La configuración y las políticas
de visibilidad, foco, overlays y juego tienen prioridad sobre esos máximos.
La pérdida de contexto muestra el poster y la restauración reconstruye las
cachés. No se introduce GPUI, Macroquad, paquetes gráficos ni acceso remoto.

## Preferencias y seguridad

El esquema `tfl-scenes-v1` mantiene su versión: la migración es aditiva mediante
defaults explícitos para `scope`, `parallax` y `dithering`. Los valores anteriores
conservan Hero como destino. Las preferencias globales y por UUID se validan en
el parser existente. La galería conserva borrador y Aplicar/Cancelar. El shell
posee una sola suscripción de almacenamiento; Hero y ventana eligen un solo
propietario del escenario.

No se amplían CSP, protocolo de archivos, ACL ni comandos Tauri. No se cambian
datos de Minecraft, OAuth ni firmas. Cargo actualiza `quinn-udp` de 0.5.15 a
0.5.16 para corregir RUSTSEC-2026-0538; también normaliza referencias transitivas
Windows del lockfile al resolverlo. Las excepciones de auditoría existentes no
se amplían.

## Referencias y licencias

Se estudiaron `scenery.rs` de Retro Realm en
`refactor/rust-macroquad-migration` y `sky.rs` de Horizon como referencias
técnicas. La geometría, paletas y renderer son originales; no se redistribuyen
código, sprites, fotografías ni otros recursos de esos proyectos. TFL Client
conserva GPL-3.0-only y los avisos de sus recursos anteriores.

## Validación y publicación

La base integrada pasó 25 pruebas unitarias, Svelte con cero errores y cinco
warnings previos, build de producción y smoke con 26 capturas. La auditoría
local posterior a la actualización de quinn-udp informa cero vulnerabilidades
sin exceptuar. Las nuevas suites son `pixel-controls.mjs`,
`living-settings.mjs` y `living-landscapes.mjs`; CI las ejecuta junto con las
regresiones anteriores. La evidencia se genera bajo `artifacts/`.

Las capturas utilizan Chromium y el fixture IPC aislado. No certifican inicio
de sesión real, una partida de Minecraft, códecs de cada OS ni consumo nativo.
El host Windows no dispone de los prerrequisitos MSVC de Tauri; los checks Rust
y los instaladores se verifican en el pipeline oficial. El rendimiento de
Chromium se registra separado de CPU/RAM nativas.

## Resultados del candidato 0.15.4-beta.1

- `bun install --frozen-lockfile`: correcto con Bun 1.4.2 y Node 24.
- Svelte: cero errores, cinco warnings previos. Lint completo correcto;
  comprobación adicional de los archivos finales sin errores.
- Build de producción: código de salida 0. Unitarias Node: 31/31.
- `cargo fmt --all -- --check`: correcto. La prueba Rust local no pudo enlazar
  por falta de `link.exe`; CI Linux sí pasó clippy, tests, MCP y audit del primer
  candidato `1634e89`. El commit definitivo vuelve a verificarse antes del tag.
- Smoke: 26 capturas; Experience: 85 entradas de evidencia/matriz completa;
  workspaces, lifecycle, 22 comprobaciones MCP/escenas, 13 de autenticación y
  beta2 con 12 capturas: correctos.
- Pixel: 116 comprobaciones de estilos/foco/geometría, 18 de acento y 18 de
  etiquetas Classic. Se probaron Readable/Classic, Dark/Light/OLED, tres
  calidades y materiales, cinco resoluciones entre 800×600 y 1920×1080,
  DPR 1/2, inglés, forced colors y movimiento reducido.
- Living: tres escenarios por tres calidades, cambio real de píxeles, parallax,
  cachés estables, contexto perdido/restaurado, fallback sin Canvas y doce
  ciclos de montaje con cero RAF al retirar el último escenario. Un vídeo
  empaquetado realmente reproduce sin Canvas redundante y el error de vídeo
  recupera el paisaje. Ajustes/overrides/Aplicar/Cancelar: correctos.

La primera CI de la base confirmó RUSTSEC-2026-0538; se corrigió la dependencia.
La primera CI de la función detectó un selector incompleto en el test de la
acción primaria; el harness ahora incluye la variante compartida `.tfl-btn`,
sin relajar la aserción de color. Forced colors se prueba por alpha calculado,
porque el RGB de un color totalmente transparente depende del navegador.

## Rendimiento observado

Tres muestras por perfil y revisión, Chromium 159 con IPC simulado y escenario
predeterminado estático; medias, en la misma máquina:

| Perfil     | UI usable base → cambio (ms) | Heap JS base → cambio (MiB) | Tareas en reposo base → cambio (ms/s) |
| ---------- | ---------------------------- | --------------------------- | ------------------------------------- |
| Lite       | 213.0 → 198.7                | 4.75 → 4.76                 | 0.45 → 0.54                           |
| Balanced   | 185.3 → 186.0                | 4.74 → 4.76                 | 0.99 → 1.04                           |
| Experience | 184.0 → 189.7                | 4.77 → 4.75                 | 1.21 → 1.17                           |

Abrir Ajustes promedió 48.3/59.3/59.3 ms antes y 47.3/60.0/61.0 después.
El detalle promedió 66.3/122.3/120.7 ms antes y 61.0/138.3/126.7 después:
la muestra Balanced incluye una observación de 171 ms. Son muestras pequeñas,
sin garantía estadística ni equivalencia a rendimiento nativo.
Las escenas nuevas consumieron aproximadamente 38–78 ms de tareas Chromium
por segundo en la observación registrada, con ocho superficies 2D por Forest
y Sunset y nueve por Aurora (incluidas cachés); Lite no crea contextos. Los
costes de inicio/caché y dibujo medio pueden medirse optativamente con la
instrumentación descrita en `docs/RENDIMIENTO.md`, sin IPC por fotograma.

## Evidencia, versión y límites

[Capturas antes/después y resultados](../qa/pixel-living-themes/README.md).
El código de implementación se inicia en `1634e89`; la versión que incorpora
la revisión final es `0.15.4-beta.1`, cuyo
[commit etiquetado](https://github.com/TFLivesStudio/TFLClient/commit/v0.15.4-beta.1)
se comprueba contra CI antes de publicar mediante el workflow oficial.
La publicación real y su fecha UTC se registran en el índice después de
verificar instaladores, firmas y `latest.json`. La estable de referencia es
`v0.15.2`, release ID `433706720`; no se reescriben su tag ni sus assets.

No se certificaron una cuenta Microsoft real, partidas, lectores de pantalla,
GPU física, autonomía ni todas las combinaciones de controladores y códecs.
Las pruebas nativas de OAuth en CI usan fixtures sin cuenta real ni bóveda del
sistema. No se desactiva ninguna compuerta para sustituir estas limitaciones.
