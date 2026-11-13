# Publicar versiones y guardar changelogs

Esta es la guía práctica de publicación de **TFL Client**. El recorrido habitual
es: preparar el código, escribir las notas, subir el commit, comprobar CI y
publicar el tag. GitHub Actions construye los instaladores y crea la release;
la web consulta sus notas automáticamente.

Los números `0.15.4-beta.1` y `0.15.4` de esta guía son ejemplos. Sustitúyelos por
la versión que corresponda en todos los nombres, claves y comandos.

## 1. Dónde guardar cada cosa

| Contenido                                  | Ruta                              | Para qué sirve                                                                           |
| ------------------------------------------ | --------------------------------- | ---------------------------------------------------------------------------------------- |
| Notas públicas completas                   | `docs/releases/v0.15.4-beta.1.md` | Texto que publica GitHub y que muestra la web en Novedades. Un archivo por tag, con `v`. |
| Notas breves del launcher                  | `src/lib/releaseNotes.ts`         | Panel «Qué hay de nuevo» de la aplicación. Una entrada por versión, sin `v`.             |
| Informe técnico de un cambio o sesión      | `docs/changes/AAAA-MM-DD-tema.md` | Archivos afectados, decisiones, pruebas y pendientes para quienes trabajan en el código. |
| Índice de versiones documentadas           | `docs/releases/README.md`         | Ayuda a encontrar las notas dentro del repositorio; no controla la web ni el updater.    |
| Notas antiguas trasladadas de la raíz      | `docs/history/legacy/`            | Archivo histórico original. Se conserva; los informes nuevos van a `docs/changes/`.      |
| Copias anteriores a una revisión editorial | `docs/history/`                   | Historial de textos sustituidos, con un nombre que identifique versión y fecha.          |

Mantén la raíz para los archivos del proyecto. Los changelogs nuevos se guardan
en las carpetas de esta tabla. Los instaladores, las firmas y `latest.json` los
publica el workflow como archivos de la release.

## 2. Preparar el repositorio

Necesitas Git, Node 24, Bun 1.4.2 y acceso de escritura al repositorio. Para
compilar localmente la parte nativa también necesitas el toolchain de
`rust-toolchain.toml` y las dependencias de Tauri de tu sistema. Los comandos
de esta guía se ejecutan desde la raíz; en Windows puedes usar Git Bash.

Comprueba primero tus cambios locales y actualiza la rama cuando estén guardados:

```sh
git status --short
git switch main
git pull --ff-only origin main
bun install --frozen-lockfile
```

