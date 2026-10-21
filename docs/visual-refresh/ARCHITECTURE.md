# Visual Refresh 2 — architecture and integration

Base: main 930c905 (0.12.3). Worktree: /workspace/TFLClient-visual-refresh. Branch: feat/visual-refresh-2. Original checkout clean and untouched. Parallel Content Discovery: 67a67ac, seven commits ahead; inspected, never merged or modified.

## Audit

The existing shell has Home/instance detail but no central library. Home fetches real recent client instances, activity and favorite servers. Preserve it. Detail owns all critical instance/server actions: preserve handlers and expose the same panels through secondary navigation. Settings and skin studio are existing overlays. Existing local Inter and licensed Monocraft assets are sufficient; do not copy third-party artwork.

The old SkinViewer imports Three.js eagerly, creates WebGL in Lite and does not pause rendering on visibility. Appearance is mirrored from root attributes; extend it with a central capability resolver and a disposable runtime. Existing quality settings remain authoritative. The legacy token scale includes sub-11px text; replace it across surfaces.

## Boundaries

- `lib/visual/`: validated versioned UI preferences, capability policy, motion and lifecycle. No game/auth/download logic.
- `lib/components/refresh/`: library, discovery landing, customization, notifications and shared presentation.
- `styles/refresh.css`: semantic Modern/Pixel geometry, materials, density, responsive and motion policy.
- Existing APIs, auth, file validation, launch safeguards and Tauri capabilities remain intact.
- New strings live in `lib/i18n/refresh.ts` using the existing locale state, avoiding conflicts in shared es/en dictionaries.
- Instance visual metadata is keyed by UUID in a separate local preference document. It never renames/moves instance folders or modifies backend instance schemas.

## Content Discovery seam

Do not edit ContentManager, ModpacksPanel, content APIs, backend search, shared types, ui/index or ModalHost. The global Discover landing links to existing TFL Selection and real instance content tabs; the parallel branch can mount ContentExplorerView inside that layout. No fake search results, pagination, installer or duplicate query state. `uiState` remains untouched; navigation lives in a new store. Resolve only explicit imports if integrating later.

## Headless decision

Retain the working custom Svelte primitives and native controls. Centralize modal focus/keyboard behavior with a reusable action; no runtime dependency needed for a wholesale migration. Bits UI is preferred for a future complex combobox/menu replacement when that behavior is required, not to restyle existing controls. Separate interaction from CSS theme. Do not add React or Tailwind.

## Validation contract

Use actual frontend rendering with a test-only Tauri IPC fixture for browser regression/screenshots; identify those images as fixture-backed, not native game execution. Native Rust tests and compile remain separate evidence. Never infer real Microsoft login, game launch, online installs, or cross-platform behavior from mocks. Measure browser timing/heap as browser observations only. Native WebKit GUI startup is blocked by system helper locations in this environment.

External research: Laby reference HTTPS request blocked by the proxy (403). Superdesign CLI runs but has no authenticated session; direct design/implementation remains with the current model as requested. No external design model is used.
