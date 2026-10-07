# Medir el rendimiento de TFL Client

TFL Client apunta a ser un launcher liviano. Para no depender de "siento que
ahora abre más lento", hay instrumentación interna. **No hay telemetría**: nada
sale de la máquina; las mediciones van al log del proceso y a un buffer en
memoria.

## Cómo activarla

- **Desarrollo** (`bun run tauri dev`): activa sola.
- **Un release instalado**: abrirlo con la variable `TFL_PERF=1`
  (`TFL_PERF=1 /ruta/al/launcher`, en Windows `set TFL_PERF=1` antes de
  ejecutarlo) y, para las mediciones de la interfaz, en la consola del
  WebView `localStorage.setItem('tfl-perf','1')` y reabrir. Sin eso, medir no
  cuesta nada.

Cada medición sale en el log con el target `tfl::perf`:

```
INFO tfl::perf: metric="instances.load" ms=3.2
```

Para el reporte completo (promedio/mín/máx por métrica y consumo en reposo), en
la consola del WebView: `await window.__tflPerf()`.

## Qué se mide

| Métrica                                                                          | Qué es                                                                                                                                                         |
| -------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `startup.setup_done` / `startup.window_shown`                                    | Desde que arranca el proceso hasta que termina `setup()` / hasta que se muestra la ventana (cold startup del lado Rust)                                        |
| `ui.usable`                                                                      | Desde que el WebView empieza a cargar hasta el primer cuadro pintado con el launcher listo (sin pantalla de carga, con instancias): **tiempo hasta UI usable** |
| `ui.accounts_settings`, `ui.instances`                                           | Carga de cuentas/ajustes y de instancias vista desde la interfaz (incluye el viaje por IPC)                                                                    |
| `accounts.load`, `settings.load`, `instances.load`                               | Lo mismo del lado Rust                                                                                                                                         |
| `content.search`                                                                 | Una búsqueda en Modrinth                                                                                                                                       |
| `content.install`                                                                | Instalar un mod/shader/pack: resolver dependencias + descargar + reemplazar                                                                                    |
| `launch.total`                                                                   | De "Jugar" a proceso de Minecraft arrancado (incluye también el tiempo hasta un error si falla)                                                                |
| `launch.java`, `launch.prepare_minecraft`, `launch.auth_refresh`, `launch.spawn` | Fases de lo anterior: resolver Java, preparar Minecraft (versión, librerías, assets, loader), refrescar la sesión de Microsoft, lanzar el proceso              |
| `idle.memory_mb`, `idle.cpu_percent`                                             | RAM y CPU del launcher en reposo (una muestra a los 15 s de abrir, y en `window.__tflPerf()`)                                                                  |

## Objetivos internos

Todavía no se fijaron: hace falta una línea base medida en una máquina real
(no se inventan números). El procedimiento:

1. Con la medición activa, abrir el launcher 5–10 veces en frío y anotar el
   promedio de `startup.window_shown`, `ui.usable`, `instances.load`,
   `idle.memory_mb`, `idle.cpu_percent`; y lanzar 3–5 veces una instancia ya
   descargada para `launch.total`.
2. Escribir los valores objetivo (algo por encima de la línea base) en
   `budget_ms()` de `src-tauri/src/core/perf.rs`.
3. Desde ahí, cualquier medición que supere su objetivo deja un `WARN` en el
   log: "esta modificación subió el startup 400 ms" deja de ser una sensación.

Objetivo de CPU en reposo: ~0 % (se mide con `idle.cpu_percent`).

## Para comparar versiones

Correr el procedimiento anterior con la versión vieja y la nueva en la misma
máquina y comparar los promedios; el reporte de `window.__tflPerf()` ya trae
mín/promedio/máx.
