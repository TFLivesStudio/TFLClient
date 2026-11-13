<!-- prettier-ignore-start -->

# TFL Client 0.14.0-beta.7.2 — Stability & Bugfix (Windows-first)

Base exacta: tag `v0.14.0-beta.7.1` (`f922bb50236a11d47b0197aec62b9ffe516b4d75`), no `main` (rama atrasada) ni PR #9. Esta rama es **trabajo en curso**, no una release ni una certificación.

## Objetivo
Diagnosticar y reparar fallos de USO REAL, no centrarse en temas/animaciones. Prioridad Windows (instalación, actualización, cuentas, Java, Fabric/Forge, mundos, errores de disco/red), compatibilidad Linux/macOS. Prohibido usar perfiles, mundos o credenciales reales del usuario para pruebas destructivas.

## Dos correcciones iniciales en este PR
1. Nombres de instancias nuevas: rechazar nombres de dispositivos reservados por Windows (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`), nombres ambiguos con espacios/puntos finales y controles. La búsqueda de instancias heredadas sigue utilizando la validación anterior para permitir acceso/migración de nombres históricos.
2. La consulta de actualizaciones deja de ocultar los errores de Modrinth como «sin actualizaciones». Ahora devuelve un error visible al consumidor del comando.

3. Updater del launcher: si otro chequeo de actualizaciones cambia la versión pendiente durante una descarga, se rechaza asociar bytes del paquete anterior a la nueva versión. Pendiente validar concurrencia y firma con QA.

## Inventario: hallazgos revisados en fuente, NO todos reproducidos en GUI

### P0 — riesgo de datos
- **B7.2-01: última reversión eliminada antes de tener una nueva** — `src-tauri/src/services/content/rollback.rs::RollbackRecorder::begin` elimina `.tfl_rollback/` inmediatamente. `update_all` inicia el recorder al conocer updates, antes de confirmar descarga/reemplazo. Si todos los downloads fallan, se puede perder la última reversión aunque no se haya actualizado ningún mod. Requiere reproducción con fixtures y staging atómico; no parchear a medias.
- **B7.2-02: retiro destructivo de jar sin backup** — `RollbackRecorder::retire` borra el jar viejo si moverlo a backup falla. Un fallo de disco/permisos puede perder la única copia. Rediseñar para abortar el cambio sin eliminar el original y preservar manifest anterior.
- **B7.2-03: restauración parcial reportada como exitosa** — `restore_rollback` ignora errores borrando archivos nuevos y elimina siempre el directorio de respaldo tras iterar, incluso si no movió todos los jars. Requiere transacción/fixture de fallo de IO, no eliminar backups hasta verificación.
- **B7.2-04: backup de mundo antes de update es best-effort silencioso** — `content/updates.rs::update_all` ignora el resultado de `backup_world_dir`; si existe `saves/` y falla la copia, se sigue tocando el conjunto de mods. Al menos mostrar aviso/permitir abortar de forma segura.

### P1 — funcionalidad
- **B7.2-10: race de updater con actualización concurrente** — `commands/updater.rs::update_download` antes guardaba bytes del download terminado en `PENDING` sin comparar la versión contra la que se inició; si otro `update_check` había reemplazado la actualización pendiente, se podían mezclar bytes y metadata. Corregido en PR #11 con validación de versión; pendiente prueba de concurrencia en ejecutable real.

- **B7.2-05: proveedor Modrinth caído se interpreta como «sin updates»** — `content/updates.rs::check_updates` usaba `.unwrap_or_default()`; corregido en esta rama, pendiente CI+GUI.
- **B7.2-06: nombres reservados/ambiguos en Windows** — `instance_manager.rs::valid_instance_name` validaba separadores pero no nombres especiales; corregido en creación/rename en esta rama, pendiente CI+GUI.
- **B7.2-07: macOS chooser mata la app** — issue #1, crash confirmado por reporte de usuario de selector nativo para icono/skin. Se necesita ruta alternativa o solución de firmado/notarización con QA real. No cerrar por build exitoso.
- **B7.2-08: flujo de update del launcher aún no probado de extremo a extremo** — verificar canal beta, `latest.json`, firma, progreso, cierre del instalador Windows y persistencia. No modificar firmas ni claves.
- **B7.2-09: instalación interrumpida de dependencias no es transaccional** — Activity Core marca fallo/interrupted correctamente, pero `content.install` aún no ofrece rollback/cancelación; probar recuperación y dejar explícita la limitación.

### P2 — candidatos pendientes de reproducción
- Renombrar o eliminar una instancia con Minecraft ejecutándose: comprobar coordinación con el launcher, nombre usado como clave del proceso y rutas abiertas (más crítico en Windows).
- Duplicación o importación cortada (disco lleno, ACL, symlinks): comprobar limpieza de carpeta parcial, conservación del original y mensajes accionables.
- Instalación de Java 8/17/21/25: arquitecturas correctas, antivirus/archivos bloqueados en Windows, reinicio después de descarga cortada; dos lanzamientos concurrentes.
- Backup/restore de mundos: archivo ZIP corrupto, sin espacio, ZIP grande, backup seleccionado tras poda y restauración que falla al renombrar directorios.
- Actualizar mods habilitados y deshabilitados; reemplazar versiones con el mismo nombre de `.jar`; volver a ejecutar rollback.
- Cuenta Microsoft real, renovación y logout; dejar PR #9 de autenticación fuera del alcance de beta.7.1 salvo aprobación explícita de integración.
- Cambio de red/proxy, modo sin conexión, paths Unicode y carpetas sincronizadas (OneDrive), escalado DPI y sandbox por OS.
- Activity Center bajo fallo, descarga lenta, reinicio forzado, historial y origen MCP/UI; pruebas nativas aún sin certificar en beta.7.1.

## Matriz mínima de pruebas nativas (usar perfiles desechables)
| ID | OS | Flujo | Esperado |
|---|---|---|---|
| WIN-01 | Windows 10/11 | instalar y actualizar beta.6→beta.7.1/7.2 | Instalador válido, firma, versión correcta, datos preservados |
| WIN-02 | Windows | crear/renombrar instancia CON, NUL.txt, COM1, nombre con punto final | Error explicativo sin crear ruta ni tocar otra instancia |
| WIN-03 | Windows | Fabric 1.21.1, mod Modrinth con dependencia | Descarga verificada, Minecraft inicia y mod funciona |
| WIN-04 | Windows | Modrinth inaccesible/HTTP error durante consultar updates | Error de consulta, jamás «todo actualizado» falso |
| WIN-05 | Windows | interrumpir update por red/IO, intentar rollback | Archivos originales recuperables, historial exacto |
| WIN-06 | Windows | backup/restore mundo propio de TEST y ZIP inválido | Hash/árbol restaurados, original intacto ante error |
| WIN-07 | Windows | crear, duplicar, renombrar, borrar instancia, bajo juego abierto y cerrado | Ninguna pérdida ni fantasma, diálogos precisos |
| WIN-08 | Windows | falta Java, runtimes antiguos y actuales | Descarga válida/reintentos, launch real, cierre |
| WIN-09 | Windows | cuentas Microsoft/offline/renovación | Sesión de prueba, sin filtración de tokens en logs |
| WIN-10 | Windows | MCP aprobado/rechazado, Activity Center | Scope correcto, origen correcto, sin tarea falsa |
| WIN-11 | Windows | red cae con Activity Center activo | Estado failed/interrupted, barra y personaje sincronizados |
| WIN-12 | Windows | ruta Unicode, OneDrive sync, antivirus/archivo abierto | Error recuperable, sin pérdida de datos |
| MAC-01 | macOS | icono y skin personalizados con chooser | No crashear: issue #1 debe reproducirse/corregirse |
| MAC-02 | macOS arm64 | Java viejo vía Rosetta y Java nativo | Juego real arranca en ambas arquitecturas |
| LIN-01 | Linux | AppImage/deb + WebKit, launcher real | Se inicia, se actualiza según soporte, instala mods |
| LIN-02 | Linux | lectura/escritura backup con permisos insuficientes | Error honesto y archivos conservados |

## Gates antes de publicar
- Bloqueadores P0 de datos resueltos **con pruebas en fixtures**.
- `cargo fmt --all --check`, `cargo test --workspace --lib`, `cargo clippy --workspace --all-targets`, `bun run check`, `bun run lint`, `bun run build` y pruebas CI por OS.
- Evidencia GUI nativa de Windows (instalación, actualización, lanzamiento de Minecraft, backup/rollback); Linux y macOS cuando estén disponibles; no inferir de IPC simulado.
- El paquete firmado usa updater y workflow original; sólo el propietario decide publicar/taggear. Ningún merge automático.
- Registrar por fallo: ID, OS, build/commit, prerequisitos, pasos reproducibles, resultado observado/esperado, logs **redactados**, impacto, commit de fix, pruebas añadidas y estado de QA.

## Dependencias conocidas
- PR #10 `feat/activity-center` todavía no fue fusionado a `main`; la rama beta.7.2 parte del tag beta.7.1, y el PR de corrección deberá tomar como base `feat/activity-center` para no arrastrar una mezcla espuria de cambios.
- PR #9 autenticación pendiente y PR #4 Content Discovery mantienen su independencia.
- Issue #1 macOS ya documenta selección de archivos nativa y crash; no asumir resuelto.

<!-- prettier-ignore-end -->
