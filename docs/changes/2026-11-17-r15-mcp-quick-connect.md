# R15 — MCP local y conexión de asistentes

Estado de trabajo: implementación incorporada; la verificación integral, nativa y de
publicación corresponde a la matriz de 0.15.4. Este informe no certifica una release.

## Comportamiento implementado

Simple tiene cuatro interruptores principales: MCP local, Codex app/CLI, OpenCode
app/CLI y Claude Code. El archivo de destino se muestra antes de modificarlo; si
hay más de un perfil/formato detectado se exige seleccionarlo. «Configurado» no
significa una sesión conectada ni autorizada. Los permisos existentes no cambian
al configurar un cliente y Simple solo permite conceder lectura a una sesión nueva.
Avanzado conserva la configuración manual, el diagnóstico, los permisos y la auditoría.

El prompt copiable pide descubrir herramientas, hacer una consulta de lectura,
explicar permisos reales y reconocer la ausencia de conexión. No contiene rutas
personales ni tokens. El diagnóstico local preexistente comprueba el rechazo de
una consulta sin autorización; no es prueba de conexión de un agente externo.

Los comandos de detección/escritura se ejecutan fuera del hilo UI. Solo aceptan IDs
de destinos acotados que el backend vuelve a resolver; no aceptan rutas arbitrarias.
Se rechazan enlaces/junctions, archivos de más de 1 MiB, estructuras mal formadas,
claves duplicadas y entradas ajenas o modificadas. El registro privado guarda hashes
de propiedad; al desconectar elimina solo la entrada correspondiente, nunca restaura
una copia antigua sobre la configuración actual del cliente.

La edición TOML conserva comentarios mediante `toml_edit` (dependencia ya transitiva).
JSONC conserva los bytes ajenos a la propiedad editada mediante spans y validación
estructural. Cada escritura prepara un respaldo privado, sincroniza el temporal y
reemplaza el destino mediante rename en su propio directorio. Si falla guardar el
registro, revierte la escritura solo cuando el archivo sigue siendo el que TFL escribió.
Los respaldos pueden contener datos privados del cliente y nunca se devuelven por IPC.

## Documentación oficial contrastada

Codex comparte la configuración MCP entre escritorio, CLI e IDE del mismo host.
El destino predeterminado es `~/.codex/config.toml`, tabla `mcp_servers`; los proyectos
deben estar confiados para usar configuración de proyecto. El escritorio necesita
reinicio de la conexión. [MCP de Codex](https://learn.chatgpt.com/docs/extend/mcp?surface=cli).
`CODEX_HOME` puede seleccionar otra raíz; solo se ofrece si es absoluta y existe.
[Variables de Codex](https://learn.chatgpt.com/docs/config-file/environment-variables).

OpenCode permite JSON y JSONC en `~/.config/opencode/opencode.json(c)`; las políticas
administradas pueden prevalecer sobre el archivo de usuario.
[Configuración](https://opencode.ai/docs/config). V2 usa `mcp.servers` y `disabled`.
[MCP](https://opencode.ai/docs/mcp-servers). V2 conserva los campos V1 compatibles
sin reescribir el archivo. Se respeta una estructura V2 existente y, en otro caso,
se añade la forma V1 compatible, sin migrar preferencias ajenas.
[Migración V1](https://opencode.ai/docs/migrate-v1/).

Claude Code separa scope user (`~/.claude.json`) de project (`.mcp.json`) y local
(entrada por proyecto en `~/.claude.json`). Quick connect usa user; la alternativa
manual permite project. No modifica `claude_desktop_config.json`: es otro producto.
[MCP de Claude Code](https://code.claude.com/docs/en/mcp). Cuando hay
`CLAUDE_CONFIG_DIR` personalizado se ofrece una explicación y configuración manual;
no se supone una ruta del registro no verificada.

## Evidencia y límites

- Se inspeccionaron, sin leer credenciales ni cambiar configuración, Codex CLI
  `0.166.0-alpha.5.1` y OpenCode Desktop/CLI `2.2.5` en Windows. Se ejecutaron
  únicamente `--version` y la ayuda de MCP. Claude Code no está en PATH ni se detectó
  su configuración predeterminada; no se certifica su ejecución real aquí.
- Los tests Rust cubren JSON/JSONC/TOML, comentarios, Unicode, escapes, conflictos
  V1/V2, eliminación en distintas posiciones, idempotencia, revisión obsoleta,
  conservación de cambios ajenos, entrada modificada y rollback ante error.
  Su resultado ejecutado debe constar en la matriz final; no se infiere de su existencia.
- La revisión de hash evita sobrescribir snapshots obsoletos y vuelve a comprobar
  el archivo antes del reemplazo. No hay una transacción distribuida con los editores
  de terceros: una sustitución externa exactamente entre la comprobación y rename
  sigue siendo una carrera residual del sistema de archivos, que debe documentarse.
- Escritorio con backend remoto, WSL y perfiles no visibles desde el host del launcher
  requieren elegir/configurar el host correcto manualmente. No se escribe en perfiles
  remotos ni directorios administrados. Un archivo configurado no demuestra conexión.
- Faltan las pruebas de lectura autorizada desde cada cliente, la negativa sin
  herramientas y la matriz nativa macOS/Linux para considerar R15 verificado.
