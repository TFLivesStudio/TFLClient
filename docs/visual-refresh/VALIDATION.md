# Visual Refresh 2 — validation and remaining certification

The implementation spans the whole shell and its major surfaces. This is a review candidate, **not a completed native/multiplatform certification**. Browser screenshots render the actual Svelte application with the explicitly isolated IPC fixture in `tests/visual/fixture.mjs`. They are not mockup designs, but their accounts, installations, server status and activity are simulated. The skin lifecycle test uses a synthetic texture. No fixture is imported into the application.

## Implementation coverage

| Phase | Delivered                                                                                                                  | Limits                                                                                                                                                                            |
| ----- | -------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A     | Audit, isolated worktree, Content Discovery boundary, architecture                                                         | External Laby reference blocked by proxy; no external model used                                                                                                                  |
| B     | Semantic tokens, Modern/Pixel, readable/classic typography, materials, contrast-aware custom accent, shared modal behavior | Existing native/custom primitives retained; no new component dependency                                                                                                           |
| C     | Home/Library/Discover navigation, compact sidebar, titlebar breadcrumb/search, command palette, responsive layout          | Native window controls not exercised by Chromium                                                                                                                                  |
| D     | Recent real client hero, artwork, character/fallback, launch safeguards, optional secondary panels                         | Automatic artwork uses existing screenshots/wallpapers; no remote artwork acquisition                                                                                             |
| E     | List/grid, search, favorites, collections keyed by UUID, detail hero and technical side navigation                         | Metadata stays in this webview's localStorage; no filesystem moves or cloud sync                                                                                                  |
| F     | Server status/connection dashboard and entry points to preserved console/plugins/files/worlds/backups                      | Technical panels retain their existing business UI with shared tokens; no fabricated telemetry                                                                                    |
| G     | Editorial Discover landing, TFL Selection and contextual instance content entry points                                     | Parallel ContentExplorer is intentionally not merged/reimplemented                                                                                                                |
| H     | Large settings overlay, side navigation, style/material/presets, home panels, per-instance artwork                         | Instance artwork selects existing bundled images; global custom-image import remains the existing validated flow                                                                  |
| I     | Account switcher, skin/cape studio, static Lite skin, optional manual 3D, lazy viewer, stale account-load protection       | Real Microsoft login, upload and cape APIs require native/manual verification                                                                                                     |
| J     | Shared focus/Escape/inert behavior, styled forms/dialogs/onboarding, event-driven toasts and bounded session history       | Existing install progress retained; no artificial progress or success states                                                                                                      |
| K     | Bounded reveal/stagger/parallax, CSS ambience, launch-dependent character motion, visibility/overlay disposal              | No shared-element morphs or bespoke multi-stage installation cinematics; renderer/performance limits take priority                                                                |
| L     | Static checks/build, Rust workspace tests/clippy, browser regression/screenshots, reproducible browser timing harness      | Native Windows/macOS/Linux GUI acceptance, real online flows and hardware performance targets remain open; PR #5 published; installer build results are tracked in GitHub Actions |

## Commands

Environment setup in this cloud workspace uses `. /workspace/.tfl-tools/env.sh`. Toolchain pins and dependency manifests are unchanged. The remote build identified new TLS advisories; Cargo.lock now pins rustls-webpki 0.103.16 and webpki-roots/webpki-root-certs 1.0.10. The audit gate remains enabled. Rust checks reuse `/workspace/TFLClient/target` as a build cache; original source checkout is unchanged.

```sh
bun install --frozen-lockfile
bun run check
bun run lint
bun run build
bun test tests/unit
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --lib --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked
```

The final browser production build was additionally exercised with the same smoke suite. The separate viewer probe records WebGL creations and callback/task costs for the density-capped renderer.

The five policy tests are included in the existing frontend CI job. Node 24 was also used to verify the same tests through `node --test`.

