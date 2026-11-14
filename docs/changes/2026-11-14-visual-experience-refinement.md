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

## Results before publication

- 43 unit tests; Svelte check: 0 errors, 5 existing warnings; Prettier/ESLint,
  production build and version/tag consistency passed locally.
- Production Chromium: beta2 (12 captures), smoke (26), compact workspaces,
  lifecycle, experience (85 evidence entries), MCP/scenes (22 checks), auth
  (13 UI checks), Pixel (116 assertions), living settings (4 cases), landscapes
  (3 scenes × 3 quality tiers and 12 disposal cycles), backgrounds/banners
  (11 checks and 50 decoded resources), sidebar (216 observations), character
  (5 lifecycle/profile scenarios), Selection (8 client/style cases).
- Rust CI on the implementation commit passed formatting, clippy, 252 workspace
  library tests, native MCP acceptance and cargo audit (748 dependency entries).
  [Implementation CI](https://github.com/TFLivesStudio/TFLClient/actions/runs/43594031721).
- The isolated distribution contract passed both PR and branch CI and merged as
  [Selection PR 11](https://github.com/TFLivesStudio/TFLClientSelection/pull/11).

## Observed browser performance

Chromium 159.0.8154.6 on Windows with SwiftShader and simulated IPC. Two
one-second intervals per character configuration after warmup; means below are
descriptive observations, not statistical estimates or native CPU percentages.
The Home scene is Midnight Forest, scope window, 24 FPS/subtle, no shader.
Work includes the whole page; cape cost cannot be isolated from timing noise.

| Revisión | Calidad    | Capa sintética | Trabajo del hilo principal (ms/1 s) | Heap JS (MiB) | Draw calls/1 s |
| -------- | ---------- | -------------- | ----------------------------------- | ------------- | -------------- |
| beta1    | Lite       | No             | 0.6                                 | 5.4           | 0              |
| beta1    | Lite       | Sí             | 0.6                                 | 5.4           | 0              |
| beta1    | Balanced   | No             | 161.9                               | 8.8           | 380            |
| beta1    | Balanced   | Sí             | 174.2                               | 10.0          | 431            |
| beta1    | Experience | No             | 161.6                               | 9.0           | 380            |
| beta1    | Experience | Sí             | 171.0                               | 10.1          | 420            |
| beta2    | Lite       | No             | 0.5                                 | 5.5           | 0              |
| beta2    | Lite       | Sí             | 0.6                                 | 5.6           | 0              |
| beta2    | Balanced   | No             | 230.6                               | 9.0           | 361            |
| beta2    | Balanced   | Sí             | 255.7                               | 9.1           | 368            |
| beta2    | Experience | No             | 199.7                               | 8.9           | 371            |
| beta2    | Experience | Sí             | 148.1                               | 9.5           | 420            |

Lite created zero WebGL contexts and zero draw calls. Reduced motion and running
game fixtures also stopped continuous character RAF. The rigid cape adds a small
spring controller and the existing cape geometry; no native GPU efficiency or
stable game FPS claim is made. Ready times are retained in raw JSON but include
browser/cache effects and are not a cold-start benchmark.

Sidebar manual collapse observations: 27; median
264.8 ms, range
32.8–383.7 ms,
including transition settlement and browser automation overhead. They do not
represent input latency measured on a native launcher.

Scene-only comparison: 30 configurations/observations across five presets and
three qualities in each revision, retained as raw JSON. This single sample per
configuration does not establish an overall performance improvement.

Reproduce with Node 24/Bun 1.4.2 and Playwright 1.57.0 against the production
build: run the scripts in .github/workflows/ci.yml; set VISUAL_URL,
BASELINE_URL, BASELINE_VERSION=0.15.4-beta.1,
APP_VERSION=0.15.4-beta.2, then run
tests/visual/scenes-performance.mjs and
tests/visual/refinement-performance.mjs. Artifacts are ignored by Git and
uploaded by CI. No real Minecraft session, OS GPU/RSS measurement or physical
device render certification was performed for this refinement.

## Publication

Published [v0.15.4-beta.2](https://github.com/TFLivesStudio/TFLClient/releases/tag/v0.15.4-beta.2)
at 2026-11-14T12:57:03Z, release ID 445296970, prerelease=true/draft=false.
[Official build 43595919726](https://github.com/TFLivesStudio/TFLClient/actions/runs/43595919726)
passed all six jobs, including Windows, macOS and Linux packaging.

The annotated tag points to the fully checked commit a8157b93d7774b26ef8e0db1318f76519ea9bf1d.
Its tree ea3de70937cdc097c63eec140a9debfa46831aca exactly matches the
squash merge bd8c63952ba38de27c7e8d5261398e02b313b28b on main. Final
[CI](https://github.com/TFLivesStudio/TFLClient/actions/runs/43594635030)
and [Windows/macOS auth checks](https://github.com/TFLivesStudio/TFLClient/actions/runs/43594634959)
passed before tagging. Subsequent documentation records publication evidence;
the published source and tag remain immutable.

Downloaded updater payloads for macOS arm64, Windows x64 and Linux x64
(AppImage, deb, rpm) matched GitHub SHA-256 digests and sizes. Node crypto
verified Ed25519 signatures over BLAKE2b-512 digests with the pinned updater
public key, the signed trusted comments, detached/latest.json consistency,
and rejection of altered data/comments. All five fixed installer aliases
were downloadable and matched their canonical asset digests. These are
updater signatures, not a claim of Apple notarization or Windows Authenticode.
The verification follows the primary
[rust-minisign format](https://github.com/jedisct1/rust-minisign/blob/master/src/lib.rs).

Release notes and updater notes matched the reviewed Markdown. The release
contains 17 assets. Stable v0.15.2 (ID 433706720), beta 1 (ID 445065636), their
notes, publication dates, asset identities/digests and existing tags were
verified unchanged. No signed Selection distribution was published.

After cache revalidation, the official [news index](https://launcher.tflives.com/novedades)
and [beta article](https://launcher.tflives.com/novedades/v0.15.4-beta.2)
returned HTTP 200 and contained the new beta and notes. No website deployment
or code change was needed.
