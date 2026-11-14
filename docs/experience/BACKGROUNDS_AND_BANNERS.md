# Global ambience and version artwork

From v0.15.4-beta.2, `AppShell` owns one `LivingScene`, using only `scenes.global`.
Navigation and selection do not replace its canvas. Heroes, detail headers and
library cards use `resolveBanner`: explicit user artwork, exact Minecraft
version, declared family, packaged fallback. A fallback makes no exact-version
claim. Library cards use smaller thumbnails. Artwork uses cover cropping without
changing its aspect ratio; scene layers remain beneath content and legibility
veils.

`tfl-scenes-v1` remains the storage key, with schema version 2. Version 1 is
backed up unchanged to `tfl-scenes-v1-before-global-backgrounds`. Its instance
scene map is retained as an archive and can be loaded into the global draft in
Settings before explicit Apply. Unknown future schemas are not overwritten by
migration. Existing custom artwork takes priority over version resolution.

The owner supplied and approved `minecraft-key-art-25-imagenes-COMPLETO.zip`.
SHA-256: `c1b086ce06ff159ea5b91fe676c7c0852fa8d2b458e7c2cc5b0faf7b8cb83827`.
Its 25 original images are preserved byte-for-byte in `static/banners/originals`.
Optimized WebP images (maximum width 1440, quality 86) and 480px thumbnails
(quality 80) were produced with Sharp 0.35.5. `static/banners/provenance.json`
records original hashes and dimensions. `banner-registry.ts` explicitly maps the
owner's filenames; the highest supplied patch in each family is its fallback.
No generated or externally downloaded replacement artwork is used. Owner
approval is recorded; it is not a separate copyright license grant.

`scripts/approved-banner-archive.mjs` validates ZIP entries before any writing:
bounded counts, expanded bytes and compression ratio, CRC, format/dimensions,
paths, Windows reserved names/ADS, collisions, links and overlapping ranges.
It rejects encrypted, split and ZIP64 archives. Runtime never extracts a ZIP.

Chromium tests cover migration/recovery, one persistent canvas through routes,
exact mapping, explicit customization, image decoding, and zero scene canvas in
Lite. They use simulated IPC and do not measure native launcher RSS or GPU load.
