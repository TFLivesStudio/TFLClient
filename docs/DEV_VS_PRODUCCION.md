# Desarrollo vs. producción

TFL Client no depende de editar nada a mano antes de publicar: cada diferencia
entre `tauri dev` y una build oficial sale de la propia configuración.

| Tema                 | Desarrollo (`bun run tauri dev`)                                                                                                                              | Producción (build/release)                                                                                                                                                 |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| CSP                  | `app.security.devCsp` en `tauri.conf.json`: permite `localhost`, `ws://localhost` (HMR de Vite) y `'unsafe-inline'`                                           | `app.security.csp`: `script-src 'self'` (+ hash que Tauri inyecta al compilar), sin `localhost`, sin `https:` abierto. Un test (`acl_tests.rs`) falla si alguien la afloja |
| DevTools             | Disponible: Tauri lo habilita solo en builds de debug                                                                                                         | No existe: la feature `devtools` de `tauri` NO está en `src-tauri/Cargo.toml` (un test lo vigila)                                                                          |
| Frontend             | Servidor de Vite en `http://localhost:1420`                                                                                                                   | Archivos estáticos de `build/` embebidos en el binario                                                                                                                     |
| Updater              | El chequeo **automático** se omite (`import.meta.env.DEV`), para no pisar el binario de desarrollo con un release; el chequeo manual de Ajustes sigue andando | Chequeo en segundo plano al abrir (si `auto_updates` está activo), endpoint y clave pública oficiales de `tauri.conf.json`                                                 |
| Logging              | `info` + `debug` de `tflclient_lib`                                                                                                                           | Solo `info`. En ambos se puede pisar con `RUST_LOG=...` (por ejemplo para pedirle a un usuario un log con más detalle)                                                     |
| Permisos de ventanas | Los mismos: `capabilities/main.json` (ventana principal) y `capabilities/log-window.json` (ventanas `log-*`)                                                  | Idem                                                                                                                                                                       |

## Permisos por ventana

- `src-tauri/build.rs` declara como "app manifest" **todos** los comandos que
  figuran en `generate_handler![...]` de `src/lib.rs` (esa lista es la única
  fuente de verdad) y genera `src-tauri/permissions/` (ignorado por git).
- La ventana principal recibe el set `app-commands-all` (todos los comandos
  propios) y solo los permisos de plugin que la interfaz usa.
- La ventana de log (`log-*`) recibe únicamente escuchar eventos, cerrarse y los
  comandos que usa `InstanceLogWindow.svelte`. Si esa pantalla pasa a usar otro
  comando, hay que sumarlo en `capabilities/log-window.json` **y** en
  `LOG_WINDOW_COMMANDS` de `build.rs`; el test
  `log_window_capability_covers_exactly_what_its_screen_uses` avisa si las
  listas no coinciden con lo que el componente realmente llama.
- Agregar un comando nuevo: registrarlo en `lib.rs`. Queda habilitado solo para
  la ventana principal.
- Los tests de `src-tauri/src/acl_tests.rs` usan el ACL real de Tauri con un
  runtime simulado: comprueban qué puede y qué no puede invocar cada ventana.

## Asset protocol

El WebView solo puede leer por `asset://` (scope en `tauri.conf.json`):

- `~/.tflclient/instances/*/icon.*` (íconos de instancia),
- `~/.tflclient/instances/*/screenshots/*` (capturas),
- `~/.tflclient/shared/appearance/*` (wallpaper propio).

`~/.tflclient/settings/**` está **denegado** explícitamente: ahí viven los tokens
de las cuentas y el secret de playit.gg. Si una pantalla nueva necesita mostrar
otra imagen local, hay que agregar esa ruta puntual al scope (y a
`asset_protocol_only_serves_icons_screenshots_and_wallpaper`).

## Imágenes remotas permitidas por la CSP

`img-src` admite `https://cdn.modrinth.com` (íconos de mods/packs) y
`https://textures.minecraft.net` (skins y capas). Si una imagen de otro host
debe mostrarse, se agrega ahí a propósito; mientras no esté, la interfaz cae al
avatar de letra.
