# Notas de versión para GitHub y la web

Las notas completas viven en `docs/releases/<tag>.md`. El workflow las publica
mediante `scripts/release-notes-body.mjs` y la web las consulta desde la API de
GitHub. No hay que mantener una segunda redacción para Novedades.

## Voz y estructura

Escribe en español claro y con nombres de controles reconocibles. Explica qué
cambia, dónde se encuentra y cuándo hace falta actuar. Evita exclamaciones,
superlativos, promesas universales y listas de cambios internos sin efecto
explicado. Una corrección pequeña puede tener una nota breve.

Empieza por un párrafo de resumen, sin título, imagen, lista ni aviso genérico.
Ese primer párrafo sirve para la tarjeta de Novedades. Después identifica el canal
y utiliza solo las secciones necesarias, en este orden:

1. `## Novedades`: funciones que se incorporan en esta versión.
2. `## Mejoras`: cambios en funciones que ya existían.
3. `## Correcciones`: fallo anterior y comportamiento corregido.
4. `## Mantenimiento`: dependencias o cambios internos relevantes.
5. `## Al actualizar`: pasos necesarios para el usuario, si los hay.
6. `## Compatibilidad y límites`: restricciones concretas documentadas.
7. `## Referencias`: comparación de código, fuente del tag y enlaces oficiales.

No rellenes secciones vacías ni escribas «sin errores conocidos» por falta de
información. Las betas se identifican como versiones de prueba. Las fechas
históricas corresponden a `published_at` en UTC; no se sustituyen por la fecha de
la revisión editorial. Al preparar una release aún sin publicar, no inventes una
fecha de publicación real.

## Comprobar los hechos

Consulta el tag y sus archivos, no solo el estado actual de `main`. Compara una
estable con la anterior estable que sea ancestro; para una beta, consulta la
anterior publicación de su línea de desarrollo. Las ramas paralelas pueden tener
funciones distintas incluso cuando una de ellas tiene una versión superior.

Usa los mensajes de commit como punto de entrada y confirma las afirmaciones en
el diff, los archivos afectados y las notas técnicas. Distingue implementación,
compilación, pruebas con datos simulados y pruebas nativas. No conviertas una de
esas evidencias en otra. Si el historial solo acredita una corrección concreta,
explica esa corrección y conserva la nota corta.

Las notas públicas de Selection describen la experiencia del jugador y los pasos
de uso, sin exponer su arquitectura. El núcleo del launcher es Rust y la interfaz
es Svelte; no describas toda la aplicación como escrita únicamente en Rust.

La revisión histórica de noviembre de 2026 conserva sus fuentes en
[provenance.json](../releases/provenance.json) y los textos previos en
[el historial](../history/release-notes-before-2026-11-13.json).

## Preparar una publicación nueva

1. Actualiza la versión con `bun run release:version <versión>`.
2. Añade las notas breves para el panel del launcher en `src/lib/releaseNotes.ts`.
3. Escribe `docs/releases/v<versión>.md` con el resumen y las secciones aplicables.
4. Comprueba el cuerpo que recibirá GitHub:

   ```sh
   node scripts/release-notes-body.mjs v<versión>
   node --test tests/unit/release-notes.test.mjs
   ```

5. Ejecuta las comprobaciones de versión y de CI correspondientes antes de crear
   el tag, siguiendo [RELEASE_SECURITY.md](../RELEASE_SECURITY.md).

El generador prefiere el Markdown completo. Si aún no existe, conserva el flujo
anterior: toma las notas in-app y genera un cuerpo con resumen, canal y cambios.
Un archivo Markdown existente pero incompleto provoca un error; no se sustituye
silenciosamente por una versión más pobre. El generador no cambia los textos del
launcher ni crea releases por sí mismo.

## Revisar una publicación existente

Utiliza `gh release edit <tag> --notes-file docs/releases/<tag>.md` en el repositorio
correcto. Antes, guarda el cuerpo anterior y comprueba que la release no haya
cambiado desde que se preparó la revisión. Después, vuelve a consultar GitHub y
compara el cuerpo publicado con el archivo revisado.

La edición se limita a las notas. No reemplaza instaladores, firmas, `latest.json`,
fechas de publicación, etiquetas ni estado de prerelease. El sitio revalida la
consulta de GitHub cada quince minutos; sus tarjetas deben utilizar el primer
párrafo como resumen y el detalle debe conservar el Markdown completo.
