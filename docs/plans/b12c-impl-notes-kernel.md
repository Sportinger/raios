# B1.2c kernel implementation notes

- Reused `ProjectInstallAction` unchanged for W6; its existing
  `install_envelope_sha256` field identifies the sealed W7 envelope.
- Old promotion records parse with absent W6 booleans as `false`, so they stay
  readable but cannot satisfy the new re-verification pins.
- The four scoped durable-append pins are evaluated after the existing M6
  signature/trust checks, preserving their prior first-failure reasons.
## install-flow packet

- Reused the foundation `GrantedCandidateInstallEnvelope` and the existing promotion/ARTSTOR appenders; the second pointer click does not call load or start.
- `SignedInstallAuthorization` is fixed-size current-boot state. The later reboot restoration function is intentionally not added here because this packet does not wire the reboot resolver.
- The granted cursor currently reuses the existing RECLOG-backed project-install cursor solely for its sequence/tail calculation; replacing its workspace projection with the later provider resolver is intentionally out of scope for this same-boot packet.
- Deliberate limitation for follow-up: the durable record currently carries the foundation's four W6 gate pins, but this packet has not yet serialized the full W7/W6 authorization payload or added `durable_store::validate_signed_install_authorization`; this must be completed before treating the path as verified.

## orchestrator compile-loop fixes (after install-flow packet)

- `run_provider_autoload` is re-exported as `agent_protocol::run_provider_autoload`
  (the repromotion module is private to agent_protocol); main.rs calls that path.
- `emit_install_commit_marker` rewritten zero-alloc in the house marker idiom
  (serial::write_raw_str + the existing write_hash); the worker's heap
  `format!`/`String` helper removed (no_std).
- The pointer-commit match gained a fail-closed
  `PendingAction::GrantedCandidateInstall` arm (the early branch handles it;
  reaching the arm denies with `granted_candidate_install_dispatch_error`).
- DEVIATION from plan Contract step 9: only the ACCEPTED install emits the
  `GRANTED_CANDIDATE_INSTALL_COMMIT` marker; denials answer through the
  existing response/denial paths without a marker. Harness predicates must pin
  denial RESPONSES (not a denied marker) or this gets reconciled in the reboot
  packet.

## reboot packet

- Extended the promotion payload with the W7/W6 authorization material and
  added append-time reconstruction/sealing plus W6 signature verification.
  Old records still parse with no W6 authorization.
- Reverify now passes the persisted W6 digest/signature/key into the core
  evaluator and restores the authorization before recovery load/start wrappers.
- UNFINISHED: `resolve_granted_candidate_install` and the Contract-11 exact
  provider-autoload marker are not wired. The current hook remains the
  foundation stub; it must fold promote/unpromote frames and link ARTSTOR
  before calling `reverify_record`.
- UNFINISHED: rollback must retain the restored authorization through the
  recovery load path and the payload parser needs compile-loop confirmation for
  all new fields. No Cargo/build/test was run, per worker scope.