Consulta las [releases existentes](https://github.com/TFLivesStudio/TFLClient/releases)
antes de elegir el número. Si utilizas GitHub CLI:

```sh
gh release list -R TFLivesStudio/TFLClient --limit 10
```

**Comprueba qué código vas a publicar.** Algunas betas históricas salieron desde
ramas distintas. `main` puede no contener los cambios de la última beta: integra
primero las correcciones que quieras incluir y revísalas. Cambiar el número de
versión no incorpora el código de otra rama.

## 3. Elegir el canal y actualizar la versión

| Publicación                | Versión en los archivos | Tag              | Estado en GitHub |
| -------------------------- | ----------------------- | ---------------- | ---------------- |
| Beta de ejemplo            | `0.15.4-beta.1`         | `v0.15.4-beta.1` | Prerelease       |
| Siguiente beta del ejemplo | `0.15.4-beta.2`         | `v0.15.4-beta.2` | Prerelease       |
| Estable del ejemplo        | `0.15.4`                | `v0.15.4`        | Release estable  |

Elige una versión superior a la que deben recibir tus usuarios. Para preparar
la beta de ejemplo:

```sh
bun run release:version 0.15.4-beta.1
```

Ese comando alinea `package.json`, `src-tauri/tauri.conf.json`,
`src-tauri/Cargo.toml` y la entrada de la aplicación en `Cargo.lock`.
Puedes revisar el resultado sin escribir archivos añadiendo `--dry-run`.
`--force` permite números iguales o menores; no hace falta en una publicación
normal y no soluciona un tag incorrecto.

## 4. Escribir las dos versiones de las notas

### Texto completo para GitHub y la web

Crea `docs/releases/v0.15.4-beta.1.md`. Empieza por un resumen breve, de hasta
260 caracteres: será el texto de la tarjeta de Novedades. Después añade solo los
apartados que correspondan. Este esquema sirve para preparar el archivo;
reemplaza las indicaciones por hechos comprobados antes de publicarlo:

```markdown
[Completar] Resume los cambios principales de esta versión.

**Canal:** Beta / prueba

> Versión de prueba: puede contener errores.

## Novedades

- [Completar] Explica la función añadida, dónde encontrarla y cómo utilizarla.

## Correcciones

- [Completar] Describe el fallo anterior y el comportamiento corregido.

## Al actualizar

- [Completar] Indica los pasos que deba realizar el usuario, si los hay.

## Referencias

[Web oficial](https://launcher.tflives.com) · [Ayuda](https://launcher.tflives.com/ayuda) · [Discord](https://discord.gg/tflives)
```

Elimina los apartados que no se apliquen. También puedes utilizar `## Mejoras`,
`## Mantenimiento` y `## Compatibilidad y límites`. Incluye al menos una sección
de cambios reales: Novedades, Mejoras, Correcciones o Mantenimiento.

Para una estable, cambia el canal a `Estable` y elimina el aviso de beta. No
inventes una fecha de publicación: GitHub y la web ya muestran la fecha real.
Puedes consultar un [ejemplo publicado](../releases/v0.15.3-beta.7.md) y la
[guía editorial](release-notes.md) para mantener la misma voz.

### Texto breve dentro del launcher

Abre `src/lib/releaseNotes.ts` y añade una entrada dentro de `RELEASE_NOTES`.
La clave lleva la versión completa **sin `v`**, incluidas las partes de beta.
Escribe de dos a cinco frases breves para jugadores. Por ejemplo, esta es la
forma de una entrada pendiente de completar, no una lista de cambios reales:

```ts
'0.15.4-beta.1': [
  '[Completar] Describe la principal novedad o corrección para el jugador.',
  '[Completar] Describe la segunda mejora o el paso necesario al actualizar.'
],
```

Conserva las entradas anteriores. El script de versión exige esta entrada al
comprobar el tag, aunque ya exista el Markdown completo. La web utiliza el texto
de la release de GitHub; el panel del launcher utiliza estas notas breves.

## 5. Comprobar antes de publicar

Revisa que no queden indicaciones de plantilla y que los cambios descritos estén
en el código que vas a etiquetar. Ejecuta:

```sh
bun run release:version --check --tag v0.15.4-beta.1
bun scripts/release-notes-body.mjs v0.15.4-beta.1
node --test tests/unit/*.test.mjs
bun run check
bun run lint
bun run build
```

El segundo comando imprime el cuerpo que recibirá GitHub. El generador prefiere
el archivo de `docs/releases/`; si no existe, utiliza las notas breves como
alternativa compatible. Para preparar una publicación completa, escribe ambos.
Si el Markdown existe pero es inválido, la generación falla.

La parte nativa se verifica con `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked`,
`cargo test --workspace --lib --locked` y `cargo audit`. CI también ejecuta las
comprobaciones de navegador y de MCP. Comprueba en una build de prueba los flujos
y plataformas afectados; una compilación correcta no sustituye esa revisión.

## 6. Subir la preparación y esperar a CI

Con el código ya revisado, guarda la preparación de la publicación:

```sh
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml Cargo.lock
git add src/lib/releaseNotes.ts docs/releases/v0.15.4-beta.1.md
git diff --cached
git commit -m "chore(release): prepare 0.15.4-beta.1"
git push origin main
```

Incluye también los informes técnicos o cambios del índice que hayas preparado.
En [Actions](https://github.com/TFLivesStudio/TFLClient/actions), comprueba que
**CI** termina correctamente para ese mismo commit. Subir este commit a `main`
ejecuta CI, pero todavía no crea la release.

Para obtener instaladores de prueba sin publicar una release, puedes ejecutar
**Build TFL Client → Run workflow**, seleccionando la rama `main`. Con GitHub CLI:

```sh
gh workflow run build.yml --ref main -R TFLivesStudio/TFLClient
```

Esa ejecución manual sobre la rama genera artefactos de Actions. La configuración
del workflow y sus secretos se explican en [RELEASE_SECURITY.md](../RELEASE_SECURITY.md).

## 7. Crear el tag: este paso publica la versión

Verifica que `HEAD` sigue siendo el commit cuya CI comprobaste y que los archivos
de preparación están guardados. Si cambió el código, revisa la CI de ese commit.
Después:

```sh
git status --short
git log -1 --oneline
git tag -a v0.15.4-beta.1 -m "TFL Client v0.15.4-beta.1"
git push origin v0.15.4-beta.1
```

**Subir el tag `v…` activa la publicación.** El workflow comprueba la versión,
ejecuta las compuertas de calidad, construye Windows/macOS/Linux, publica las
notas y adjunta instaladores, firmas y metadatos del updater. El tag con sufijo
beta se marca como prerelease.

El workflow gestiona la creación de la release y sus notas. Espera a que termine
la matriz completa: una release puede aparecer cuando una plataforma ha acabado,
aunque otra siga construyéndose o haya fallado.

## 8. Verificar la publicación y la web

```sh
gh release view v0.15.4-beta.1 -R TFLivesStudio/TFLClient --json tagName,isPrerelease,isDraft,publishedAt,assets
gh api repos/TFLivesStudio/TFLClient/releases/latest --jq .tag_name
```

En la beta, `isPrerelease` debe ser `true` e `isDraft` debe ser `false`. Comprueba
que están los instaladores de las tres plataformas, las firmas y `latest.json`.
La beta debe quedar fuera de la última estable que muestra el segundo comando.

Revisa [Novedades](https://launcher.tflives.com/novedades) y el artículo de la
versión. La consulta de GitHub se revalida cada quince minutos: normalmente basta
con esperar ese intervalo. La web se alimenta de la API y no necesita un commit
ni un despliegue propio por cada release del launcher.

Añade la versión al [índice documental](../releases/README.md), utilizando la
fecha real de `publishedAt` en UTC. Si escribiste un informe en `docs/changes/`,
anota en él la versión y el tag que finalmente lo incorporaron. Los registros
`provenance.json` y `publication-verification.json` conservan la revisión histórica
de noviembre de 2026; no hay que regenerarlos para publicar una versión nueva.

## Pasar una beta a estable

Prepara una nueva versión sin sufijo, por ejemplo `0.15.4`, sobre el código
revisado. Repite el proceso con:

- Versión `0.15.4` en los cuatro archivos, usando `bun run release:version 0.15.4`.
- Entrada `'0.15.4'` en `src/lib/releaseNotes.ts`.
- Archivo `docs/releases/v0.15.4.md`, canal Estable y notas de los cambios incluidos.
- Commit validado por CI y tag nuevo `v0.15.4`.

La estable necesita sus propios binarios con su número de versión. Cambiar solo
la casilla de prerelease de una beta conservaría la versión beta dentro de sus
instaladores. Consulta [CANAL_BETA.md](../CANAL_BETA.md) para el comportamiento
del actualizador.

## Corregir las notas de una release existente

Si cambia únicamente la explicación, edita su Markdown en `docs/releases/` y
guarda una copia del texto anterior. Para copias versionadas puedes utilizar
`docs/history/release-notes/`, con versión y fecha en el nombre. Guarda el cambio
revisado mediante un commit y súbelo a `main`.

**Subir el Markdown a `main` no cambia una release ya publicada.** Para actualizar
su descripción, por ejemplo:

```sh
bun scripts/release-notes-body.mjs v0.15.4-beta.1
gh release edit v0.15.4-beta.1 -R TFLivesStudio/TFLClient --notes-file docs/releases/v0.15.4-beta.1.md
```

Comprueba después el texto en GitHub y en la web. Este comando edita las notas;
no requiere crear otro tag ni reconstruir los instaladores. Si cambia el programa,
prepara una nueva versión en vez de reutilizar un tag o sustituir los binarios
que ya recibieron los usuarios.

## Si algo falla

| Situación                        | Qué comprobar                                                                                                               |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Las versiones no coinciden       | Ejecuta el script de versión y revisa los cuatro archivos; comprueba también el tag.                                        |
| Falta la entrada in-app          | La clave de `RELEASE_NOTES` debe coincidir exactamente con la versión, sin `v`.                                             |
| Las notas salen incompletas      | Comprueba el nombre de `docs/releases/<tag>.md` y la salida del generador.                                                  |
| CI está en rojo                  | Revisa el paso fallido antes de publicar. El workflow de instaladores depende de que pase.                                  |
| Falta un instalador              | Comprueba todos los jobs de Build TFL Client; ver una release creada no significa que hayan terminado las tres plataformas. |
| La web muestra el texto anterior | Confirma que se editó el cuerpo en GitHub, no solo el archivo del repo, y espera la revalidación.                           |
| La beta aparece como estable     | Revisa el tag y `isPrerelease` antes de anunciarla o distribuirla.                                                          |

El [informe de validación](release-notes-validation.md#resultado-de-ci-y-dependencia-pendiente)
documenta un bloqueo de auditoría detectado el 13 de noviembre de 2026 en
`quinn-udp 0.5.15`. Comprueba su estado actual en CI al preparar la próxima versión.
