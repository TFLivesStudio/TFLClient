# Connecting clients

Open **Settings → Integrations**, enable MCP, choose a client, and copy the generated
configuration. Its command is the executable actually located beside the launcher or
inside its resource directory. Start that client's connection, then authorize its
new session in TFL Client. Start with **Read** only. The launcher must remain running;
its window may be hidden. Configurations contain no token or port.

## Codex CLI

Codex CLI 0.163.0 is installed in the development environment. Its MCP configuration
interface was inspected; inference-driven tool execution is not claimed certified.
Use the absolute path shown in Integrations:

```toml
# ~/.codex/config.toml
[mcp_servers.tflclient]
command = "/absolute/path/from/Integrations/tflclient-mcp"
startup_timeout_sec = 15
tool_timeout_sec = 45
```

Alternatively, `codex mcp add tflclient -- /absolute/path/to/tflclient-mcp` registers
stdio; `codex mcp get tflclient` inspects it. On Windows, quote the actual `.exe` path.
The UI emits escaped paths valid for TOML. Do not paste a macOS/Linux path on Windows.

## Claude Code

Project `.mcp.json` configuration:

```json
{ "mcpServers": { "tflclient": { "command": "/actual/helper/path", "args": [] } } }
```

Or use `claude mcp add --transport stdio tflclient -- /actual/helper/path` according to
your installed version. Respect project trust dialogs; never approve a session you
did not initiate. Claude Code is not installed here; this client is not certified.

## OpenCode

```json
{ "mcp": { "tflclient": { "type": "local", "command": ["/actual/helper/path"], "enabled": true } } }
```

Put this in the client's supported `opencode.json` scope. OpenCode is not installed
here and has not been certified. Inspect your version's configuration reference.

## ChatGPT/cloud

A cloud client cannot reach this desktop's local stdio. No HTTPS/OAuth bridge has
been implemented, exposed or verified. Do not forward the relay port or publish its
credential. Local Codex support does not imply remote ChatGPT support.

## Platforms and troubleshooting

- Windows: installer bundles `tflclient-mcp.exe`; use the exact UI path and JSON/TOML
  escaping. No Python/Node/runtime is needed by users.
- macOS: helper is bundled inside the app; move the whole app together, not the helper
  alone. Preserve the application's existing signing/ad-hoc policy.
- Linux: helper follows the package's external-binary layout; AppImage launches must
  use the helper in the mounted app, not an invented stable mount path. Regenerate the
  snippet after restarting an AppImage if its mount directory changes.
- Missing helper: use an installer with MCP bundled, or build it using the documented
  script. Development binaries can reside beside the app in `target/debug`.
- Disabled/closed launcher: enable MCP in the running launcher, then reconnect.
- Approval required: find the matching fresh session in Integrations and grant read.
- Permission/scope denied: review the local grant; never retry with another UUID to
  bypass it. Revoking and reconnecting creates a fresh authorization opportunity.
- Launcher restart/idle expiry: reconnect; sessions are intentionally ephemeral.
- Critical action: request a plan, inspect it locally, approve it, then execute its ID
  before expiry. State changes require a fresh plan.

The external TypeScript SDK acceptance client is a real stdio MCP consumer and is
used to verify discovery, data access, mutations and policy rejection. It is not an
AI inference test of Codex/Claude/OpenCode.
