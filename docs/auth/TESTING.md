# Authentication validation

Baseline: v0.14.0-beta.6. All account identities and credentials in automated tests
are synthetic fixtures or the already existing public application ID. No real
account login, new Minecraft approval or native vault certification is claimed.

## Automated checks

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace --lib
cargo audit --no-fetch
bun run check
bun run lint
bun run build
node --test tests/unit/*.test.mjs
node tests/visual/auth.mjs
```

The visual script uses the existing Playwright fixture and production frontend;
set `VISUAL_URL`, `PLAYWRIGHT_MODULE` and `CHROMIUM_PATH` for the local environment.
It does not replace a real OAuth login test. CI runs it with the other beta suites.
`artifacts/auth/results.json` and screenshots distinguish simulated IPC explicitly.

The native authentication workflow runs fixtures and the real Tauri permission
resolver on Windows and macOS; Linux uses the existing full-workspace CI. Windows
unit-test executables need a Common Controls v6 activation manifest because Tauri's
normal application resource embedding targets application binaries. The workflow
adds that manifest only for tests. The asset-scope test uses Tauri's platform home
resolver, not the Unix-only HOME environment variable.

| Area         | Automated evidence                                                                                                                         |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------ |
| PKCE         | Random independent verifiers, S256, RFC 7636 test vector, authorization parameters                                                         |
| Callback     | Valid callback once, duplicate code/state, wrong host/path/origin/method, fragments, replay, real TCP socket                               |
| Lifecycle    | Expiry, cancellation, 64 commit/cancel races, single-flight ownership, pending-secret cleanup                                              |
| Alternatives | Embedded unavailable/unsupported → external; external unavailable → device; denial never retried with another identity                     |
| Microsoft    | Real HTTP fixture for code exchange and refresh grant; verifier/redirect/client ID checked; safe error/consent classification              |
| Xbox/XSTS    | HTTP fixture, missing claims, user-hash match, distinct errors and family/profile restrictions                                             |
| Minecraft    | HTTP fixture profile, valid UUID/name, distinct profile errors, both registration error fields, unknown 403 not mislabeled                 |
| Credentials  | OS-vault abstraction, random key + ciphertext roundtrip, unavailable vault preserving record, original legacy format, provider isolation   |
| Refresh      | Rotated/omitted refresh token handling, expiry, issuer change, no network fallback for rejection                                           |
| Public data  | Tokens skipped by serialization/Debug; pending credentials absent from progress; launch/diagnostic redaction                               |
| Logout       | Memory cleared, profile retained, offline selected, onboarding preserved                                                                   |
| Permissions  | Actual Tauri resolver denies auth window at local and Microsoft origins; MCP preference allowlist rejects auth keys                        |
| UI           | Modern/Pixel onboarding/accounts at 800×600 and 1280×720; method switch, storage consent, retry, renewal, cancel/close, English and Escape |

## Manual platform matrix — not yet certified

Use an internal test build. Its application identity, private webview store, launcher
profile and credential namespace are isolated automatically beneath
`.tflclient-auth-tests/<Client ID>/.tflclient`; it cannot load, migrate or delete
legacy production credentials. This remains true when `TFL_DATA_DIR` supplies a
portable base directory. Use the owner's own account directly with Microsoft; do not
supply credentials to the agent, CI, screenshots or test fixtures. Obtain real Client
ID approval separately.

| Scenario                                                                         | Windows | macOS   | Linux   |
| -------------------------------------------------------------------------------- | ------- | ------- | ------- |
| Official Microsoft page in private embedded window, trusted domain, cancel/title | Pending | Pending | Pending |
| MFA, passkey, account selector, consent, popup/federated fallback                | Pending | Pending | Pending |
| System browser callback, timeout/cancel, unavailable browser → code              | Pending | Pending | Pending |
| Device code success, decline, expiry, child/no-Xbox/no-Java-profile errors       | Pending | Pending | Pending |
| Real Credential Manager/Keychain/Secret Service save/read/remove                 | Pending | Pending | Pending |
| Locked/missing store, retry after recovery, explicit temporary session           | Pending | Pending | Pending |
| Real legacy credential migration, relaunch and rotated refresh persistence       | Pending | Pending | Pending |
| Multiple accounts, renewal UUID mismatch, ID activation/reversion                | Pending | Pending | Pending |
| Login → settings change → launch → logout/remove → restart                       | Pending | Pending | Pending |

Linux headless native webview testing in this workspace is limited by WebKit helper
paths from the unpacked system library. Windowless native tests/fixtures and real ACL
resolution do run. CI compilation is not evidence that Microsoft's web page, MFA or
an interactive native vault prompt was successfully used on that OS.

## Owner test sequence

1. Start the unmodified-production-identity build; verify existing device login and
   offline accounts, without losing instances/preferences or repeating onboarding.
2. Use the test feature with the actual own ID. Confirm any unapproved-registration
   response is visible and never falls back to the inherited OAuth identity.
3. Complete browser and device login with a real licensed account. Check loading
   phases, cancel/expiry, duplicate attempts, keyboard control and both UI styles.
4. Restart and launch with the stored session. Exercise multiple accounts and token
   refresh. Test storage denial, then explicit memory-only use and restart expiry.
5. Start a login, close the panel and wait: no account should appear later. After
   persistence starts, cancel must report finishing rather than claim cancellation.
6. Renew an existing profile using another Microsoft account: original stays intact.
7. Disconnect/remove and restart. Check local credentials are gone, public profile
   behavior matches the operation, and existing instances/settings remain intact.
8. Try MCP read/configure/operate scopes: no tool can retrieve auth progress, tokens,
   OAuth/device polling credentials, cookies or alter the application identity.
9. Record OS/version/browser engine, result and nonsensitive error code. Publish only
   after the separate approval/activation process in [OWNER_ES.md](OWNER_ES.md).

## Local evidence

[Validation summary](evidence/local-validation.json) records 191 Rust library tests,
18 Node behavior tests and 13 simulated-IPC browser checks.
[Browser assertions](evidence/browser.json) list the actual verified interactions.

- [Modern accounts](screenshots/modern-accounts.png)
- [Pixel onboarding at 800×600](screenshots/pixel-onboarding-800.png)
- [Explicit storage consent](screenshots/storage-consent.png)
- [English browser flow](screenshots/english-external.png)

These screenshots contain only synthetic fixture accounts, not a Microsoft login.
Native platform CI results and the exact tested commit are linked in the draft PR.
