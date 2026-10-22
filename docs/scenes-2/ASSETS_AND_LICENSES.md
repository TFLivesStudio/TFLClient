# Asset provenance and dependency decisions

All five `static/scenes/cinematic/*.webp` images and their thumbnails are original
images generated specifically for this project using OpenAI image generation on
2026-10-22 UTC. Prompts requested original landscapes without text, logos, characters,
copied game textures or third-party assets. No image was downloaded from Minecraft,
a wallpaper site or a stock library. They are distributed under this repository's
GPL-3.0-only terms to the extent applicable. There is no third-party attribution
requirement for these newly generated outputs. Existing resource licenses remain.

The aurora/embers MP4 and WebM files are locally encoded silent loops of those
original images, with a small periodic camera offset and runtime atmospheric effects.
They are not claimed to be photogrammetry or a simulated flowing world. Encoding:
1280×720, 24 fps, H.264 yuv420p faststart and VP9 fallback, 12 seconds, no audio.
Generation-source PNGs remain in the execution workspace; optimized deliverables
are committed. `CinematicAtmosphere.svelte` and shader expressions are original code.

| Candidate                       | Verified package metadata                   |                            Unpacked size | Decision                                                                                        |
| ------------------------------- | ------------------------------------------- | ---------------------------------------: | ----------------------------------------------------------------------------------------------- |
| rmcp 3.6.0                      | official Rust SDK, Apache-2.0, Rust >=1.88  |               separate native dependency | pinned; server + stdio features only; GPLv3-compatible                                          |
| OGL 1.0.11                      | Unlicense                                   |                            423,239 bytes | not added: existing small WebGL adapter already owns lifecycle; no second abstraction needed    |
| PixiJS 8.14.0                   | MIT                                         |                         63,316,030 bytes | not added: no measurable advantage for this curated raster/atmosphere renderer                  |
| @tsparticles/slim 4.4.0         | MIT                                         |                            916,091 bytes | already installed; do not add another particle engine; reuse existing low-count scene particles |
| Three.js / Vanta                | existing skin viewer has Three transitively | not measured for a new background bundle | no new world renderer; avoid extra contexts and Lite costs                                      |
| ReactBits / Svelte Bits effects | no particular component copied or licensed  |                             none shipped | reference concept only; no React runtime, no copied effect code/assets                          |

Metadata was read with `cargo info rmcp@3.6.0` and `npm view <package>@<version>
license dist.unpackedSize`. Unpacked package size is not a compressed bundle
measurement. OGL/tsParticles/Pixi have no new code included by this change.

The MCP packaging overlay also bundles original license files for rmcp and its newly
introduced code-generation/schema dependencies under `mcp-licenses/`. Source copies
are retained at `src-tauri/resources/mcp-licenses/`, including Apache-2.0 and MIT terms.
The resource mapping merges with the existing icon and mp-guard mappings.
