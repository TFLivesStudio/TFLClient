# Validación de la revisión de publicaciones

Fecha: 13 de noviembre de 2026. Repositorio: `TFLivesStudio/TFLClient`.

## Publicaciones

- 85 descripciones revisadas y publicadas, desde `v0.8.0` hasta
  `v0.15.3-beta.7`, incluidas las betas existentes.
- Contraste con el código de cada tag, su historial alcanzable, los changelogs
  técnicos y las notas in-app. Las fuentes están en
  [provenance.json](../releases/provenance.json).
- Una consulta nueva a la API confirmó que los 85 cuerpos publicados coinciden
  exactamente con sus archivos Markdown.
- Se conservaron identificadores, tags, nombres, fechas de publicación, canal,
  destino y estado de las releases. Los identificadores, nombres, tamaños,
  hashes disponibles, fechas y URLs de sus assets permanecieron iguales.
- [Registro de verificación](../releases/publication-verification.json).

La revisión distingue las ramas paralelas: el centro Actividades de
`v0.14.0-beta.7.1` no se atribuye a `v0.14.1`. Tampoco se describe la autenticación
de `v0.15.3-beta.6` como si ya estuviera integrada en el `main` anterior. La edición
de las notas no fusiona esas ramas ni modifica los binarios existentes.

## Organización del repositorio

- 46 documentos históricos trasladados a `docs/history/legacy/`.
- 52 archivos de evidencia trasladados a `docs/qa/visual-refresh/`.
- 6 instaladores heredados trasladados a `archives/installers/legacy/`.
- Los 104 traslados conservan el mismo objeto Git: contenido idéntico al original.
- La raíz pasa de 61 a 16 archivos; conserva los manifiestos, lockfiles,
  configuraciones, licencia, README, instrucciones e instalador público de Linux.
- Se actualizaron las referencias documentales. Los tests continúan generando
  resultados en `artifacts/`, ignorado por Git.

## Comprobaciones

| Comprobación                                  | Resultado                                              |
| --------------------------------------------- | ------------------------------------------------------ |
| `npm run check`                               | 0 errores; 7 avisos en código existente                |
| `npm run lint`                                | Prettier y ESLint correctos                            |
| `node --test tests/unit/*.test.mjs`           | 24 pruebas correctas                                   |
| `npm run build`                               | Interfaz de producción compilada                       |
| `node scripts/release-version.mjs --check`    | Versiones alineadas en `0.15.3-beta.5`, sin cambiarlas |
| Generador para los 85 archivos revisados      | Salida idéntica al Markdown                            |
| Notas completas, alternativa in-app y errores | Cubiertos por 6 pruebas del generador                  |
| Enlaces nuevos de documentación               | Destinos comprobados en disco o índice Git             |
| `git diff --cached --check`                   | Correcto                                               |

El generador conserva la alternativa basada en notas in-app para no romper la
preparación de futuras versiones. Un Markdown presente pero incompleto provoca
un error. No se reconstruyeron instaladores nativos: los cambios de código nativo
se limitan a referencias en comentarios y la lógica del launcher permanece intacta.
