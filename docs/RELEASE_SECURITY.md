# Seguridad del pipeline de release de TFL Client

Este documento separa dos cosas: lo que **ya está implementado en el código** (workflows, scripts) y lo que **solo el dueño del repo puede hacer** en la configuración de GitHub o fuera de línea. Lo segundo es un checklist al final.

## 1. Qué está implementado en el repo

| Qué                                                                               | Dónde                                                |
| --------------------------------------------------------------------------------- | ---------------------------------------------------- |
| Versión única (package.json, tauri.conf.json, Cargo.toml, Cargo.lock)             | `scripts/release-version.mjs`                        |
| El release falla si el tag no coincide con esas cuatro versiones                  | `build.yml`, paso "Verificar versión"                |
| Compuertas de calidad antes de construir (check, lint, fmt, clippy, tests, audit) | `.github/workflows/ci.yml`                           |
| El release exige que las compuertas pasen (`needs: quality`)                      | `build.yml`                                          |
| Permisos mínimos (`contents: read`) a nivel workflow y por job                    | `build.yml`, `ci.yml`, `mp-guard.yml`                |
| Todas las acciones de terceros fijadas a un commit SHA                            | todos los workflows y `.github/actions/`             |
| `persist-credentials: false` en cada checkout                                     | todos los workflows                                  |
| `concurrency`: dos builds del mismo tag no corren en paralelo                     | `build.yml`                                          |
| Secrets solo en los pasos que los usan (nunca a nivel workflow/job)               | `build.yml`                                          |
| Rust fijado a una versión exacta, igual en CI y en las 3 plataformas              | `rust-toolchain.toml` + `.github/actions/setup-rust` |
| `Cargo.lock` versionado y `--locked` en CI; `bun install --frozen-lockfile`       | `ci.yml`, `build.yml`                                |
| `cargo audit` con excepciones justificadas una por una                            | `.cargo/audit.toml`                                  |
| Plomería de firma de macOS (inactiva hasta cargar secrets)                        | `build.yml`, paso "Configurar firma de macOS"        |
| `environment: release` en el job de build                                         | `build.yml`                                          |

### Cómo se sube la versión

```
bun run release:version 0.11.0           # actualiza los 4 archivos
bun run release:version --check          # verifica que coincidan
bun run release:version --check --tag v0.11.0
```

El script **no** toca `src/lib/releaseNotes.ts`: las notas para jugadores se escriben a mano en cada versión (el script avisa si falta la entrada).

Flujo de release: `release:version X.Y.Z` → escribir las notas → commit → tag `vX.Y.Z` → push del tag.

### Qué NO se puede probar desde el código

Los workflows no se pueden ejecutar fuera de GitHub. Se validaron estáticamente (`actionlint` + shellcheck, sintaxis YAML y los scripts de shell por separado), pero hasta que corra de verdad un `workflow_dispatch` y un tag real no está probado: la resolución de los pins, el `environment: release`, el paso de firma de macOS y los tests en Linux. **Antes de depender de esto, hacer un `workflow_dispatch` de prueba** (no publica Release).

## 2. Checklist manual del dueño del repo

Estado de la configuración, leído el 2026-10-07 con `gh api repos/TFLivesStudio/TFLClient/...` (con permisos de admin):

