# Animated Backgrounds 2.0

Five curated originals replace the weakest flat presentation without changing IDs:

| ID     | Direction         | Composition and movement                                                                                                      |
| ------ | ----------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| forest | Living Overworld  | Alpine valley, turquoise river, layered cloud and mist, warm right-hand sunlight; restrained fog and reflected-light movement |
| embers | Nether Emberlands | Basalt arches and lava canyon, deep warm atmosphere; ember-colored particles and slow heat light; 720p loop                   |
| void   | The Quiet Void    | Floating islands, fractured eclipse and purple depth; sparse drifting haze                                                    |
| aurora | Aurora Drift      | Luminous curtains, snowy mountains and mirrored lake; organic optional shader, 720p loop                                      |
| pixel  | Pixel Overworld   | Deliberately block-built landscape, torch-lit cabin, bridge, sunset and river; warm light pulses and crisp detailed art       |

Night Sky and Crystal Hollow keep their existing SVG art. They are **not** claimed as
new paintings. The original SVGs remain fallbacks if a new image cannot decode.
Custom backgrounds, imported videos, IDs and per-instance inheritance are preserved.

Art is approximately 1672×941 WebP with lightweight 448px thumbnails. Only the
selected scene loads its full asset; gallery cards do not mount players or shaders.
Ambient motion uses browser compositor transform/opacity timelines with quantized
profile FPS; it adds no JavaScript RAF. The existing shared clock remains for shader
and character rendering. Atmospheric layers are unmounted whenever scene policy
suspends motion. Lite renders one static image, zero automatic video/WebGL.
Experience's optional shader has a 960×540 upper resolution bound and scene-specific
fog/curtain light, replacing the earlier single sine-wave overlay. Context loss
stops drawing and reveals the poster; selecting another scene recreates resources.

The background veil was adjusted to reveal the new compositions while preserving
quiet space under text. Modern and Pixel settings use the same authentic scene
posters; the Pixel Overworld composition is original block art, not a CSS pixelation
filter on a photograph.
