# Security model and boundaries

MCP is disabled on installation. Enabling it does not approve any client. The threat
model includes malicious MCP callers, hostile project descriptions/log lines, replayed
approvals, oversized requests and cross-instance/session access. It does not claim
protection against malware already executing as the same OS account with full access
to launcher binaries, memory and credentials. Same-account local filesystem changes
can race existing path-based services; MCP cannot create links and rejects linked trees,
but this is not a kernel filesystem capability sandbox.

| Boundary       | Enforcement                                                                                                                                                                                                |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Network        | IPv4 loopback only, ephemeral port, random 256-bit credential; no HTTP/tunnel/UPnP                                                                                                                         |
| Secret file    | 0700/0600 Unix; owner-only protected DACL on Windows, fail closed                                                                                                                                          |
| Session        | local approval, 1h TTL, 8 clients, revocation, no persistent blanket grant                                                                                                                                 |
| Request        | 64 KiB framed stdio/relay input, strict shared JSON schemas, 120 requests/min/session                                                                                                                      |
| Response       | 1 MiB maximum, bounded pagination, 400 lines max, log tail capped at 256 KiB                                                                                                                               |
| Execution      | permission + exact UUID + instance scope checked in host; 8 queued/running jobs globally                                                                                                                   |
| Critical       | 120s plan, local decision, exact original parameters and managed state fingerprint, one use                                                                                                                |
| Files          | no generic read/write, exact UUID lookup, managed-root check, symlink-tree rejection, 50k-entry ceiling                                                                                                    |
| Content        | existing compatibility/dependency/conflict engine; HTTPS Modrinth CDN allowlist, restricted redirects, 512 MiB/file streamed to disk, required SHA1, size check, incremental hash + temporary write/rename |
| Sensitive data | no account/env/JVM endpoints; recursive secret-key filtering; sensitive log lines replaced, home path redacted                                                                                             |
| Audit          | 300 local decisions; no raw args/tokens/logs, survives restarts, local clear action                                                                                                                        |

Fingerprints cover managed manifest, running state, relative names, sizes and mtimes.
They detect ordinary intervening UI/file changes, not hostile same-account attempts to
forge filesystem metadata. Local approvals expire rather than authorizing future
unrelated actions. Idempotency responses return the original job, never execute a
consumed critical plan twice.

Log redaction is defense in depth: generic secrets without recognizable markers can
never be guaranteed detectable. Do not ask a model to ingest unnecessary logs. Project
text, log lines and filenames are untrusted **data**, never agent instructions. No
server-console command, shell, arbitrary URL, arbitrary Tauri invocation, account/OAuth
operation, launcher update or arbitrary filesystem operation is offered.

Windows DACL behavior, Apple signing and native webview codecs require platform build
and manual runtime verification; see TESTING.md. Do not call remote/cloud support or an
unverified client certified. Critical operations must remain disabled for a client
unless the user explicitly grants that scope and approves the exact pending action.

MCP backup creation cannot prune old backups: at the existing ten-backup limit it
fails and requires an explicit separately approved deletion. This prevents using
repeated noncritical backup creation to bypass critical delete permissions. The
backup service serializes retention decisions across UI/MCP. Approved restoration
still creates its safety backup using the launcher's documented retention policy.
