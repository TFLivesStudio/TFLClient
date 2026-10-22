# Authentication security boundaries

The existing OAuth identity stays enabled. A new identity is compile-time test
configuration behind `auth-test-client` and an explicit development acknowledgement.
No remote preference, MCP tool, command argument or downloaded content sets it.
There is no client secret. Approval of an unspecified TFL Client ID is not claimed.
The test build has a distinct application identifier, private webview data directory
and a profile rooted below `.tflclient-auth-tests/<Client ID>/.tflclient`, even when
the portable base directory is configured. Its credential provider namespace includes
both the test lane and Client ID, so it cannot read, migrate or delete a production or
other-registration record for the same Minecraft UUID.

## Transaction owner

Rust owns one active operation, random state, S256 verifier, callback validation,
deadline and cancellation. JS receives public progress and the human device code,
not the device polling credential, callback code, Microsoft/Xbox/Minecraft tokens
or refresh tokens. Browser callbacks are single-use and matched against the exact
origin, port and path. Duplicate parameters, state mismatches, fragments, unexpected
hosts, methods, bodies and nonlocal listener access are rejected. State comparison
uses equal-length constant-time accumulation. Invalid probes do not consume a valid
transaction. A valid OAuth error consumes it. HTTP calls have finite timeouts and
bounded JSON responses; redirects on token/service requests are disabled.

Cancellation and persistence share a lock. Once credential persistence begins,
cancellation reports `AUTH_FINISHING`; it never claims cancellation and later adds
an account. A user can still cancel while waiting for Microsoft or choosing storage.
Native vault prompts must be resolved through the OS, not forcibly interrupted.
Cancelled/expired pending credentials are forgotten; verifiers, state and serialized
credential records use zeroizing buffers. This is not a guarantee against memory
inspection by an attacker controlling the current OS account.

Changing browser method starts fresh state/verifier. Unavailable embedded browsing
falls back to a newly bound system-browser transaction; unavailable OS opening falls
back to device code. Provider errors do not switch OAuth identities or hide a
Minecraft registration rejection. Refresh validates the encrypted credential's
issuing Client ID before any request. A login renewing an existing account also
requires the same Minecraft UUID.

## Remote content

`microsoft-auth-*` matches no capability. The Tauri permission resolver is tested
for both local and Microsoft origins: no account, settings, launch, MCP, event,
opener or clipboard commands are granted. There is no remote capability and no
wildcard window grant. There is no iframe or custom script injected into Microsoft.
The opener plugin's automatic page injection is disabled, and authentication windows
are excluded from persistent window-state tracking. Native browsing is private with
DevTools disabled. Navigation to the exact callback is intercepted before rendering;
other nonallowlisted origins/popups fall back to the external browser.

Tauri is still the runtime creating the window; isolation is enforced by its ACL,
not by claiming that the framework's internal transport cannot exist. MFA/passkeys,
federation and browser engine policies require manual platform verification. The
system browser is the RFC 8252 recommendation and remains an explicit alternative.

The loopback confirmation is a separate document owned by TFL, with no reflected
request data, external resources, cache or referrer. A static nonce-authorized script
only replaces the URL with `/complete`; it does not read OAuth parameters, forms or
cookies. No credential is put in its response body.

## Credential storage

One random 256-bit envelope key per provider/account lives in the real OS store.
`keyring-core` uses explicit Windows Credential Manager, Apple Keychain or Linux
Secret Service providers, with no production mock/sample/default-file provider.
Initialization is retried per operation so an unavailable service can recover. New
keys are verified by read-back, and encrypted AES-GCM records are atomically written
and decrypted for verification. Records are bounded to 64 KiB; filenames hash the
provider/UUID, and legacy path components are validated. Microsoft and Yggdrasil
records have separate namespaces. Microsoft records from a nondefault Client ID have
their own issuer namespace, and auth-test records add a separate test-lane namespace.
The encrypted issuer and expiry are restored when loading; they are not trusted from a
frontend preference update. Legacy records are read only by the production/default
lane; test sessions never remove them.

The known weak legacy derivation is read-only. Migration inserts only if no newer
protected record appeared, verifies storage before removing old files, and preserves
account metadata on failure. No insecure write fallback exists. A user may explicitly
choose a memory-only session; persisted token blobs for that account are discarded
first, including partial-save files, so a failed save cannot leave new tokens while
claiming memory-only persistence. If cleanup fails, the choice does not report
success. After process restart it requires reauthorization.
Account operations and refresh share a lock; disk settings saves are also serialized.
General settings updates preserve accounts, active selection and onboarding state.

A memory-only choice never needs to contact the vault: removing all token blobs
already prevents a residual random key from restoring tokens. Such an orphan key
is reclaimed on later disconnect if the vault is reachable.

Disconnecting deletes protected ciphertext, its vault key and legacy records before removing
or clearing local credentials. A deletion failure is visible; it is not reported as a
successful logout. Logout preserves the public account profile and selects offline.
This is local logout, not a promise to revoke an already running Minecraft process.
A rotated refresh token is persisted and missing rotation preserves the previous
refresh token. Transient network fallback is permitted only for a token with proven
unexpired protected expiry, with a visible notice; rejection never takes this path.

## Logs and MCP

Credential fields never serialize in `MinecraftUser`; Debug also redacts them.
LaunchConfig Debug omits credentials/environment/argument values, and launch logging
no longer prints command arguments. Known access tokens are removed from game output
before broadcast to the UI. Crash report excerpts and diagnosis input redact JWTs and
labelled auth material. Profile/service failures do not log raw HTTP response bodies.
MCP preferences reject credential/identity/account keys and no OAuth/account tools
were introduced. Generic MCP log/diagnostic redaction remains in force.

Minecraft's protocol still requires its token in the game's process arguments.
Untrusted game/mod code can write its own files; launcher filtering cannot promise
that every arbitrary third-party secret in every such file is identifiable. Real
credential stores and process-level access must be tested on each target OS. CI
uses synthetic credentials, loopback fixtures and an injected vault, never user
passwords or real tokens.

## Dependency/license review

New OS-store dependencies are pinned: keyring-core 1.0.1, Windows provider 1.1.0,
Apple provider 1.0.2, Secret Service provider 1.0.1. They use MIT OR Apache-2.0,
compatible with the existing workspace licensing; no license was changed. The
existing RustCrypto AES-GCM/SHA-256 and rand implementations are reused. zeroize
provides buffer cleanup. Cargo.lock records transitive versions; cargo audit and
platform checks are part of validation. No cryptographic algorithm was invented.
