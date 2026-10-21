# Capturas y mediciones de revisión

Todas las imágenes se capturaron de la aplicación Svelte ejecutándose en Chromium. **Tauri IPC está simulado** por `tests/visual/fixture.mjs`: cuentas, instancias, estadísticas y servidores son datos de prueba. Las capturas no certifican ejecución nativa ni lanzamiento real de Minecraft. Las skins de las pruebas de ciclo de vida son una textura sintética de 64×64.

| Vista | Evidencia |
| --- | --- |
| Base original 930c905 | [baseline.png](baseline.png) |
| Home Modern Experience | [home-modern-experience.png](home-modern-experience.png) |
| Home Modern Lite | [home-modern-lite.png](home-modern-lite.png) |
| Home Pixel Experience | [home-pixel-experience.png](home-pixel-experience.png) |
| Light / English | [home-light-en.png](home-light-en.png) |
| OLED / reduced motion | [home-oled-reduced.png](home-oled-reduced.png) |
| Biblioteca | [library.png](library.png) / [library-grid.png](library-grid.png) |
| Detalle técnico | [detail-1366x768.png](detail-1366x768.png) |
| Servidor | [server-dashboard.png](server-dashboard.png) |
| Explorar | [discover.png](discover.png) |
| Settings | [settings.png](settings.png) / [settings-pixel.png](settings-pixel.png) |
| Estudio | [skin-studio-no-skin.png](skin-studio-no-skin.png) / [skin-studio-lite.png](skin-studio-lite.png) |
| Creación / paleta | [create-dialog.png](create-dialog.png) / [command-palette.png](command-palette.png) |
| Error real del bus simulado | [notification-error.png](notification-error.png) |
| Mínimo 800×600 | [home-800x600.png](home-800x600.png) / [detail-800x600.png](detail-800x600.png) |

El resto de las imágenes cubre Balanced y tamaños 1100×700, 1920×1080 y DPR 2. `smoke-results.json` enumera las capturas del último recorrido. `performance.json` conserva las muestras sin sustituirlas por cifras inventadas. Interpretación y límites en `docs/visual-refresh/PERFORMANCE.md`.

Capturas adicionales de los espacios técnicos mínimos: [Backups](backups-800x600.png), [Mods](mods-800x600.png), [Consola](server-console-800x600.png), [Settings](settings-800x600.png) y [Estudio](studio-800x600.png).

`viewer-performance.json` mide el visor con textura sintética y renderizado por software, tras limitar su densidad. No equivale a FPS ni a un benchmark nativo de GPU.
