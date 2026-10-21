# Browser performance observations

Production frontend builds: baseline `930c905` and refresh implementation `2839a28`. Chromium 154.0.8037.57, 1280×720, identical offline IPC fixture, three fresh pages per revision/profile on the same cloud host. No build or browser regression suite ran alongside the recorded comparison. The dev/preview servers remained resident.

These are **browser measurements**, not native startup, TTI, process RSS, GPU memory or whole-process CPU usage. `ready` is navigation through the visible Play button, including Playwright overhead. `heap` is CDP JSHeapUsedSize, without forced GC. `idle task` is CDP TaskDuration delta during a 1-second window; it excludes compositor/GPU work. Settings/detail are click-to-visible-panel times, not frame smoothness. With three samples, differences are observations rather than statistically established improvements.

## Medians

| Revision | Quality    | Ready ms | JS heap MiB | Idle task ms / 1s | Settings ms | Detail ms |
| -------- | ---------- | -------- | ----------- | ----------------- | ----------- | --------- |
| baseline | Lite       | 299      | 5.44        | 0.74              | 50          | 72        |
| baseline | Balanced   | 285      | 5.43        | 2.23              | 67          | 78        |
| baseline | Experience | 299      | 5.44        | 2.31              | 62          | 74        |
| refresh  | Lite       | 325      | 4.64        | 0.73              | 47          | 98        |
| refresh  | Balanced   | 269      | 4.67        | 2.24              | 74          | 136       |
| refresh  | Experience | 326      | 4.67        | 2.28              | 80          | 154       |

The refresh's offline Home uses roughly 0.8 MiB less JS heap in this probe. Lite shows the lowest idle main-thread task cost. Startup is mixed. Detail navigation regresses by 26–80 ms in these samples, with Experience's median increasing from 74 to 154 ms. The new detail renders additional artwork controls and the shared transitions; this probe does not isolate the cause. These sub-200-ms observations do not justify claiming that navigation improved. Validate and profile on native WebKit and representative hardware before acceptance.

Raw samples: [performance.json](../../artifacts/visual-refresh/performance.json). Reproduce with `tests/visual/performance.mjs`. The fixture has an offline mascot, so this table does **not** measure skin rendering. The separate synthetic-skin software-renderer probe is below.

## Synthetic-skin viewer probe after density correction

Production refresh revision `b9a9968`, same Chromium and 1280×720 at device scale 1. One sample per profile/revision with the same synthetic 64×64 skin. Chromium used its software WebGL renderer with `--enable-unsafe-swiftshader`; results are a resource-lifecycle probe, not representative hardware performance. The renderer now caps pixel ratio at the smaller of the physical display density and the quality profile's ceiling.

| Revision | Quality    | Context creations | rAF callbacks in sample | JS heap MiB | Main-thread task ms | Actual sample ms |
| -------- | ---------- | ----------------- | ----------------------- | ----------- | ------------------- | ---------------- |
| baseline | Lite       | 1                 | 61                      | 7.32        | 567.96              | 1005.15          |
| baseline | Balanced   | 1                 | 61                      | 9.28        | 598.04              | 1003.76          |
| baseline | Experience | 1                 | 60                      | 9.32        | 582.81              | 1003.64          |
| refresh  | Lite       | 0                 | 0                       | 4.9         | 1.11                | 1004.25          |
| refresh  | Balanced   | 1                 | 61                      | 8.82        | 581.96              | 1005.77          |
| refresh  | Experience | 1                 | 61                      | 9.19        | 567.46              | 1012.02          |

The old Lite initializes a WebGL context and runs animation callbacks; the refresh Lite creates none and remains static. After capping render density, the refresh's Balanced/Experience main-thread cost is close to the baseline in this single software-renderer sample. A preceding exploratory run before the correction showed 774 ms (Balanced) and 1036 ms (Experience) of task work versus 582 ms and 567 ms after the correction; those exploratory samples were not controlled repeated benchmarks, so no percentage speedup is claimed. The raw final probe records the actual elapsed interval.

Callbacks are instrumented requestAnimationFrame callbacks across the page, **not rendered frames or FPS**. JS heap is not GPU memory. For a no-WebGL/static/manual/disposal regression, use `tests/visual/lifecycle.mjs`; for these observations use `tests/visual/viewer-performance.mjs`. Raw final results: [viewer-performance.json](../../artifacts/visual-refresh/viewer-performance.json).

## Certification limits

Native process memory/CPU, cold launcher startup, GPU memory, frame pacing, theme-switch latency, long-session leak behavior and Windows/macOS measurements remain unverified. No claim of native performance improvement or 60 FPS is made. Chromium's headless software renderer is not a substitute for a real GPU/WebKit measurement.
