# TFL Client

**[⬇ Descargar para Windows](https://github.com/TFLivesStudio/TFLClient/releases/latest/download/TFLClient-Windows-Setup.exe)** &nbsp;·&nbsp; **[⬇ Descargar para macOS](https://github.com/TFLivesStudio/TFLClient/releases/latest/download/TFLClient-macOS.dmg)** &nbsp;·&nbsp; **[⬇ Descargar para Linux](https://github.com/TFLivesStudio/TFLClient/releases/latest/download/TFLClient-Linux.AppImage)**

Estos links siempre apuntan a la última versión publicada.

**Linux:** para no lidiar con `apt`/`dnf`/permisos a mano, corré esto en una terminal — detecta tu sistema solo e instala el paquete que corresponda (deb, rpm, o AppImage con ícono en el menú de aplicaciones):

```sh
curl -fsSL https://raw.githubusercontent.com/TFLivesStudio/TFLClient/main/install-linux.sh | bash
```

Si tu sistema es una instalación mínima (sin `curl`), instalalo primero: `sudo apt install -y curl` (o `sudo dnf install -y curl` en Fedora).

---

Everything you need to build a Svelte project, powered by [`sv`](https://github.com/sveltejs/cli).

## Limitaciones conocidas

- **Diálogos de archivo nativos (subir ícono/wallpaper propio, agregar mods por archivo):** el launcher tiene un modo "Automático" (default) y uno "Manual", elegible al primer inicio y editable en cualquier momento desde Ajustes. En Automático estas funciones quedan desactivadas — no hace falta usarlas, todo se instala solo. En Manual quedan habilitadas, pero pueden crashear el launcher en macOS (el diálogo nativo de archivos rompe con la firma ad-hoc del build, sin certificado de Apple Developer). Ver `CHANGELOG_macos-dialog-crash-java26.txt` y `CHANGELOG_modo-auto-manual-dialogos.txt` para el detalle técnico.

## Creating a project

If you're seeing this, you've probably already done this step. Congrats!

```sh
# create a new project
npx sv create my-app
```

To recreate this project with the same configuration:

```sh
# recreate this project
bun x sv@0.17.0 create --template minimal --types ts --add sveltekit-adapter="adapter:static" prettier eslint --install bun .
```

## Developing

Once you've created a project and installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```sh
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of your app:

```sh
npm run build
```

You can preview the production build with `npm run preview`.

> To deploy your app, you may need to install an [adapter](https://svelte.dev/docs/kit/adapters) for your target environment.
