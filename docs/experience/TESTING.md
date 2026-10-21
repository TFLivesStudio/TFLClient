# Verificación y límites

Implementación: `0.14.0-beta.1`, sobre la beta integrada `v0.13.1-beta.4` y con main integrado hasta `bf049c3` (incluye la corrección de Detener que prepara 0.13.2). Sin tag ni release.

## Comprobaciones ejecutadas

| Comprobación                                      | Resultado local                                                                                                                                                       |
| ------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `bun install --frozen-lockfile`                   | Correcto                                                                                                                                                              |
| `bun run check`                                   | 0 errores; 7 avisos conocidos en componentes existentes                                                                                                               |
| `bun run lint`                                    | Correcto; Prettier y ESLint                                                                                                                                           |
| `bun run build`                                   | Build de producción correcto                                                                                                                                          |
| `bun run release:version --check`                 | Cuatro versiones y notas coinciden                                                                                                                                    |
| `node --test tests/unit/*.test.mjs`               | 18 pruebas correctas                                                                                                                                                  |
| `cargo fmt --all -- --check`                      | Correcto                                                                                                                                                              |
| `cargo clippy --workspace --all-targets --locked` | Correcto; avisos previos no bloqueantes                                                                                                                               |
| `cargo test --workspace --lib --locked`           | 141 pruebas correctas                                                                                                                                                 |
| `tests/visual/beta2.mjs`                          | Correcto: consola histórica >100% CPU, niveles, orden con puntero, sincronización entre ventanas, parada de muestreo, exploración/paginación                          |
| `tests/visual/smoke.mjs`                          | Correcto: navegación, diálogo/foco, notificaciones, Modern/Pixel, vistas y tamaños                                                                                    |
| `tests/visual/workspaces.mjs`                     | Correcto: backups, contenido, servidor, ajustes y estudio a 800×600                                                                                                   |
| `tests/visual/lifecycle.mjs`                      | Correcto: Lite sin import/contexto WebGL automático, activación manual y limpieza/recreación tras overlay                                                             |
| `tests/visual/experience.mjs`                     | Correcto: destinos canónicos, búsqueda/tildes/retorno, validación, borrador/undo/cancel/apply, iconos, doble clic, selección explícita de versión, vídeo e instancias |

Las pruebas Rust nuevas usan archivos reales en un directorio temporal para importación/copia, listado, formato, límite de tamaño con archivo disperso, cupo de biblioteca y eliminación restringida por ID. Las pruebas ACL reales de Tauri comprueban que la ventana de log no obtiene los comandos de medios. El sampler conserva su prueba contra un proceso real; no se sustituye por valores demo.

## Matriz visual

Se comprobaron **2 estilos × 3 temas × 3 perfiles × 5 tamaños**: 90 combinaciones de ventana. En cada combinación se verificaron Apariencia, Fondos y escenas y Consola: 270 comprobaciones de desbordamiento. Los tamaños son 800×600, 1100×700, 1280×720, 1366×768 y 1920×1080; los temas, Dark/Light/OLED; los perfiles, Lite/Balanced/Experience.

La suite conserva más de 75 capturas nuevas y registra los escenarios funcionales en [ui-matrix.json](metrics/ui-matrix.json). Se revisaron visualmente muestras de claro, OLED/Pixel/Lite compacto, editor y escena por instancia. Lite verifica cero contextos WebGL automáticos y superficie de diálogo opaca sin blur. Los inputs Pixel se comprueban sin un marco interior adicional.

La búsqueda se evalúa mediante tareas de principiante/intermedio/experto (idioma, color, vídeo, memoria, cuentas, Java/JVM/entorno, núcleos y actualizaciones), sin afirmar que se realizó un estudio con personas. Cada descriptor del catálogo debe tener exactamente un grupo real en Ajustes. También se conserva la selección por teclado, foco dentro del diálogo y regreso a los resultados.

La suite de vídeo prueba bucles silenciados, pausa por overlay/foco/juego, persistencia, reproducción limitada por calidad/accesibilidad, importación mediante IPC simulado y fallback ante archivo desaparecido. La importación nativa en disco se cubre aparte con Rust. Los tests no instalan códecs ni descargan fondos.

## Mediciones antes/después

Se conservaron dos builds de producción separados: beta.4 y rework. Chromium **154.0.8037.57**, misma fixture de IPC, viewport 1280×720. No hubo compilaciones ni otros tests de navegador durante las mediciones.

Home sin skin premium, tres pasadas por perfil; tabla de medianas. «Trabajo» es TaskDuration de la pestaña durante aproximadamente un segundo, no porcentaje de CPU del launcher ni del sistema. [Datos completos](metrics/home.json).

