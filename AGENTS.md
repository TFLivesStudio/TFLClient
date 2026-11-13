# TFL Client — organización y publicaciones

## Documentación

- Usa [docs/README.md](docs/README.md) como índice. Conserva en la raíz los manifiestos,
  configuraciones, lockfiles, README, licencia, estas instrucciones y el instalador
  público de Linux. No añadas changelogs de sesiones a la raíz.
- Los informes antiguos de `docs/history/legacy/` conservan sus bytes y fechas
  como material histórico. Los informes técnicos nuevos van a `docs/changes/`,
  con fecha y tema en el nombre; no se añaden a la raíz ni al archivo original.
- Las pruebas de navegador siguen generando `artifacts/`, ignorado por Git. La
  evidencia histórica versionada está en `docs/qa/`; los instaladores heredados
  que ya existían están en `archives/installers/legacy/`.
- `install-linux.sh` conserva su ruta pública: el README y usuarios externos la usan.

## Notas de versión

- Para preparar una publicación sigue la [guía práctica](docs/maintainers/publishing.md).
- Sigue [la guía editorial](docs/maintainers/release-notes.md). El resumen inicial y
  la voz deben ser consistentes en GitHub y en la web oficial, `launcher.tflives.com`.
- Las notas completas son `docs/releases/<tag>.md`. Las breves de
  `src/lib/releaseNotes.ts` siguen alimentando el panel del launcher; no elimines ese
  flujo. `scripts/release-notes-body.mjs` prefiere el Markdown revisado y conserva
  las notas in-app como alternativa compatible.
- Atribuye cada cambio al tag que lo contiene. Contrasta commits, diff y archivos;
  no transfieras funciones entre ramas ni describas una prueba simulada como una
  ejecución nativa. Conserva requisitos y limitaciones documentados.
- No anuncies funciones, mediciones ni compatibilidad universal sin evidencia.
  Mantén Selection comprensible para jugadores, sin detallar su arquitectura en
  las notas públicas que también muestra la web.
- Editar una descripción publicada no requiere crear un tag ni reconstruir los
  instaladores. Preserva assets, firmas, metadatos del updater y canales.
- Verifica el generador con `node --test tests/unit/release-notes.test.mjs` y
  `node scripts/release-notes-body.mjs <tag>`; ejecuta las comprobaciones de CI
  pertinentes si cambias scripts o código.
