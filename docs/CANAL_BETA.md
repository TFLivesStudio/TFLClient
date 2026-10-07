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

1. Versionar con sufijo: `bun run release:version 0.11.0-beta.1 --force`
   (`--force` solo hace falta si el número "baja" respecto al actual, por ejemplo
   al pasar de `0.11.0` a `0.11.0-beta.1`).
2. Agregar las notas en `src/lib/releaseNotes.ts` con **la versión completa como
   clave** (`'0.11.0-beta.1'`).
3. Commit y tag con el mismo nombre: `git tag v0.11.0-beta.1`.
4. El workflow de release (`build.yml`) detecta el guion del tag y publica el
   Release como **prerelease** (y un paso final lo fuerza, por si acaso).
5. **Verificar** que quedó como prerelease:
   `gh release view v0.11.0-beta.1 --json isPrerelease`. Si saliera `false`, hay
   que corregirlo YA (`gh release edit v0.11.0-beta.1 --prerelease`): sería la
   "latest" y se le ofrecería a todos los usuarios del canal estable.

## Cómo pasar una beta a estable

Versionar sin sufijo (`bun run release:version 0.11.0`), notas con la clave
`'0.11.0'` y tag `v0.11.0` (sin prerelease). Los usuarios de beta la reciben por
ser más alta que su beta; los estables, porque pasa a ser "latest".

## Limitaciones conocidas

- No hay "volver a la estable" desde una beta: hay que reinstalar la estable a
  mano o esperar una estable más nueva.
- Los instaladores de Linux (`.rpm`) con versión prerelease no se probaron antes
  de la primera beta: si el `.rpm` de una beta fallara al empaquetar, el Release
  beta queda incompleto, pero eso no afecta a nadie del canal estable.
