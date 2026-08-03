# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-08-03, owner commit-all pass complete)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`bfcafc9` on GitHub and Codeberg. Five independently verified slices landed:
JSON-fixture lock (`1ea2f0b`), rollback image security (`5dc3f3a`, 11/11
contract tests), current-main rustfmt (`23e346f`), bounded JSON fixtures
(`148678f`), and historical audit artifacts (`bfcafc9`, docs hygiene 12/12).
The original detached snapshot is preserved and published as
`owner/commit-all-20260803` at `fc833ce` on both remotes.

Two old-root strands remain deliberately uncommitted. Crash-loop supervision
is still rejected: only synthetic Echo paths classify crashes; general
Runtime/protocol dispatch and hostcall authority are unproved, requiring a
wider allocation including `wasm_runtime/probes.rs` and
`agent_protocol_wasm.rs`. NET8 TLS is blocked after two strategies: .NET and
OpenSSL probes both proved Schannel rejects the process-local ephemeral server
key (`0x8009030E`, "platform does not support ephemeral keys"). Persisted PFX/
certificate-store credentials or a contract/host change require owner/security
authorization.

The canonical main worktree's four H26 files remain foreign stopped-lane WIP:
`seed-kernel/src/{wifi.rs,marvell_wifi_pcie.rs}` plus the two WLAN predicate
scripts. Preserve them exactly. ADR 0045's R3 stop remains in force; no image,
USB write, unsafe-baseline update, or H26 product commit is authorized.

## Next step

Owner chooses a new strategy for one blocked strand: authorize the wider
crash-loop integration, authorize a NET8 credential/host contract, or reopen
H26 with a new Ready-replacement ownership model. Until then, preserve both
worktrees and continue only independent scope.

## Recently (exactly 3, newest first)

### 2026-08-03 - bounded owner WIP published
Five green slices were committed with exact file sets and pushed to both
GitHub and Codeberg; Codeberg `main` now matches GitHub `main`.

### 2026-08-03 - NET8 parked on Schannel credential boundary
Build passed, but independent .NET/OpenSSL TLS probes confirmed the host cannot
use the required ephemeral process-local key; red WIP was not committed.

### 2026-07-22 - H26 R3 rejected at Ready replacement
Review proved stale Ready quarantine can destroy a concurrent winner; the four
H26 files remain preserved and hardware release remains denied.
