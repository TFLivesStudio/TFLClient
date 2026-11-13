# TFL Client

[![TFL Client — sitio web oficial](https://launcher.tflives.com/images/social/tfl-client-es.png)](https://launcher.tflives.com)

El launcher de **TFLives** para Minecraft, con un núcleo nativo escrito en **Rust** y una interfaz en **Svelte**.

## Web oficial · [launcher.tflives.com](https://launcher.tflives.com)

La web oficial es el punto de entrada para conocer TFL Client, consultar sus características, elegir una descarga y encontrar ayuda. Este repositorio contiene el código fuente del launcher.

**[Visitar la web oficial](https://launcher.tflives.com)** · [Descargas](https://launcher.tflives.com/descargas) · [Características](https://launcher.tflives.com/caracteristicas) · [TFL Selection](https://launcher.tflives.com/selection)

[Novedades](https://launcher.tflives.com/novedades) · [Ayuda](https://launcher.tflives.com/ayuda) · [Discord](https://discord.gg/tflives) · [Website in English](https://launcher.tflives.com/en)

---

## Descargas directas y Linux

**[⬇ Descargar para Windows](https://github.com/TFLivesStudio/TFLClient/releases/latest/download/TFLClient-Windows-Setup.exe)** &nbsp;·&nbsp; **[⬇ Descargar para macOS](https://github.com/TFLivesStudio/TFLClient/releases/latest/download/TFLClient-macOS.dmg)** &nbsp;·&nbsp; **[⬇ Descargar para Linux](https://github.com/TFLivesStudio/TFLClient/releases/latest/download/TFLClient-Linux.AppImage)**

Estos links siempre apuntan a la última versión publicada.

**Linux:** para no lidiar con `apt`/`dnf`/permisos a mano, corré esto en una terminal — detecta tu sistema solo e instala el paquete que corresponda (deb, rpm, o AppImage con ícono en el menú de aplicaciones):

```sh
curl -fsSL https://raw.githubusercontent.com/TFLivesStudio/TFLClient/main/install-linux.sh | bash
```

Si tu sistema es una instalación mínima (sin `curl`), instalalo primero: `sudo apt install -y curl` (o `sudo dnf install -y curl` en Fedora).

## Limitaciones conocidas

- **Visor 3D de Skin y capa en macOS:** en investigación — el panel de Skin y capa (Ajustes de cuenta → ícono junto a Ajustes) no renderiza correctamente en macOS por ahora (el mismo panel anda bien en Windows y Linux). Subir/restaurar skin y elegir capa siguen funcionando igual en las 3 plataformas — es solo la vista previa 3D la que falla en Mac.

## Desarrollo de la interfaz

El frontend utiliza SvelteKit y Vite. Instala las dependencias y arranca el servidor de desarrollo:

```sh
npm install
npm run dev
```

Para abrirlo directamente en el navegador, utiliza `npm run dev -- --open`.

## Compilación de la interfaz

```sh
npm run build
npm run preview
```

Estos comandos compilan y previsualizan la interfaz web. Para revisar cómo se generan los instaladores nativos del launcher, consulta el [workflow de compilación](.github/workflows/build.yml).

## Documentación y publicaciones

- [Índice de documentación](docs/README.md).
- [Notas completas de cada versión](docs/releases/README.md).
- [Guía para preparar notas de publicación](docs/maintainers/release-notes.md).
- [Historial técnico conservado](docs/history/README.md).