| Perfil     | Home listo: beta.4 → rework | Heap JS: beta.4 → rework | Trabajo inactivo: beta.4 → rework |
| ---------- | --------------------------: | -----------------------: | --------------------------------: |
| Lite       |                386 → 370 ms |          5.21 → 4.58 MiB |                    0.69 → 0.90 ms |
| Balanced   |                317 → 342 ms |          5.22 → 4.57 MiB |                    2.48 → 2.56 ms |
| Experience |                362 → 319 ms |          5.24 → 4.56 MiB |                    2.72 → 2.47 ms |

Los cambios pequeños, incluida la subida de 25 ms en Balanced, están dentro de una muestra corta y ruidosa. No se presenta una mejora universal ni una reducción confirmada de memoria: la recolección de basura afecta el heap. La apertura de Ajustes quedó entre 50 y 99 ms en las tres pasadas del rework; el detalle, entre 93 y 153 ms.

Visor con textura sintética y **SwiftShader**, una observación por perfil de aproximadamente un segundo. [Datos completos](metrics/viewer.json).

| Perfil     | Contextos WebGL beta.4 → rework | Draw calls beta.4 → rework | Trabajo de pestaña beta.4 → rework |
| ---------- | ------------------------------: | -------------------------: | ---------------------------------: |
| Lite       |                           0 → 0 |                      0 → 0 |                     1.85 → 1.64 ms |
| Balanced   |                           1 → 1 |                 1159 → 380 |                 617.32 → 267.16 ms |
| Experience |                           1 → 1 |                 1159 → 380 |                 621.73 → 312.81 ms |

El límite de render reduce las llamadas de dibujo observadas aproximadamente un 67%; el reloj central sigue recibiendo RAF del navegador, por lo que no se confunde número de RAF con renders. La cifra de trabajo del visor baja en este entorno software, pero **no equivale a consumo nativo de CPU/GPU, batería ni FPS en hardware físico**. No se añadió un motor 3D de escenas ni dependencias visuales nuevas.

## Reproducción

Con Bun 1.4.2, Node 24 y Rust fijado en el repositorio:

```sh
bun install --frozen-lockfile
bun run check
bun run lint
bun run build
node --test tests/unit/*.test.mjs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked
cargo test --workspace --lib --locked
bun run preview -- --host 127.0.0.1 --port 1433
```

En otra terminal, con Playwright y Chromium instalados fuera del proyecto:

```sh
export PLAYWRIGHT_MODULE=/ruta/a/playwright/index.mjs
export CHROMIUM_PATH=/ruta/a/chromium
export VISUAL_URL=http://127.0.0.1:1433
export VISUAL_OUTPUT=artifacts/experience/regression
node tests/visual/beta2.mjs
node tests/visual/smoke.mjs
node tests/visual/workspaces.mjs
node tests/visual/lifecycle.mjs
node tests/visual/experience.mjs
```

Para comparar rendimiento, servir también el build original y establecer `BASELINE_URL` y `BASELINE_VERSION=0.13.1-beta.4`. Ejecutar `performance.mjs` y `viewer-performance.mjs` sin otras compilaciones/tests concurrentes. Los scripts guardan sus resultados en `artifacts/experience/performance`. CI fija Node 24 para importar los modelos TypeScript en estas pruebas, ejecuta las 18 pruebas y las cinco suites de navegador, y conserva las capturas como artefacto.

## Verificación manual pendiente

- El workflow nuevo `review.yml` está preparado en la rama; GitHub lo mostrará para ejecución manual cuando esté disponible en la rama predeterminada. No se fusionó el PR para habilitarlo ni se generaron instaladores nativos en esta misión.
- Instalar los paquetes de revisión en Windows, macOS y Linux; probar WebView2/WKWebView/WebKitGTK con vídeos H.264 y WebM de cada sistema, archivo no decodificable y cambio de escala/DPR real.
- Confirmar pausa/reanudación al minimizar, cambiar foco y abrir Minecraft real, y medir CPU/GPU/energía con herramientas nativas.
- Iniciar sesión Microsoft con una cuenta real; abrir y detener Minecraft; probar argumentos/env por instancia y compatibilidad/dependencias reales con Modrinth.
- Comprobar skins/capas con GPU física, sobre todo la limitación 3D de macOS que el repositorio ya documenta.
- Validar instaladores, firma/notarización y actualización firmada cuando el responsable decida publicar. Esta misión no ejecuta tags ni releases.
- Medir tiempos nativos del perfil `review` frente a `release` en los tres runners antes de atribuir un ahorro a ThinLTO. El perfil oficial y el flujo de firma/updater permanecen intactos.

Mica/Acrylic/Vibrancy y una escena voxel 3D completa no se activan sin validación de plataforma. Las alternativas entregadas son materiales CSS y las escenas 2.5D originales; el shader de luz es opcional y tiene fallback a esas capas.
