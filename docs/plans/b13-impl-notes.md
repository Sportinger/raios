# B1.3 implementation notes

## foundation packet

- Scope is limited to `raios-core` install/scoped evaluation plus the shared
  RECLOG/ARTSTOR foundations; program state, workspace, protocol, Genesis,
  autoload/rollback resolution, `main.rs`, and harness wiring remain later.
- The existing granted-candidate record structs are constructed in out-of-scope
  modules. Adding a required enum field would break those callers, so
  `PromotionSubject` is threaded through typed record views: existing records
  infer `GrantedCandidate`; new fixed-size UI-program records carry
  `PromotionSubject::UiProgram` explicitly.
- Granted-candidate payload builders and field order are unchanged. In
  particular, no `subject_kind` field was inserted into their JSON; only the
  new UI-program variants emit the discriminator.
- UI-program authorization stores the W6 action predecessor hash because it is
  part of the existing signature payload and must be reconstructible when the
  RECLOG tail is non-empty.
- UI-program promotion requires the linked authorization, envelope/activation
  binding, W6 signature, canonical-byte verification, consumed activation and
  install approvals, exact authorization-frame link, and physical readback.
  It deliberately omits M6 signature/grant/placeholder and service-slot pins.
- UI-program promotion records carry only subject identity, canonical program
  hash/length, ABI, activation binding, W6 link decisions, generation, and the
  optional rollback event. They carry no RUIP bytes and no W7/M6/service fields.
- New record IDs are `install_authorization.origin_boot.ui_program.v0`,
  `promotion_transaction.origin_boot.{promote|unpromote}.ui_program.v0`, and
  `artifact_persist.origin_boot.ui_program.v0` under the existing schemas.
- The program artifact-persist record links canonical identity, activation,
  envelope, authorization frame, promotion frame, and the typed ARTSTOR ref.
- `read_verified_artstor_payload` now accepts the existing granted record or
  the new UI-program record through one fixed typed reference; granted callers
  and verification order are unchanged.
- DER storage remains the existing fixed 256-byte W6 maximum; no growing
  authorization state or new dependency was added.
- Capacity is code-constant-backed: RECLOG payload is 4,096 - 88 = 4,008 bytes,
  below the 16,384-byte RUIP maximum; ARTSTOR is
  round_up(16,384 + 48, 512) = 16,896 bytes.
- UI-program authorization/promotion parsing and newest-complete-link folding
  remain for the later `program_persistence` packet; this packet only writes
  their typed foundation records and parses UI artifact-persist links.
- No Cargo, rustc, build, formatter, or test command was run per worker scope.

## install-flow packet

- `program_workspace` keeps canonical RUIP bytes only in the existing draft
  field. Activation approval stores fixed revision/length/hash metadata and a
  `raios.ui_program_activation_approval.v1` domain hash; replacing the retained
  revision invalidates it.
- Durable workspace provenance is a typed `Source::Durable` value carrying the
  canonical identity plus W6 authorization/promotion/persist links. Restore
  reparses, requires canonical round-trip and exact persisted hash/length, and
  rollback removal matches both durable source and program hash.
- Same-boot install reuses the existing W6 cursor, pointer-approval hash,
  signature verifier, UI-program authorization append, linked promote append,
  and ARTSTOR writer. Installed state and `PROGRAM_INSTALL_COMMIT` appear only
  after all three RECLOG frames and the blob read back successfully.
- Existing project and granted-candidate response/marker branches are unchanged.
  The UI-program variant reports `svc.user.shell`, `ui_program`,
  `ruip_canonical`, the canonical program hash, and no W4 receipt; serial
  approval remains denied and pointer failure emits no commit marker.
- Genesis pointer priority is unchanged. An accepted RUIP run now records the
  frozen approval and emits `PROGRAM_INSTALL_READY` immediately after the
  unchanged `PROGRAM_CURRENT_BOOT_ACTIVATION` line; a signed program preview
  renders `Approve + persist program`.
- `run_boot_autoload`, `resolve_installed_program`, `rollback_preview`, and
  `rollback_apply` are compiling fail-closed stubs for the later packet.
  `main.rs` declares the module but deliberately does not call autoload.
