# TFL Client MCP — contract 1

`AI client → rmcp 3.6.0 stdio sidecar → authenticated private relay → policy → shared Rust services → Tauri events`.

The helper is a native Rust executable, not a chatbot or click driver. Only stdout
JSON-RPC is used; diagnostics go to stderr. The official `rmcp` SDK negotiates MCP
versions, tools and resources. HTTP, OAuth, tunnels, remote access and cloud ChatGPT
are deliberately **not implemented or advertised**. No socket exists by default.

## Processes and authorization

The launcher owns the data and policy. When explicitly enabled it binds an ephemeral
IPv4 loopback TCP relay (a private framed protocol, **not HTTP**). A 256-bit random
transport secret and port are written into `~/.tflclient/settings/mcp/endpoint.json`.
Unix permissions: directory 0700, file 0600. Windows: protected owner-rights DACL;
failure to secure the endpoint fails activation. Nothing sensitive goes in process
arguments or URLs. A transport credential authenticates the user's helper, but does
**not** grant tools access: every connection is a new unapproved session.

The main Tauri window alone can grant permissions or approve plans. Sessions are
limited to eight, expire after one hour, and are revoked immediately on disconnect,
disable or launcher restart. Reported client names are unverified display data.
The helper reports an actionable error when the launcher is closed. After a restart
or five-minute IPC idle timeout, reconnect the client and approve the new session.
Hidden/minimized launcher windows do not affect business operations or start visuals.
No second launcher/headless core is spawned by the helper.

## Permissions and transactions

Read/configure/operate/critical scopes are independent. Instance UUID allowlists apply
before dispatch and again for jobs. Restricted clients cannot create or duplicate
instances outside that allowlist. Critical actions use a short-lived exact parameter
plan, local approval, state fingerprint recheck and one-shot consumption. MCP mutations
are serialized globally (a conservative stronger limit than per-instance locks).
Content mutations share per-instance locks between Tauri and MCP. Other existing UI operations retain their service boundaries; MCP never relies on an LLM
honoring annotations. Running instances cannot have managed files mutated via MCP.

Jobs expose queued/running/succeeded/failed/cancelled states, owned results and bounded
retention. Idempotency keys coalesce identical calls and reject changed payloads.
Only queued jobs can be cancelled. Running downloads finish through the existing
installer even if the client disconnects; arbitrary abort is not safe. The UI receives
normal service progress events plus `mcp-library-changed`; no second CPU sampler is
created. Job APIs do not invent byte progress or percentages when existing events
cannot be reliably attributed to a particular session.

## Preference authority

A separate revisioned `visual-authority.json` stores visual, scene and console records.
Missing records are migrated once from localStorage; existing authoritative records
win over stale caches. LocalStorage remains a cache for fast rendering and old-window
compatibility. UI and MCP writes use compare-and-swap revisions. Conflicts reload the
persisted record and are not silently retried over another writer's changes. Native
events update open views. Theme/quality remain in the existing SettingsManager; account,
OAuth, Java paths, environment variables and JVM strings are never exposed by MCP.

## Packaging

`bun scripts/build-mcp-sidecar.mjs [--review|--debug]` builds the pinned SDK and copies
an OS/target-suffixed helper into `src-tauri/binaries/`. `tauri.mcp.conf.json` includes
it as an external binary. Both official and review workflows build and bundle the
helper; the review workflow keeps updater artifacts disabled. The standard release
signing and updater public key are unchanged. The UI resolves the helper's actual
installed path instead of fabricating a user-specific location.

`TFL_DATA_DIR` is an optional absolute existing portable/test profile directory shared
by host and sidecar. It is not an MCP tool parameter. The feature-gated native harness
is never built into distributed installers and requires a marked isolated profile.
