# Reproducible verification

Baseline main `bd8fd883`: frontend check 0 errors / 7 existing warnings; 18 Node
behavior tests and 141 Rust library tests passed. Current local checks: 156 Rust library tests and 18 Node behavior tests passed;
frontend check has 0 errors / 7 existing warnings; production build passed.
The official external SDK discovered 35 tools and completed the native acceptance
flow, including a real Modrinth/Fabric API installation with hash validation.
Evidence: [native-acceptance.json](evidence/native-acceptance.json).
Simple info calls in that fixture session took approximately 3–8 ms (8 observations;
not a universal latency guarantee). Browser regression: 85 evidence entries plus
22 new integration/scene checks. Native UI rendering was not exercised by the
windowless Tauri acceptance harness.

## Native MCP acceptance (isolated, no personal accounts)

Build the native Tauri test host and sidecar:

```bash
cargo build --locked -p tflclient --example mcp_host --features mcp-integration-tests
cargo build --locked -p tfl-mcp --bin tflclient-mcp
```

Create an empty absolute directory, add a `TEST_PROFILE` marker, and set
`TFL_DATA_DIR` to that directory for **both** processes. Start
`target/debug/examples/mcp_host` in a desktop session (or Xvfb). The harness has no
webview window; it uses the actual Tauri runtime, native events and Rust services.
It only consumes local test-control files when explicitly feature-built; no such
control path exists in distributed builds.

Install `@modelcontextprotocol/sdk@1.29.0` in a separate test-only Node project.
Set `MCP_SDK_ROOT` to its `dist/esm` directory, `MCP_BINARY` to the actual sidecar,
and run `node tests/mcp/client.mjs`. It verifies initialization, tools discovery,
unapproved/approved read, read-only mutation rejection, preference persistence and
native event, revision conflict, real instance create/RAM/world/ZIP backup and exact approved restore, key
coalescing, two processes with independent scopes, local approval/denial/execution
and revocation. A marked throwaway instance is deleted only after local test approval.
Outputs are `acceptance.json` and audit records inside that isolated directory.

No Microsoft login, account tokens or personal game worlds are required. The SDK
client's successful test is not a claim that an AI model chose correct tools.

## Security and lifecycle

`cargo test --workspace --lib --locked` includes strict-schema, frame limits,
permission/scope, expiry/replay/denial/revocation, changed-state, secret redaction,
traversal, symlinks and download-origin tests. Existing instance launch and content
regressions remain covered by the project's existing tests.

Native Windows/macOS installer startup, owner-only Windows DACL behavior, Apple
notarization, Microsoft login, a real Minecraft process and codec support in all three
native webviews need manual validation on the corresponding systems. Do not replace
these checks with Chromium results. HTTP/cloud are unsupported, not untested features.

## Coverage decisions

MCP exposes safe services through a curated contract, not every Tauri command.
Modpacks are discoverable but cannot silently replace an existing instance. Arbitrary
file import/export paths, JVM/environment editing, account changes, new server
creation, server console commands and compound mass updates are not exposed in this
version. Server start/stop/stats, plugins and managed server-world listing are covered.
Jobs report real state and results; per-job byte attribution is not invented. Queued
cancellation is supported; running shared installers cannot be safely aborted.

## Native process observations

See [native-performance.json](evidence/native-performance.json): Linux **debug**,
windowless Tauri harness (constant test-control polling, no GPU/webview). Host RSS
was 95,632 KiB with MCP disabled and 95,700 KiB enabled without a client in this run.
The separate stdio helper used 6,808 KiB idle / 7,012 KiB after queries. Twelve simple
queries measured roughly 3.8–8.8 ms. Short CPU samples, debug symbols and test-harness
I/O prevent extrapolating these numbers to a production launcher UI or another OS.
The debug helper binary is 52 MB with symbols; packaged optimized size is measured
separately by the review build script. No claim of a 10 MB whole-launcher footprint.