- Repo **público**, Actions habilitado, `allowed_actions: all`, `sha_pinning_required: false`.
- Permisos por defecto del workflow: **read** (correcto). `can_approve_pull_request_reviews: false` (correcto).
- Aprobación de workflows de forks: `first_time_contributors`.
- **Sin rulesets**, **sin protección de la rama `main`**, **sin environments**.
- Secrets del repo: `RELEASE_TOKEN` y `TAURI_SIGNING_PRIVATE_KEY`. **No existe** `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (el workflow lo referencia y llega vacío; el release v0.10.16 se firmó así, lo que indica que la clave se generó sin contraseña, pero no se pudo verificar desde acá).
- Secret scanning, push protection y Dependabot: **desactivados**.
- Secrets a nivel organización: no se pudieron leer (403, hace falta ser admin de la org). Revisarlos a mano.

### 2.1 Quién puede publicar un release

Un tag `v*` empuja el workflow que firma con la clave del updater y publica el Release del que se actualizan todos los usuarios. Quien pueda empujar un tag controla lo que reciben.

- [ ] Settings → Rules → Rulesets → **New tag ruleset**: patrón `v*`, activar _Restrict creations_, _Restrict updates_ y _Restrict deletions_, con bypass solo para los pocos mantenedores que pueden publicar. Así nadie reescribe ni borra un tag ya publicado.
- [ ] Settings → Rules → **branch ruleset** para `main`: exigir PR y que pase el check `CI` antes de mergear; sin force-push. (Ahora mismo cualquiera con permiso de escritura puede empujar directo a `main` y desde ahí cortar un tag.)
- [ ] Revisar Settings → Collaborators and teams: dar _Write_ solo a quien deba tenerlo.

### 2.2 Permisos de Actions

- [ ] Settings → Actions → General → _Workflow permissions_: dejar **Read repository contents** (ya está) y **sin** _Allow GitHub Actions to create and approve pull requests_ (ya está).
- [ ] _Fork pull request workflows_: dejar que los PR de forks **no reciban secrets** (es el comportamiento por defecto en repos públicos; no activar _Send secrets to workflows from pull requests_) y exigir aprobación para colaboradores nuevos. Hoy está en "first-time contributors"; considerar "all outside collaborators".
- [ ] _Actions permissions_: limitar a acciones de GitHub + las ya usadas (`dtolnay`, `Swatinem`, `oven-sh`, `tauri-apps`, `taiki-e`, `gradle`) y activar **Require actions to be pinned to a full-length commit SHA**. Los workflows ya cumplen; activarlo después de la primera corrida exitosa.
- [ ] Activar **Secret scanning** y **Push protection** (gratis en repos públicos) y Dependabot alerts.

### 2.3 Secrets dentro del environment `release`

El job de build declara `environment: release`; GitHub crea el environment solo la primera vez que corre. Mientras no tenga reglas, no cambia nada. Para que sirva:

- [ ] Settings → Environments → `release` → **Required reviewers**: agregar al menos una persona (puede ser vos). Cada build (también los `workflow_dispatch` de prueba, porque comparten job) esperará aprobación manual antes de tocar los secrets.
- [ ] En ese environment, _Deployment branches and tags_: permitir solo tags `v*` **y** la rama `main` (para poder seguir haciendo builds de prueba), o bien aceptar que los de prueba también pidan aprobación.
- [ ] **Mover** `RELEASE_TOKEN`, `TAURI_SIGNING_PRIVATE_KEY` (y `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` si se crea) de _Repository secrets_ a _Environment secrets_ de `release`, y **borrarlos** del repo. Si quedan en los dos lados, el del environment tiene prioridad pero el del repo sigue expuesto a cualquier workflow.
- [ ] Los secrets de Apple (sección 3) también van ahí.

Con esto, un workflow nuevo o modificado en una rama cualquiera no puede leer la clave de firma: solo el job `build` en un tag aprobado.

### 2.4 El token `RELEASE_TOKEN` (PAT)

El `GITHUB_TOKEN` es de solo lectura por política de la org, así que el release usa un PAT.

- [ ] Que sea un **fine-grained PAT**, limitado **solo a este repo**, con únicamente _Contents: Read and write_ (más _Metadata: Read_, que es implícito). Nada de _classic_ con scope `repo`.
- [ ] Ponerle **vencimiento** (90 días o menos) y anotar la fecha en un calendario. Cuando vence, el release falla con 403 al crear el Release.
- [ ] Rotarlo si sale del equipo alguien que lo conoció, o ante cualquier sospecha: crear uno nuevo, actualizar el secret, revocar el viejo.
- [ ] Alternativa mejor a largo plazo: una **GitHub App** instalada solo en este repo (tokens de 1 hora), en vez de un PAT.

### 2.5 Clave del updater: respaldo y rotación

La clave privada firma cada actualización; la **pública** está en `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`) y es la única que los launchers ya instalados aceptan.

```
                    ┌──────────────────────────────┐
  clave privada ──► │ GitHub Secret (environment   │ ──► builds de release firman
  del updater       │ `release`)                   │     latest.json + instaladores
        │           └──────────────────────────────┘
        │
        └─ copia cifrada fuera de línea ──► gestor de contraseñas / pendrive cifrado
           (NUNCA en el repo, NUNCA en chat o email)   en otra ubicación física
