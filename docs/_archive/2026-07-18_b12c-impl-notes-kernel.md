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

## resolver packet

- Added one RECLOG-order resolver shared by provider autoload and manual
  repromotion. A newer complete unpromote tombstones the install; a newer
  incomplete/malformed promote or unmatched artifact-persist record resolves to
  no active install and never falls back to an older pair.
- Provider autoload now requires a success-marked Normal boot, restores the
  persisted W6 authorization, drives the existing reverify/load/start chain,
  and emits the Contract-11 marker for every outcome with the resolved promote,
  artifact-persist, or unpromote frame identity.
- Recovery load now carries the restored fixed-size W6 authorization into its
  promote append and leaves it in state for a later rollback/unpromote. Fixed
  the persisted install-signature parser delimiter needed by that path.
- Remainder: no Cargo, build, format, or test command was run, per resolver
  scope; compile-loop and focused VM evidence remain for the orchestrator.

## harness cycle 1

- Moved the network-acquisition command/reader/predicate 1-24 sequence
  unchanged into Invoke-W7M6PhysicalActivation; its returned click count is 2
  because the authoritative prefix contains the stale-binding denial click
  followed by the fresh accepted activation click.
- Derived the install needle and parser from the merged
  emit_install_commit_marker: only the accepted path emits
  GRANTED_CANDIDATE_INSTALL_COMMIT, with run_count=1 and
  trust_tier=dev_key_not_owner_sealed literals; promotion and artifact frame
  identities are parsed as sha256: values (the emitter can render none).
- The serial project.install_approve proof pins the current response reason
  project_install_physical_pointer_approval_required; it expects no denied
  install marker and compares unchanged durable.record_log_scan and
  artifact.store_scan evidence.
- The current project-install response exposes the granted source through
  install_source, receipt_kind, w4_project_receipt_present,
  activation_approval_sha256, and install_envelope_sha256; its
  promotion/artifact frame fields remain null before the physical click.
- Code/plan difference: the shared response emitter still renders its inherited
  service_id as svc.workspace.current_boot even for the granted-candidate
  variant. Cycle-1 predicates therefore identify this branch with
  install_source=granted_candidate and the exact candidate/W7/activation
  fields; they do not mislabel the inherited service_id as granted provenance.
- artifact.store_scan exposes the verified candidate-to-promote link but not
  the artifact-persist RECLOG frame hash. Predicate 30 therefore uses the
  existing RECLOG diagnostic's post-install head/tail hashes: promote is the
  head and the accepted marker's artifact-persist frame is the tail.
- Source-read the merged provider-autoload emitter for later cycles. Its exact
  chain is result, phase, reason, posture, candidate, promotion,
  artifact-persist, m6_reverified, w6_signature_verified, loaded, running,
  run_count, cross_reboot_proven; cycle 1 adds no m6d/reboot wiring.

## frame-split packet

- Added `raios.install_authorization.v0` as a separate origin-boot RECLOG
  record. Its append validates the signed W6 authorization before any write,
  then uses the existing plan/write/readback/reparse/rescan path and returns
  the authorization frame sequence, offset, and hash.
- Promotion and unpromotion payloads now carry only
  `install_authorization_frame_sha256` plus the existing four authorization
  booleans; the embedded W6 fields were removed. The accepted
  `GRANTED_CANDIDATE_INSTALL_COMMIT` marker was not changed.
- Physical install now appends authorization, promotion, and artifact-persist
  in that order. State retention and the accepted marker still happen only
  after all three durable operations succeed; an orphan authorization or an
  authorization+promotion pair grants nothing.
- Resolver/autoload joins the exact authorization frame before accepting the
  exact promotion-to-artifact pair, reconstructs the envelope inputs from the
  linked promotion, and verifies the W6 signature before re-promotion. Missing,
  old embedded-only, malformed, superseded, and incomplete sets stay denied
  with no fallback.
- No raios-core change was needed. No Cargo, build, or VM command was run per
  packet scope; direct `rustfmt --check` parsed the three Rust files and
  reported the pre-existing unformatted B1.2c regions, so formatting and live
  evidence remain for the orchestrator.