The browser harness needs an external Playwright installation and Chromium, keeping the application dependency graph and the parallel branch's lockfiles untouched. With Playwright installed locally, omit `PLAYWRIGHT_MODULE`; otherwise set it to the absolute path to its `index.mjs`. `CHROMIUM_PATH` is optional when Playwright's managed browser is installed.

```sh
bun run dev -- --host 127.0.0.1 --port 1422
PLAYWRIGHT_MODULE=/path/to/playwright/index.mjs CHROMIUM_PATH=/usr/bin/chromium node tests/visual/smoke.mjs
PLAYWRIGHT_MODULE=/path/to/playwright/index.mjs CHROMIUM_PATH=/usr/bin/chromium node tests/visual/lifecycle.mjs
```

For comparative timing, serve **production builds** of base `930c905` at port 4173 and this branch at 4174, then run `tests/visual/performance.mjs` with the same environment variables. `BASELINE_URL` and `VISUAL_URL` override those endpoints. Results are browser observations, not native startup/working-set/CPU benchmarks.

## Automated evidence

- Unit tests cover profile precedence, invalid persisted data/path rejection, bounded groups, preference round trip and custom-accent foreground.
- Browser regression covers six style/profile combinations, 800×600, 1100×700, 1280×720, 1366×768 at DPR 2 and 1920×1080, Light/English and OLED/reduced motion, navigation, list/grid, favorites persistence, context menu, dialog keyboard containment/Escape, error notifications and studio fallback.
- Compact workspace regression also covers backups, mods, console, settings and studio at 800×600, and adding/removing collections without removing instances.
- Lifecycle regression confirms that Lite does not request skinview3d or create a WebGL context, optional manual activation works, covered Home disposes/recreates its renderer, and returning to static removes the canvas.
- `artifacts/visual-refresh/` contains labeled screenshots and raw results. Baseline is the original production frontend under the same fixture, not a design mockup.
- Svelte's six pre-existing warnings remain: initial-value capture in ContextMenu/InstanceDetail and focusable code elements in onboarding/settings. No added warning suppressions.
- Rust clippy's pre-existing warnings remain; CI itself does not use `-D warnings`.

## Native manual acceptance still required

1. On Windows, macOS and Linux: minimum/maximized/high-DPI window, titlebar controls, drag regions, focus return, screen-reader announcements and keyboard-only navigation.
2. Offline account and Microsoft account: add/switch/logout; actual skin upload/reset/cape selection; corrupt texture/network failure; no old account's skin after switching.
3. Launch vanilla/Fabric/Forge clients; trigger multi-instance warning; close/crash and inspect logs; ensure launch feedback follows actual events.
4. Create/import/duplicate/rename/export/delete instances and servers; verify no lost settings or changed folder paths.
5. Install/update/remove mods, shaders, packs and modpacks; cancellation/failure/retry; TFL Selection network flows. Repeat after integrating Content Discovery.
6. Start/stop server, inspect live console, LAN/tunnel connection, file editing, world backup/restore and destructive confirmations.
7. Persist style/font/accent/density/material/artwork/groups across a native restart; test storage failure messaging and switching between presets and manual overrides.
8. Measure native startup/input readiness, process memory/CPU, 3D/ambience cost and frame timing per quality tier on representative hardware. Confirm minimize/restore and repeated overlay/navigation cycles do not leak GPU contexts/listeners.

The Linux GUI could not be certified here: WebKit looks for helpers under `/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1`, whereas the cloud dependencies are installed under the local sysroot. Compilation and Rust tests succeed, but that does not establish native rendering parity. No CSP, filesystem scope, Tauri capability, authentication or launch API was relaxed to work around this.

## Remote build follow-up

PR [#5](https://github.com/TFLivesStudio/TFLClient/pull/5) is published. The first installer run stopped at cargo audit because RUSTSEC-2026-0381/0382/0383 were published after the original local validation. The lockfile-only patch upgrades the three affected TLS/certificate crates; no ignore rule or CI gate was weakened. This is the sole new overlap with Content Discovery's modified paths and must be preserved when that branch is integrated. Final installer runs and downloadable artifacts are linked from the PR.
