# Draft PR: Visual Refresh 2 — Modern/Pixel y espacios de trabajo

Base: `main` (`930c905`). Rama: `feat/visual-refresh-2`.

La navegación anterior concentraba las instancias en la barra lateral y mezclaba los controles técnicos con la portada. Este cambio añade Biblioteca y Explorar, una Home centrada en la instancia reciente, detalle con navegación técnica lateral y dashboard de servidor. Settings pasa a un overlay amplio con navegación lateral; el estudio reúne cuentas, skins y capas.

Modern y Pixel comparten comportamiento, tokens y política de rendimiento. Lite muestra la skin estática y no importa ni crea WebGL automáticamente; Balanced y Experience limitan resolución y movimiento, y el visor se libera cuando queda oculto. Las preferencias visuales por UUID incluyen favoritos, colecciones y artwork, sin mover archivos. Los diálogos comparten foco/Escape y las notificaciones proceden de eventos reales, con historial limitado a 50 entradas por sesión.

## Alcance de integración

No se modifican backend, autenticación, lanzamiento, instaladores, permisos, CSP ni tipos compartidos. El build remoto detectó avisos TLS recientes: `Cargo.lock` actualiza únicamente `rustls-webpki` a 0.103.16 y `webpki-roots`/`webpki-root-certs` a 1.0.10. No se fusiona Content Discovery (`67a67ac`); el único archivo modificado compartido con sus siete commits auditados es ahora ese lockfile, por esta corrección puntual. Explorar ofrece una superficie editorial y puntos de entrada al contenido existente, preparada para integrar su explorador.

## Validación

- `bun run check`: sin errores; seis advertencias preexistentes.
- `bun run lint` y `bun run build`: aprobados.
- Cinco pruebas de política/persistencia/contraste: aprobadas.
- `cargo fmt`, `cargo check --workspace --locked`, `cargo test --workspace --locked` y clippy: aprobados, con advertencias preexistentes de Rust.
- Recorridos Chromium: 26 capturas, seis combinaciones de estilo/perfil, tamaños desde 800×600, Light/English, OLED/movimiento reducido, favoritos persistentes, menús, foco, notificaciones y estudio.
- Espacios técnicos a 800×600: backups, mods, consola, ajustes y estudio; colecciones sin eliminar instancias. Pruebas de política añadidas al CI existente.
- Ciclo de vida del personaje: Lite sin carga WebGL, activación manual, liberación al cubrir Home y recreación al volver, retorno a vista estática.

Las capturas muestran la aplicación real en Chromium **con IPC simulado**. No certifican login, lanzamiento de Minecraft, instalaciones o renderizado nativo. Ver [validación y límites](VALIDATION.md), [arquitectura](ARCHITECTURE.md) y [mediciones](PERFORMANCE.md).

## Pendientes antes de aprobar

Certificación nativa Windows/macOS/Linux, accesibilidad con lector de pantalla, flujos reales de cuentas/contenido/servidores y rendimiento en hardware representativo. No se implementaron morphs de elementos compartidos ni cinemáticas específicas de cada etapa de instalación. Los paneles técnicos conservan sus controles existentes y reciben los tokens comunes.

## Estado de publicación

Rama publicada: `feat/visual-refresh-2`. PR en borrador: [#5 — Visual Refresh 2](https://github.com/TFLivesStudio/TFLClient/pull/5).

Los instaladores de revisión se generan mediante `workflow_dispatch` de [Build TFL Client](https://github.com/TFLivesStudio/TFLClient/actions/workflows/build.yml), sobre esta rama y con sus compuertas de calidad. La matriz incluye Windows, Linux y macOS. El modo manual sube artifacts y no publica una release estable. Los resultados y enlaces de descarga se mantienen en el PR y en el run de Actions.

La certificación nativa/manual descrita arriba sigue pendiente hasta probar los instaladores en sus sistemas correspondientes.
