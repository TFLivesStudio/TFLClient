# Canal de versiones beta

TFL Client tiene dos canales de actualización. Cada usuario elige el suyo en
**Ajustes → Sistema → Actualizaciones → "Versiones beta"**.

- **Estable** (por defecto): recibe solo las versiones normales (`vX.Y.Z`).
- **Beta**: recibe la versión **más alta** que haya publicada, sea beta o estable
  (si sale una estable más nueva que la beta que tenés, también la recibís).

Activar las betas muestra **siempre** un aviso de que pueden tener errores
(cada vez que se intente, no solo la primera); desactivarlas es directo. El
launcher no vuelve solo a la versión estable al desactivarlas: se queda en la
beta instalada hasta que salga una estable más nueva (y esto está dicho en el
aviso).

## Cómo funciona

- Una beta es una **prerelease de semver**: `0.11.0-beta.1`, `0.11.0-beta.2`…
  La app la muestra como `0.11.0 (beta)` (`src/lib/version.ts`). Para semver,
  `0.11.0-beta.2` < `0.11.0`, así que la estable siempre reemplaza a su beta.
- El canal estable usa el endpoint de `tauri.conf.json`
  (`releases/latest/download/latest.json`). GitHub **no** considera "latest" a las
  releases marcadas como _prerelease_: por eso una beta nunca le llega a nadie
  que no la pidió.
- El canal beta (`src-tauri/src/commands/updater.rs`) lee la lista de releases de
  la API de GitHub, elige la de mayor versión que tenga `latest.json` y usa ese
  manifest. Si la API falla (sin red, límite de pedidos) o el manifest de esa
  release no sirve para la plataforma, **cae al canal estable**: quien tiene
  betas nunca se queda sin las actualizaciones normales.
- La firma del paquete se verifica siempre contra la clave pública de
  `tauri.conf.json`, venga del canal que venga.

## Cómo sacar una beta

La [guía de publicación](maintainers/publishing.md) explica el recorrido completo,
con comandos y ejemplos. Para una beta:

1. Elige una versión nueva con sufijo, como `0.15.4-beta.1`, y actualiza los cuatro
   archivos con `bun run release:version 0.15.4-beta.1`. El número es un ejemplo;
   comprueba las versiones existentes antes de usarlo.
2. Escribe las notas completas en `docs/releases/v0.15.4-beta.1.md` y añade la
   entrada breve `'0.15.4-beta.1'` a `src/lib/releaseNotes.ts`.
3. Sube el commit de preparación y comprueba que su CI pasa. Crea el tag
   correspondiente sobre ese mismo commit y súbelo para activar la publicación.
4. El workflow detecta el guion del tag y publica la release como prerelease.
   Espera a que terminen los instaladores de las tres plataformas.
5. Verifica `isPrerelease` con
   `gh release view v0.15.4-beta.1 --json isPrerelease -R TFLivesStudio/TFLClient`.
   Debe ser `true`. Si no lo es, corrige el estado con
   `gh release edit v0.15.4-beta.1 --prerelease -R TFLivesStudio/TFLClient`
   antes de distribuirla, y revisa el workflow que la publicó.

## Cómo pasar una beta a estable

Prepara una versión nueva sin sufijo, con sus notas completas y breves y un tag
propio. Por ejemplo: versión `0.15.4`, entrada `'0.15.4'`, archivo
`docs/releases/v0.15.4.md` y tag `v0.15.4`. El proceso construye nuevos binarios
con la versión estable; no consiste en cambiar la casilla de prerelease del
instalador beta existente.

Los usuarios del canal beta la reciben si es una versión superior a la suya;
los del canal estable la reciben cuando es la última estable disponible.

## Limitaciones conocidas

- No hay "volver a la estable" desde una beta: hay que reinstalar la estable a
  mano o esperar una estable más nueva.
- Los instaladores de Linux (`.rpm`) con versión prerelease no se probaron antes
  de la primera beta: si el `.rpm` de una beta fallara al empaquetar, el Release
  beta queda incompleto, pero eso no afecta a nadie del canal estable.