## harness cycle 2

- Added `m6d-rollback` to the network-acquisition launch path: mandatory
  `-Network`, Rust signer build, QMP/monitor ports, e1000/W7 fixture and
  temporary pin-bearing image, caller-image rejection, and its own exact
  valid-a disposable persist disk in the hardware profile/report.
- Rewrote the profile around `Invoke-W7M6PhysicalActivation` and
  `Invoke-SignedGrantedCandidateInstall`; the activation helper deliberately
  retains its authoritative `network-acquisition:*` predicate names and reports
  two clicks (stale denial plus accepted activation), while W6 adds the third
  QMP click and second consumable approval.
- The merged rollback literals are `no_recorded_promotion_to_roll_back`,
  `rollback_plan_recorded_ram_only`, and
  `unpromoted_dev_key_granted_external_wasm_current_boot`; manual repromotion
  after the tombstone reports `status=no_artifacts`,
  `reason=granted_candidate_install_rolled_back`, and `performed=true`.
- The shared install helper does not return the origin authorization-frame hash,
  and no guest diagnostic exposes promotion payloads. The profile therefore
  pins the observable frame split without inventing a hash: three consecutive
  RECLOG frames, marker sequence as authorization sequence, linked promote hash,
  artifact-persist tail/record, then the performed linked unpromote and the
  resolver's rolled-back result. The resolver can reach that result only after
  joining and validating the same `install_authorization_frame_sha256` carried
  by the unpromote.
- No Cargo build, kernel build, or VM run was performed in cycle 2. The worker
  only performs the required PowerShell parser/encoding checks after editing.

## harness cycle 3

- Replaced the persistence profile's serial-ingress/clickless promotion flow
  with the proven boot-1 `Invoke-W7M6PhysicalActivation` and
  `Invoke-SignedGrantedCandidateInstall` helpers. Boot 1 now packages the same
  temporary W7-pin image, runs the TLS fixture over e1000/guestfwd, exposes QMP
  for the existing pointer coordinates, and builds the shared Rust signer.
- Derived the frame geometry from the merged writer plus both green cycle
  reports: install authorization is `pre_tail+1`, the accepted marker sequence
  and promote frame are `pre_tail+2`, and artifact-persist is `pre_tail+3`.
  Clean-quit host inspection pins all three records and the exact ARTSTOR blob.
- Boot 2 parses the exact provider-autoload marker field order with `TrimEnd()`
  for CR-CR-LF logs before any tagged command, proves the boot-1 RECLOG frames
  remain an identical prefix, accepts the legitimate fresh promote/artifact
  pair as a newer valid tail, and appends the linked unpromote only at the end.
  Boot 3 pins the exact `rolled_back` marker before commands and proves the
  service and slot stay absent with host/guest RECLOG agreement.
- Kept the memory-content, Safe-posture, recovery-load-by-hash, hash-tamper,
  torn-tail, clean-QEMU, and kept-disk proofs. Recovery-load-by-hash children
  use real `Probation` BOOTCTL state: automatic provider autoload is denied,
  while the existing recovery executor explicitly allows Normal/Probation.
- Plan/code gap: `--tamper-persist-record` changes the artifact-persist hash and
  rebuilds the later RECLOG chain. The merged newest-install resolver rejects
  that non-exact promote/artifact link before `reverify_record`, so its exact
  automatic marker is `result=accepted phase=not_installed
  reason=no_w6_authorized_install`, not `artifact_sha_mismatch`. The predicate
  keeps the plan name but pins this stronger early fail-closed behavior and no
  fallback; changing the emitter/resolver is outside the one-harness-file scope.
- Minimal-change reading: existing boot-1 durable-memory writes remain after
  installation, so the immutable boot-1 prefix includes those unrelated frames;
  the boot-2 continuation proof starts at that complete clean-quit prefix.
- No Cargo command, build, package, signer invocation, or VM was run in cycle 3.
  Verification was limited to PowerShell parsing, predicate/static inspection,
  diff review, and LF/no-BOM checks.
