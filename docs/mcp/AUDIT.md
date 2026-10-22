# Audit — 2026-10-22 UTC

Base: main bd8fd883; latest beta v0.14.0-beta.1; stable v0.13.2.
Isolated worktree: TFLClient-mcp, branch feat/tfl-client-mcp-scenes-2.
Open PR #4 (content discovery) is unrelated and will remain untouched.

| Existing capability               | Real Rust service                                   | Proposed MCP surface                             | Permission            | Verification                           |
| --------------------------------- | --------------------------------------------------- | ------------------------------------------------ | --------------------- | -------------------------------------- |
| Instances/UUID/config             | instance_manager                                    | list/get/create/rename/memory                    | read/operate          | UUID, sandbox, concurrent requests     |
| Content/dependencies/hashes       | content::ContentService                             | search/detail/versions/installed/install/updates | read/operate          | same install engine, bounded schemas   |
| Launch/session metrics            | launcher/server_process, commands::instance sampler | sessions/stats/start/stop                        | read/operate/critical | no second sampler, local stop approval |
| Worlds/backups                    | world_backup, commands::world_backups               | list/create/restore/delete                       | read/operate/critical | exact object, one-shot approval        |
| Java                              | java_manager                                        | status                                           | read                  | omit personal paths                    |
| Logs/diagnostics                  | crash_diagnosis + managed logs                      | bounded redacted diagnostics                     | read                  | traversal/size/secret tests            |
| Visual/scene/console localStorage | Svelte preference modules                           | central revisioned preference store              | configure             | migration + conflict tests + UI events |
| Accounts/OAuth                    | SettingsManager                                     | NEVER exposed                                    | none                  | allowlisted outputs only               |

Current scene engine: seven three-layer SVG landscapes, two 480x270 video loops;
shared clock, static Lite, optional shader and bounded managed local video imports.
Preserve IDs and lifecycle. Improve original art assets rather than loading another
heavy rendering framework. Third-party candidates/licenses are recorded separately.

Baseline commands and results are captured in TESTING.md as checks finish.
No tags, releases, merges or unrelated branches are authorized for this mission.
