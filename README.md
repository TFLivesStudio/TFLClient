# sv

Everything you need to build a Svelte project, powered by [`sv`](https://github.com/sveltejs/cli).

## Limitaciones conocidas

- **macOS:** subir una imagen propia (ícono de instancia, fondo de pantalla) no está disponible por ahora — el diálogo nativo de archivos crashea el launcher en esta plataforma (firma ad-hoc sin certificado de Apple Developer). El resto de las funciones no se ve afectado, y esto no aplica a Windows ni Linux. Ver `CHANGELOG_macos-dialog-crash-java26.txt` para el detalle técnico.

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
