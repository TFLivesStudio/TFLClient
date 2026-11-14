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

Publicación y resultados finales: pendientes de completar antes de etiquetar.
