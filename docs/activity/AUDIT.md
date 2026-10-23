# Activity Core audit — before implementation

Base: `v0.14.0-beta.6`, `47c0832`, clean new worktree `feat/activity-center`.
No applicable AGENTS.md exists in the repository or workspace parents. Read README,
package scripts, Tauri config/ACL/build manifest, architecture and beta/release docs.
The latest published prerelease is beta.6. Authentication PR #9 and Content Discovery
PR #4 are separate, unmerged work and are not included. The owner explicitly requests
publishing this beta, overriding the attachment's default review-only handoff.

| Capability                    | State   | Evidence                                                                                                                                    |
| ----------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| Session history               | EXISTS  | `services/activity_log.rs`, `commands/activity.rs`, `RecentActivityPanel.svelte`: last 20 played sessions, distinct from operations         |
| Download display              | PARTIAL | `core/event_bus.rs`, `state/downloadState.svelte.ts`: transient UI progress keyed by human names; no authoritative task lifecycle           |
| Content domain                | EXISTS  | `services/content/install.rs`, `download.rs`: shared provider service, integrity checks, per-instance mutation lock                         |
| MCP install                   | EXISTS  | `services/mcp_tools.rs::tfl_install_content`: existing scoped/policy-checked call to same install service                                   |
| MCP queue/job metadata        | PARTIAL | `services/mcp.rs`, `mcp_policy.rs::Job`: scoped protocol jobs, idempotency, queued cancellation; preserve these contracts and policy checks |
| Shared task authority/history | MISSING | No cross-UI/system domain task authority, revisioned recovery or persisted operation history found                                          |
| Global Activity UI            | MISSING | `AppShell.svelte`, `Sidebar/Sidebar.svelte`, `visual/navigation.svelte.ts` have Home/Library/Discover                                       |
| IPC security                  | EXISTS  | generated main-only command set in `build.rs`; log window limited; no additional capabilities needed                                        |
| Other async operations        | EXISTS  | launcher, Java/runtime, updater, world backups; migrate later, preserve existing behavior now                                               |

Plan: add one bounded Rust task authority, typed safe fields, monotonic snapshot
revisions, debounced atomic persistence and interrupted recovery. Track the existing
content-install service (including MCP source) and actual download bytes. Installation
is explicitly noncancellable because dependency replacement is not transactional.
Add main-window query/cancel contract, main-only invalidation and subscribe-before-
snapshot frontend projection. Use existing theme tokens, no new dependencies or
canvas generation: this is an incremental product implementation, not a new visual
direction. Baseline check/test logs are captured separately before code changes.

MCP job metadata remains the existing scoped transport envelope. Once it executes a
content installation it references the common domain activity ID in its result. No
parallel MCP executor or permission path is introduced. Queue cancellation before
domain execution remains with MCP; started installations remain noncancellable.
