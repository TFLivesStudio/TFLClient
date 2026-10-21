# Beta 2 · 0.13.1-beta.1

Esta rama integra la beta de Content Discovery (`67a67ac`, `v0.13.0-beta.1`)
con Visual Refresh (`1f7c837`, `v0.12.4-beta.1`). Conserva la historia de ambos
trabajos. No requiere migrar carpetas de instancias, cuentas ni partidas.

## Ajustes por tarea

| Página           | Qué pertenece aquí                                                                                         |
| ---------------- | ---------------------------------------------------------------------------------------------------------- |
| General          | Idioma, selección de archivos e integración con diálogos, caché temporal                                   |
| Apariencia       | Tema, Modern/Pixel, material, acento, fondo, tipografía, densidad, Home y calidad/animaciones del launcher |
| Minecraft y Java | RAM del juego, aviso de varias instancias y versiones de Java; los overrides siguen en cada instancia      |
| Cuentas          | Sesiones, cuentas offline, Microsoft y mascotas                                                            |
| Consola          | Historial de recursos, orden/ubicación/colores de gráficas, texto y colores de niveles                     |
| Actualizaciones  | Versión instalada, canal beta, descarga e instalación                                                      |

La búsqueda orienta a páginas por nombre de opción, con términos ES/EN y sin
exigir acentos. No hay un modo simple/avanzado que esconda opciones. Las acciones
de cuenta y los overrides por instancia conservan sus mecanismos existentes.

## Consola

- Un componente compartido para el log del cliente y la consola del servidor;
  el servidor consulta su propio PID, sin confundirlo con Minecraft cliente.
- CPU cruda: **100 % equivale a un núcleo lógico**. Se conserva también el
  porcentaje del equipo completo y se permite elegirlo. El eje se adapta a
  valores superiores al 100 %. RAM es memoria residente del proceso, no heap
  usado de Java. Disco muestra lectura + escritura por segundo.
- Muestras reales de `sysinfo` cada segundo aproximadamente, con tiempo e
  intervalo de backend. El primer muestreo no inventa un valor de CPU/disco.
  Consumidores simultáneos comparten el muestreo del mismo PID durante 500 ms.
- Historial acotado a 182 muestras / unos tres minutos; ventanas de 30/60/180 s.
  La falta de datos o una pausa dejan huecos. Al detenerse conserva el gráfico
  durante la sesión de esa consola. Cerrar/desmontar la consola inicia otra
  sesión de historial; no se escribe telemetría al disco.
- Ocultar la ventana o todas las gráficas suspende el sondeo; al terminar o
  desmontar se cancela. No hay solicitudes superpuestas.
- Gráficas encima, debajo o junto al registro; en ventanas pequeñas el lateral
  pasa arriba. Orden por arrastre o flechas, columnas y visibilidad por métrica.
  El control de arrastre por puntero funciona sin desactivar el file-drop nativo
  de Tauri en Windows. El registro siempre mantiene su área.
- Colores configurables, niveles explícitos además del color, búsqueda, filtro,
  copia del resultado visible, ajuste de líneas, tamaño y seguimiento del log.
  Se detectan WARN/ERROR/INFO/DEBUG/TRACE en Java, incluso si salen por stderr;
  stderr sin nivel se considera error. Se conserva el texto original.
- Preferencias locales versionadas (`tfl-console-v1`), validadas y sincronizadas
  entre ventanas por `storage`; no se amplía el acceso del log a ajustes,
  credenciales ni archivos. Solo recibe permiso de **escritura de texto** al
  portapapeles, nunca lectura. Se mantienen diagnóstico, rollback y detener.

## Pixel, sidebar e integración

Pixel añade botones de piedra con relieve, acciones verdes, controles hundidos,
slots de inventario y marcos de menú con CSS estático. Respeta tema, material,
Lite y reducción de movimiento; no añade efectos continuos.

La sidebar compacta usa controles de 44 × 44 px y mantiene el pie abajo,
independientemente de la longitud de la lista. En expandido el perfil ocupa una
fila y las acciones otra para mostrar el nombre completo. Se conserva la
mascota offline. Las skins premium usan la textura oficial, con URL HTTPS y
protección contra respuestas atrasadas al cambiar de cuenta. El aviso de
actualización ocupa el área de contenido para no tapar los controles del pie.

Explorar abre el catálogo amplio compatible con la instancia. Se mantienen
paginación, orden, fichas Markdown saneadas y selección de versiones de
Content Discovery. Se corrigió un efecto que leía el estado de búsqueda al
reiniciar el contexto: avanzar de página podía dispararlo y regresar a página 1.
La corrección se aplica a Descargar, Modpacks y la vista amplia. Esta última
recibe el mismo control de foco y Escape que los demás diálogos.

## Verificación

Ejecutado localmente en Linux:

- `bun install --frozen-lockfile`, `bun run check`, `bun run lint`, `bun run build`.
- `bun test tests/unit`: 10 pruebas, incluyendo persistencia, límites de
  historial, huecos, tasa de disco y CPU >100 %.
- `cargo test --workspace --lib --locked`: 139 pruebas; incluye lectura real
  del proceso de pruebas, coalescencia de consumidores y ACL de Tauri.
- `cargo clippy --workspace --all-targets --locked` y `cargo audit`: sin errores
  ni vulnerabilidades no exceptuadas; persisten avisos preexistentes.
- Navegador sobre build de producción: smoke (26 capturas), workspaces a
  800×600, ciclo de vida Lite/WebGL y beta2 (12 capturas). Sin errores de página.
  Beta2 verifica las seis páginas, búsqueda, catálogo paginado y ficha, tamaños
  de sidebar, CPU >100 %, colores, arrastre por puntero, persistencia,
  sincronización entre ventanas y fin del sondeo al cerrar/detener.

Las pruebas de navegador usan IPC simulado y textura sintética. No equivalen a
iniciar sesión en Microsoft, instalar contenido real, ejecutar Minecraft ni
medir una GPU/instalador nativo de Windows o macOS. El muestreador de Rust sí
lee un proceso real en su prueba. Las nuevas gráficas no cargan librerías de
charts ni WebGL. Los avisos de Svelte existentes se documentan en la salida de CI.

## Build y publicación

CI incluye un job de navegador sobre producción y sube capturas como artifact
`beta-ui-review`. El workflow **Build TFL Client** conserva su matriz Windows,
Linux y macOS. Un `workflow_dispatch` sobre esta rama construye instaladores
para revisión **sin crear un release**.

La versión preparada es **0.13.1-beta.1**. El propietario decide cuándo crear
`v0.13.1-beta.1` y publicar; el workflow existente lo reconocerá como prerelease.
No se creó tag ni release durante esta actualización. No mezclar este código
con el tag anterior `v0.12.4-beta.1` ni sobrescribir sus instaladores.

Para repetir las pruebas de navegador sin sobrescribir evidencia histórica:

```sh
export VISUAL_URL=http://localhost:1423
export VISUAL_OUTPUT=artifacts/beta2/regression
node tests/visual/beta2.mjs
node tests/visual/smoke.mjs
node tests/visual/workspaces.mjs
node tests/visual/lifecycle.mjs
```

Las capturas nuevas están en `artifacts/beta2/`, ignorado por git. Playwright se
instala aislado y fijado a 1.57.0 en CI, sin añadir dependencias al launcher.
