# Microsoft Authentication 2.0 — audit before implementation

Baseline inspected: `47c0832` (`v0.14.0-beta.6`), 2026-10-22.
The latest beta is on `feat/scenes-image-pack`, seven commits beyond main's
`ae24e91`. Work branches from that beta; PR targets that branch to avoid
reintroducing scene changes or modifying Content Discovery #4.

## Findings

- `core/mod.rs` aliases launchwerk's CubicLauncher Client ID. No TFL-owned ID or
  evidence of Minecraft approval was supplied. Production must retain that ID.
- launchwerk has S256 generation, authorize URL and code exchange, but the UI only
  uses device code. No browser callback owner, state validation, single-use
  transaction, cancellation or concurrency control exists. Redirects share the
  ambiguous `http://localhost/callback` constant.
- Device code and polling intervals are accepted back from JS. Cancel only changes
  the UI; the blocking worker can still finish and add an account afterwards.
- Token persistence is AES-GCM with a key derived from public account UUID plus a
  source-code constant. This protects neither confidentiality against a local file
  reader nor tokens copied with profiles. Migration must read this format only.
- MinecraftUser token fields use serde(skip), but derived Debug exposes them.
  `save_tokens` failures are logged and success still returned. Deletion errors
  are also ignored. Refresh uses the current global ID without provenance,
  swallows failures, and does not update the settings user's in-memory tokens.
- Xbox parsing indexes the first claim without checking it exists. Service errors
  include raw HTTP response bodies. Network calls lack an operation-wide timeout.
- Main-window capability has the command set; log windows have a limited set.
  No remote auth window capability exists. MCP has no auth/account mutation tools;
  this boundary must remain intact.
- Offline and Yggdrasil flows must remain supported. Public account metadata and
  instancing should survive credential migration and reauthorization.

## Design decisions

One Rust-owned operation controls browser/device flow, deadline, cancellation,
phase and public outcome. Secrets and device polling credentials never return to
JS. PKCE and state stay in Rust; external callbacks bind only IPv4 loopback and
validate request host, path, parameters and state. Browser auth and device code
reuse the existing Microsoft → Xbox → XSTS → Minecraft pipeline.

A compile-time, validated development override is distinct from an explicit
production activation. No new production ID, secrets, tags or releases. Sessions
record their issuing ID; changing it requires reauthorization, not silent fallback.

OS credentials replace the old format: one versioned record per account/provider,
read-back verification before removing legacy files. Store failures are visible;
users can explicitly choose memory-only sessions. No silent file fallback.

Microsoft native-app documentation distinguishes `nativeclient` (embedded) and
`http://localhost` (system browser). RFC 8252 recommends external user agents.
Remote embedded windows must have zero Tauri capabilities, no injected scripts,
no opener privilege and no account/cookie interception. Unsupported navigation,
construction or platform restrictions fall back visibly to the system browser;
server rejection must never switch OAuth identity. Legacy registration support
cannot be invented: device code remains available with the existing identity.

## References consulted

- https://learn.microsoft.com/en-us/entra/identity-platform/scenario-desktop-app-configuration
- https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow
- https://learn.microsoft.com/en-us/entra/msal/dotnet/acquiring-tokens/using-web-browsers
- https://www.rfc-editor.org/rfc/rfc8252
- https://help.minecraft.net/hc/en-us/articles/16254801392141-Minecraft-Java-Edition-Launcher-API-Migration-FAQ
- https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html
- https://docs.rs/keyring/latest/keyring/

Fetched documentation is implementation guidance, not evidence that an unspecified
TFL application has been approved or that a real account was authenticated here.
