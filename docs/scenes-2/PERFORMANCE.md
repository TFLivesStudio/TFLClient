# Performance policy and measurement limits

Five full posters plus five thumbnails total approximately 1.4 MiB. The four silent
720p video encodings are each under 500 KiB. Only selected full resources are loaded.
A decoded 1672×941 RGBA image alone can require about 6 MiB; encoded size is not RAM.
The user's earlier ~10 MB observation is not treated as a certified benchmark.

Lite has no atmospheric subscriptions, no shader and no automatically mounted video.
Balanced uses restrained compositor transform atmosphere (max 24 fps) and video only when
explicitly enabled. Experience respects the configured FPS, caps shader backing
resolution at 960×540 and DPR at the actual screen density/profile cap. All paths
respect visibility, focus, modal and game policies. The shared clock stops when its
last subscriber is removed. Video decoding follows its encoded 24 fps, separately
from the procedural-effect FPS control.

Chromium screenshots and lifecycle counters are browser evidence, not measurements
of native WebView2/WKWebView/WebKitGTK codecs or hardware GPU energy. The optional
shader uses software rendering in this cloud host; no hardware FPS claims are made.
Native launcher/sidecar process memory and response latency must be reported
separately, alongside exact scenario and sampling method. Results follow in the
measurement artifacts; unsupported native-platform checks remain explicitly pending.

The independent helper's debug RSS measured ~6.6 MiB idle and ~6.8 MiB after a small
query session in the isolated native harness. These values exclude launcher, browser
and GPU memory. Exact raw samples and methodology: ../mcp/evidence/native-performance.json.

## Comparable browser observations

[evidence/performance.json](evidence/performance.json) compares the frozen 0.14.0-beta.1
production frontend with this refresh using the same Chromium fixture, three runs per
profile. This observes JS heap, main-thread tasks and UI response, not resident/GPU
memory. [evidence/scenes.json](evidence/scenes.json) adds 30 observations: five selected
scenes × three profiles × two revisions, with optional shader/video turned off. Lite
has zero mounted video, shader or atmospheric animation layer in every observation.
The richer scene atmosphere adds some main-thread work in Balanced/Experience; exact
raw values are retained rather than claiming a universal speed improvement.

A measured optimization replaced per-frame CSS variable writes with transform/opacity
compositor timelines. In the same software-rendered browser probe, the median new
Balanced scene task time fell from ~73.6 to ~17.7 ms per one-second window. The frozen
baseline measured ~10.6 ms in the second run. Experience measured ~66.1 vs ~38.4 ms;
Lite ~2.0 vs ~2.2 ms. These are main-thread task observations under software rendering,
not full-process CPU percentages or a hardware performance guarantee. The prior
measurement is preserved in evidence/scenes-before-compositor.json.
