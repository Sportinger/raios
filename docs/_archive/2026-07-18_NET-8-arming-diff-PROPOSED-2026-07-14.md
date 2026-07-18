# NET-8 — the owner arming diff (PROPOSED, NOT APPLIED)

This is the single authority-granting act in the entire net-imports lane. Every
slice so far (NET-1 … NET-7) has been *grants-nothing*: the whole machine is
built, signed, cross-checked, and proven to REFUSE. This diff is the one line
that flips a signed service from "refused" to "may run" — for exactly one
artifact, exactly one import list, exactly one test source. It is presented for
explicit owner approval and has NOT been committed.

## What it flips

Today, `svc.net.acquire.w7` is denied before instantiation with
`import_beyond_env_not_owner_authorized`, because every path constructs the
evaluator input with `policy_allows_beyond_env: false`. The arming diff makes
that field `true` at ONE evaluated path, and ONLY when all three bindings match
the pinned W7 identity:

```
policy_allows_beyond_env =
        artifact_sha256      == 32a018b0c730a4f85210ca820483ca68f8a4d0715021a1dda97951fe305e9e54
    &&  import_list_sha256   == <the pinned raios.host_imports.v1 16-import list hash>
    &&  source_policy_id     == "local.qemu.w7"
```

Any other service, any other artifact hash, any other import list, any other
source → the expression is `false` → still denied. The flip is a property of
this exact binding, not a global switch. A grep proves there is exactly one
`true`-constructing site and it is gated by this conjunction.

## What it opens

- `svc.net.acquire.w7`, and only it, may instantiate and run against the fixed
  QEMU test source `local.qemu.w7` (10.0.2.2, exact SNI/path, an ephemeral
  pin-only test certificate) to fetch one owner-approved content hash into
  quarantine as an inert `current_boot`/`local_only` candidate.

## What stays closed (unchanged by this diff)

- No install, load, execute, promotion, persistence, or durable-memory
  authority. The fetched bytes remain an inert candidate; the existing
  M6/M7/provider/owner load gates still deny.
- No secret-lease import. No provider key, no Vault access.
- No production/public network source — QEMU test source only.
- Trust labels stay honest: `pin_only_no_webpki_chain_validation`,
  `not_validated_stage0`, `dev_key_not_owner_sealed`, `owner_sealed:false`.
- The F12 kill, the singleton-TCP lease, the opaque key custody, the
  one-shared-M12-seam finalize — all unchanged and still in force.

## The evidence that will follow approval (NET-8 second half)

Arming is only honest if it is proven. After approval I build the host-side
ephemeral-cert TLS fixture + the `network-acquisition -Network` profile and
prove, live in QEMU: a real e1000/DHCP/TCP/TLS 1.3/HTTP fetch of the pinned
artifact; convergence on the shared chunk/finalize; the retained candidate still
denied at the load preflight; F12 kill during a silent peer; provider/acquisition
busy in both directions; cleanup and retry. Then NET-9 adds the full denial
matrix and inspect/discard.

## Why this is the right stop point

The owner asked to build "up to the arming diff." That is here. Nothing above
this line grants authority; this diff does. It should not be committed on my
judgment alone — it is the owner's gate, by design (ADR 0008, the arming ladder
in `m11-beyond-env-net-imports-scope-2026-07-14.md` §8, and the
resumable-execution addendum). Approve the exact binding above and I build the
live proof; withhold it and everything stays grants-nothing exactly as committed.
