# Validación — Activity Center v1

Base: `feat/activity-center` desde tag `v0.14.0-beta.6` / `47c0832`.
Worktree nuevo, sin cambios locales preexistentes; autenticación #9 y Content
Discovery #4 quedan fuera. Sin AGENTS.md aplicable. Inventario y plan:
[AUDIT.md](AUDIT.md); contratos y límites: [DESIGN.md](DESIGN.md).

## Flujo real

Instancia → Contenido → Descargar → instalar un mod/shader/resource pack/plugin →
**Actividades** en sidebar. Se crea antes de esperar el bloqueo por instancia, muestra
resolución y bytes reales cuando el servidor entrega tamaño; finaliza tras archivos
y dependencias. Cambiar de pantalla no elimina la tarea. Los detalles incluyen ID,
revisión y Modrinth ID; el historial conserva hasta 100 resultados.

La prueba nativa opt-in descarga **FerriteCore para Fabric 1.21.1** desde Modrinth,
verifica SHA1 mediante el pipeline real, instala el `.jar` en un perfil desechable,
comprueba `succeeded` e historial persistido y provoca un fallo real por instancia
inexistente. No inicia Minecraft ni usa una cuenta. La evidencia conserva solo campos
públicos y nombres de artefactos: [real-install.json](evidence/real-install.json).

También pasó el SDK MCP real con `MCP_NETWORK_TEST=1`: Fabric API instalado y
actividad con origen MCP e instancia correctos, sin cambiar políticas. Evidencia:
[mcp-native.json](evidence/mcp-native.json). La regresión basal pasó beta2, smoke,
workspaces, lifecycle, Experience (85 entradas) y MCP/scenes (22 comprobaciones).
El nuevo acceso global exigió conservar espacio de scroll en la sidebar: los
controles compactos son uniformes a 40 px en ventanas de hasta 660 px de alto;
siguen a 44 px en las demás. La prueba recorre la última instancia a 800×600.

```sh
# TFL_DATA_DIR debe ser una carpeta temporal absoluta, nunca el perfil habitual.
touch "$TFL_DATA_DIR/TEST_PROFILE"
cargo run -p tflclient --example activity_install --features mcp-integration-tests
```

`tests/mcp/client.mjs` conserva su prueba opcional `MCP_NETWORK_TEST=1`: instala
Fabric API por el SDK MCP real, comprueba permiso/scope previo, ID compartido,
origen `mcp`, resultado y archivo real. El host de pruebas escribe `activities.json`
solo en su perfil con marcador; no añade herramientas ni archivos de diagnóstico de
este tipo en producción.

## Comprobaciones

| Comando/recorrido                          | Resultado                                                                                                                 |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| Baseline `bun run check`                   | PASS: 0 errores, 7 avisos existentes                                                                                      |
| Baseline `cargo test --workspace --lib`    | PASS: 156 pruebas                                                                                                         |
| `cargo fmt --all --check`                  | PASS                                                                                                                      |
| `cargo clippy --workspace --all-targets`   | PASS con advertencias previas; no `-D warnings` global                                                                    |
| `cargo test --workspace --lib`             | PASS: 161 pruebas, incluye 5 nuevas de Activity Core y ACL actualizado                                                    |
| `node --test tests/unit/*.test.mjs`        | PASS: 21 pruebas, incluye subscribe/snapshot, stale revision, cleanup tardío y recuperación                               |
| `bun run check`                            | PASS: 0 errores, 7 avisos previos                                                                                         |
| `bun run lint`                             | PASS                                                                                                                      |
| `bun run build`                            | PASS                                                                                                                      |
| `activity_install` con red real            | PASS: descarga/archivo/hash, éxito/fallo e historial                                                                      |
| `tests/visual/activity.mjs`                | PASS: Modern/Pixel a 800×600 y 1280×720, 10 grupos de comprobaciones; IPC simulado explícito                              |
| Login real, lanzamiento de Minecraft y MFA | NO EJECUTADO: no forman parte de la operación piloto; no se usan credenciales del usuario                                 |
| Recorrido GUI nativo Windows/macOS/Linux   | NO EJECUTADO localmente: entorno Linux headless con WebKit desempaquetado; compilación/empaquetado se verifica en Actions |