- Unfinished here by scope: RECLOG folding, cross-reboot ARTSTOR restore,
  autoload acceptance markers, linked unpromote/one-shot rollback, program
  protocol rollback routes, and all VM-harness proof.
- No Cargo, rustc, build, formatter, or test command was run per worker scope.

## reboot packet

- `resolve_installed_program` now folds UI-program authorization, promote,
  persist, and unpromote records in RECLOG order. A newer incomplete UI-program
  chain resolves as not installed and never falls back to an older complete
  install; an unpromote resolves only from the exact active promote/persist and
  original authorization link.
- The foundation did not include UI-program authorization or promotion payload
  parsers. They now live beside their durable record models in
  `durable_store.rs`, revalidate the reconstructed W6 authorization/signature
  and all fixed UI-program promotion fields, and do not change granted-candidate
  parsing or serialization.
- The UI-program append gate previously required the linked authorization to be
  the current RECLOG tail for both transaction kinds. That remains exact for
  `Promote`; `Unpromote` instead accepts the separately revalidated original
  authorization link, because the completed promote and artifact-persist frames
  necessarily follow that authorization before rollback.
- Boot autoload mirrors the B1.2c Normal plus success-marked posture gate, emits
  the exact `PROGRAM_AUTOLOAD` field order for every outcome, reads ARTSTOR only
  for an active complete link, and restores canonical RUIP as inert
  `Source::Durable` without starting the shell.
- Program rollback preview/apply routes use the existing `raios.agent.v0`
  `body.result` response shape. Apply consumes the dispatch event id, requires
  the exact active hash and a stopped personal shell, appends/readback-verifies
  one linked unpromote, then removes only a matching durable workspace entry;
  replay is denied and emits no second commit marker.
- `main.rs` calls program autoload after provider autoload and before input
  initialization. No re-export was needed because `program_persistence` is a
  direct kernel module.
- Remaining B1.3 work is VM-harness wiring and the three-boot positive,
  tamper-denial, rollback, and byte-compatibility proof packet.
- No Cargo, rustc, build, formatter, or test command was run per worker scope.

## harness cycle 1

- `genesis-ui` now reuses the proven W6 Rust signer, exact valid-a disposable
  persist disk, monitor, and QMP setup without adding network.
- NEW `Invoke-SignedUiProgramInstall` mirrors the granted-candidate ceremony:
  exact approved-RUIP prepare, reliable dev signature, typed serial denial,
  one Genesis click, the merged `PROGRAM_INSTALL_COMMIT` field order, tolerant
  `TrimEnd()` marker parsing, and pre/denied/post RECLOG plus ARTSTOR scans.
- The eight section-4 `genesis-ui` predicates are appended after every existing
  Genesis predicate so the compatibility predicate can prove the unchanged
  calculator/editor, malformed-input, HID, inventory, F12, trap, fuel, and
  recovery checks all ran first.
- Open kernel diagnostic gap: merged `artifact_store::emit_artifact_store_scan`
  calls only `artifact_persist_records_from_reclog`, whose parser requires the
  granted-candidate `artifact_sha256`/M6 fields. It does not call
  `ui_program_persist_records_from_reclog`. Consequently a valid UI-program
  blob is currently emitted as `garbage_blobs` and no UI-program record exposes
  its canonical 176-byte hash, authorization link, or persist-frame link. The
  requested `editor-second-click-persists-without-rerun` ARTSTOR-count pin and
  `editor-artstor-canonical-readback` predicate intentionally fail with the full
  parsed scan until that merged emitter is extended; the harness does not fake
  or infer the missing positive record.
- No Cargo, build, formatter, or VM command was run per worker scope.

## artstor-scan ui_program fix

- `artifact.store_scan` now folds UI-program persist records into a parallel diagnostic array.
- Each UI record exposes canonical program, ARTSTOR verification, and install-chain links.
- UI-program ARTSTOR offsets now suppress the same false garbage report as granted records.
- The scan remains read-only and grants no load, execution, or durable-write authority.
