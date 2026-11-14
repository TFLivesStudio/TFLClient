# Visual experience refinement — v0.15.4-beta.2

Base: published `v0.15.4-beta.1` (`c5a4df7`), plus its documentation follow-ups on
main. The beta base passed 31 unit cases and the 26-capture Chromium smoke suite
before implementation. This change is intended for a new official beta; earlier
tags and the stable channel remain untouched.

## Implementation

- One shell-owned global ambient scene; archived per-instance scenes can be
  recovered into a draft without overwriting their saved values. Schema 1 gets
  an untouched backup; unknown future schemas are preserved.
- 25 owner-approved original banners, optimized WebP derivatives and thumbnails,
  deterministic version/family/custom resolution. See
  [provenance and architecture](../experience/BACKGROUNDS_AND_BANNERS.md).
- Pixel shallow relief and 4px spacing; shared sidebar edge collapse control,
  scrolling instance list, compact composition and portaled keyboard tooltips.
  See [sidebar](../experience/SIDEBAR.md).
- Official identity is hydrated from Selection provenance, independent of its
  loader. Existing locked profiles remain protected even if their lock becomes
  invalid. Managed UI has release/channel, update/verify/repair, personal worlds,
  resource packs, screenshots and backups. Shaders require explicit capability;
  the exact legacy signed 1.8.9 manifest is recognized for existing Lucerna support.
- Backend protects indirect mod/modpack mutation, validates pinned signed
  manifests and actual managed file hashes, propagates quarantine errors, and
  serializes launch/install against content mutation. Repair retains user data.
- Optional signed icon contract bounds SHA-256, bytes, PNG size and immutable
  release source; existing manifests use the packaged logo. The separate
  TFLClientSelection branch changes schema/preflight/docs/tests only; signed
  manifests, catalogs, signatures and game JARs are not changed.
- Frontal character controller with bounded head tracking, greeting, manual
  rotation/reset, event blending and rigid cape spring/damping. Static profiles
  have no continuous character RAF. See [character](../experience/CHARACTER.md).

## Validation scope

Unit, Svelte, lint, production build and Rust formatting run locally. Native
compilation, clippy, Rust tests and audit require CI: this machine lacks MSVC.
Browser suites use the production frontend with isolated IPC, actual packaged
artwork and synthetic skin/cape textures. They do not establish that Minecraft
was launched, that a Microsoft account logged in, or that native GPU/RSS are
improved. Published client internals remain pinned.

Final commands, observed performance, CI and release URLs are recorded after
the corresponding checks complete. Browser evidence is generated under
`artifacts/` and uploaded by CI; it is not committed as source history.