Las pruebas de núcleo cubren unicidad concurrente (320 trabajos), límites 64/100,
transiciones/terminales/revisiones obsoletas, rechazo de cancelación sin cambiar el
estado, progreso inválido/medible/indeterminado, aborto/unwind, recuperación, JSON
corrupto/campos ajenos, identificadores y fallo de escritura sin perder memoria.
Las de navegador comprueban detalle por teclado, ausencia de cancelación ficticia,
un listener después de cinco navegaciones, resync/error, interrupción y aviso de
almacenamiento. No se afirma que IPC simulado certifique una sesión nativa completa.

## Validar manualmente en Windows, Linux o macOS

1. Activar canal beta y actualizar a beta.7.1; confirmar versión en Ajustes.
2. En una instancia de prueba detenida, instalar contenido compatible. Abrir
   Actividades desde sidebar mientras descarga. Ver tipo, instancia, origen Usuario,
   bytes del archivo actual o progreso indeterminado y resultado final.
3. Cambiar entre Biblioteca, instancia y Actividades: misma operación, no duplicados.
   Abrir detalles con Tab/Enter. Repetir Modern/Pixel, claro/oscuro/OLED y Lite.
4. Reiniciar tras finalizar: resultado presente. Cerrar abruptamente durante una
   instalación: la entrada persistida reaparece interrumpida, sin reanudarse.
5. Para error, cortar red durante descarga o elegir versión incompatible. La tarea
   debe fallar y el flujo original mostrar su error. No cancelar ni borrar archivos
   parcialmente instalados automáticamente.
6. Si se usa MCP, aprobar la sesión con el scope de instancia habitual y realizar
   `tfl_install_content`; Activity Center debe indicar MCP. Una sesión sin permiso
   no llega al servicio y no crea actividad ni instala archivos.

## Aceptación

La revisión final encontró que la barra flotante heredada podía conservar una descarga
tras un fallo. Se detuvo el workflow de beta.7 antes de publicar una release y se
preparó beta.7.1, sin reescribir tags. Barra flotante, personaje y Activity Center
consumen ahora la misma autoridad para la operación piloto. Una prueba de regresión
comprueba el progreso en Biblioteca, ausencia de duplicados en Actividades y retirada
de la barra al llegar un fallo backend.

| Criterio | Estado y evidencia                                                                               |
| -------- | ------------------------------------------------------------------------------------------------ |
| AC-01    | ✅ Autoridad única de dominio en `services/activity_tasks.rs`; cola MCP existente preservada     |
| AC-02    | ✅ Proyección Svelte de snapshots, sin estados finales optimistas                                |
| AC-03    | ✅ Suscripción antes de snapshot, revisión, dirty/requery, foco y reintento                      |
| AC-04    | ✅ Transiciones/revisiones/terminales y duplicados cubiertos en Rust y Node                      |
| AC-05    | ✅ No cancelable explícito; rechazo backend probado sin cambio de estado                         |
| AC-06    | ✅ Historial 100 + máximo 64 activas, whitelist, límites y recuperación                          |
| AC-07    | ✅ Servicio real, descarga Modrinth, hash/archivo y persistencia; UI comprobada por IPC simulado |
| AC-08    | ✅ Origen UI/MCP seleccionado en Rust, correlación con job MCP real; System reservado            |
| AC-09    | ✅ Modern/Pixel, tamaños mínimos, teclado, sin animación continua y reduced motion               |
| AC-10    | ✅ Sin cambios de políticas/capabilities; regresión MCP/escenas/consola/ajustes/contenido        |
| AC-11    | ✅ Checks y pruebas ejecutados; GUI nativa interactiva declarada NO EJECUTADO                    |
| AC-12    | ✅ Contratos y registro incremental de operaciones documentados en DESIGN.md                     |

AC-01–AC-12: implementados y verificados con la combinación de pruebas descrita.
AC-05 usa la alternativa explícita no cancelable admitida por la misión. AC-07 tiene
instalación real de red/archivo; presentación por IPC simulado y recorrido GUI manual
se declaran por separado. Las verificaciones nativas interactivas anteriores siguen
siendo pendientes de QA, no se marcan PASS. No se reescriben políticas MCP, updater,
cuentas, consola, escenas ni launch. Cambios de versión/notas y tag están autorizados
expresamente por el mensaje del propietario solicitando publicar esta beta.