```

- [ ] Hacer un **respaldo cifrado fuera de línea** de la clave privada (y de su contraseña, guardada aparte). Los secrets de GitHub **no se pueden leer de vuelta**: si se pierde la clave y no hay respaldo, **ningún launcher instalado podrá actualizarse nunca más** (hay que pedir a todos que reinstalen a mano).
- [ ] Guardar el respaldo en al menos dos lugares distintos (por ejemplo, gestor de contraseñas + dispositivo cifrado en otro lugar).
- [ ] Considerar protegerla con contraseña (hoy no existe el secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). Protegerla implica generar un par nuevo → ver rotación.

**Rotación de la clave** (si se filtra, o para ponerle contraseña): la app instalada solo confía en la pubkey que lleva dentro, así que el cambio tiene que hacerse en dos releases:

1. Generar el par nuevo (`bunx tauri signer generate`).
2. Release **puente** N: cambiar la `pubkey` en `tauri.conf.json` por la **nueva**, pero **firmar este release con la clave VIEJA** (la que los launchers instalados aún aceptan). Subirlo.
3. Cuando los usuarios actualizaron a N, ya llevan la pubkey nueva. Desde el release N+1, cargar la clave **nueva** en el secret y firmar con ella.
4. Revocar/destruir la clave vieja solo cuando casi nadie quede en versiones anteriores a N.

Si la clave vieja se **filtró**, este camino sigue siendo el único sin reinstalación manual, pero hasta completarlo un atacante con esa clave podría firmar actualizaciones: acelerar el release puente y avisar a los usuarios.

### 2.6 Otros

- [ ] Revisar los avisos de dependencias que no se pueden arreglar desde acá (`.cargo/audit.toml`, y `bun audit`, que hoy marca paquetes de desarrollo y no bloquea).
- [ ] Revisar las ejecuciones del workflow de Actions tras cada release las primeras veces.

## 3. macOS: firma y notarización

**Estado hoy:** la app se firma _ad-hoc_ (`bundle.macOS.signingIdentity: "-"` en `tauri.conf.json`). Eso evita el falso "está dañado", pero Gatekeeper sigue avisando "desarrollador no identificado". Para quitar el aviso hace falta una cuenta **Apple Developer Program** (USD 99/año), firma con _Developer ID Application_ y notarización.

**Qué está preparado:** el job de build de macOS ya tiene un paso que lee seis secrets y exporta cada uno **solo si no está vacío**. Esto importa: Tauri lee `APPLE_*` con `var_os`, y GitHub entrega un secret inexistente como cadena vacía, que Tauri trataría como "definida" (intentaría importar un certificado vacío y `APPLE_SIGNING_IDENTITY=""` pisaría la firma ad-hoc `-`). Con los secrets sin cargar el paso no exporta nada y el build queda **idéntico** al de hoy. Solo corre en el job de macOS.

**Pasos cuando haya cuenta de Apple:**

1. En developer.apple.com → Certificates: crear un certificado **Developer ID Application** (con una CSR creada desde Acceso a Llaveros).
2. En Acceso a Llaveros, exportar el certificado **con su clave privada** como `.p12` con una contraseña fuerte.
3. Codificarlo: `base64 -i certificado.p12 | pbcopy` → secret `APPLE_CERTIFICATE`. La contraseña del `.p12` → `APPLE_CERTIFICATE_PASSWORD`.
4. `APPLE_SIGNING_IDENTITY`: el nombre exacto que muestra `security find-identity -v -p codesigning`, p. ej. `Developer ID Application: Nombre (TEAMID)`.
5. `APPLE_ID`: el correo de la cuenta de Apple. `APPLE_PASSWORD`: una **contraseña específica de app** (appleid.apple.com → Inicio de sesión y seguridad → Contraseñas de apps), **no** la contraseña de la cuenta. `APPLE_TEAM_ID`: los 10 caracteres de developer.apple.com → Membership.
6. Cargar los seis secrets en el environment `release` (sección 2.3). Hay que cargar **todos o ninguno**: un certificado sin contraseña, o `APPLE_ID` sin `APPLE_PASSWORD` y `APPLE_TEAM_ID`, hace fallar el build.
7. **Cambios en `src-tauri/tauri.conf.json`** (no se tocaron acá porque ese archivo lo edita otra persona): al tener credenciales reales, quitar o cambiar `bundle.macOS.signingIdentity: "-"` (la variable `APPLE_SIGNING_IDENTITY` la pisa, pero conviene dejarlo coherente), poner `hardenedRuntime: true` (hoy `false`; la notarización lo exige) y agregar un archivo de entitlements (`bundle.macOS.entitlements`) con lo que la app necesite bajo hardened runtime, p. ej. `com.apple.security.cs.allow-jit` y `com.apple.security.cs.allow-unsigned-executable-memory` por el WebView, y `com.apple.security.cs.disable-library-validation` si se cargan bibliotecas de terceros (Java incluido). Probar con un `workflow_dispatch` antes de publicar.
8. Cuando funcione, el updater de macOS sigue verificando con la misma clave del updater (es independiente de la firma de Apple).

## 4. Detalles que conviene saber

- El job de calidad corre los tests con `cargo test --workspace --lib`. Hay tests que dependen de la red (manifests de Mojang) y de rutas del sistema: tres tests de `aqua` (`jre::client::tests::test_find_runtime_root_*`) fallan **en macOS** por comparar `/var/...` con `/private/var/...`; en el runner de Linux deberían pasar, pero no se pudo verificar sin correr el workflow.
- `clippy` corre **sin** `-D warnings` porque hay unas 20 advertencias previas. Cuando se limpien, conviene agregarlo en `ci.yml`.
- `cargo audit` ya no tiene vulnerabilidades pendientes (se actualizó `rustls` 0.23.44 → 0.23.45, RUSTSEC-2026-0285). Los avisos restantes son crates sin mantenimiento que traen Tauri/GTK/postcard; están listados con su motivo en `.cargo/audit.toml`.
