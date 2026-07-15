# 1. Contract

B1.3 persists a bounded RUIP program as data. `RAIOS_UI_SPEC_V1` is parsed into
`raios_core::ui_program::Program`; its canonical RUIP bytes and
`ProgramIdentity.sha256` are the durable subject. The executable remains the
already-signed, checked-in `svc.user.shell` Wasm guest. No new Wasm service,
guest artifact, import, loader entry, or service slot is installed.

The end-to-end flow is:

1. The proof uses the existing serial ingress: `program.submit_chunk` calls
   `program_workspace::submit_serial_chunk`, `program.submit_finalize` calls
   `program_workspace::finalize_serial`, and `program.workspace` calls
   `program_workspace::snapshot`. Existing `Program::parse`,
   `Program::canonical_bytes`, and `Program::identity` establish one canonical
   byte string and SHA-256 identity. Provider-authored drafts continue to enter
   through existing `program_workspace::accept_provider_answer`; B1.3 stores
   canonical RUIP, never provider prose or `RAIOS_UI_SPEC_V1` source text.
2. The owner clicks the existing Genesis context-panel button labelled
   `Approve + run program`. Existing
   `ShellHost::handle_pointer_interaction` obtains
   `program_workspace::retained_program`, calls existing
   `PersonalSurface::enter_program`, which calls existing
   `personal_shell_service::invoke_current_boot_program`, and emits the
   unchanged accepted prefix of `PROGRAM_CURRENT_BOOT_ACTIVATION`. Only after
   `PersonalSurfaceRoute::Entered`, NEW
   `program_workspace::approve_retained_program` freezes the exact revision,
   canonical byte length, program SHA-256, and a domain-separated activation
   approval SHA-256. NEW `program_persistence::emit_install_ready_marker`
   reports that binding; it writes nothing durable.
3. After F12 returns to Genesis, the host sends existing
   `project.install_prepare sha256:{activation_approval_sha256}`. Existing
   `agent_protocol_project_install::emit_install_prepare` and `prepare_install`
   dispatch to NEW
   `agent_protocol_project_install::prepare_ui_program_install`. NEW typed
   `raios_core::project_install::UiProgramInstallEnvelope` binds
   `subject_kind=ui_program`, `engine_service_id=svc.user.shell`, ABI version,
   canonical program SHA-256/length, activation approval SHA-256, generation,
   `auto_load=true`, and `trust_tier=dev_key_not_owner_sealed`. NEW
   `seal_ui_program_install_envelope`,
   `validate_ui_program_install_envelope`, and
   `ui_program_install_envelope_hash` seal and recheck it. This is an install
   envelope for shell-owned data, not a shell-Wasm replacement.
4. W6 reuses existing `ProjectInstallAction`, `approval_hash`,
   `authority_evidence_hash`, `install_action_signature_payload_sha256`, and
   the exact existing `PROJECT_INSTALL_PREVIEW` marker. The action has
   `kind=Install`, `authority=PhysicalOwner`, `service_id=svc.user.shell`, and
   `install_envelope_sha256=Some(ui_program_envelope_hash)`; its signed subject
   is explicitly `ui_program` in the envelope. Its sequence and predecessor
   bind the current RECLOG tail, so an intervening append makes it stale.
5. The harness signs `action_signature_message_sha256` through existing
   `New-ReliableDevPromotionSignatureHex` and sends existing
   `project.install_signature`. Existing `emit_install_signature`,
   `accept_signature`, `verify_promotion_authority_signature`, and
   `seal_install_action` arm the preview. Existing serial
   `project.install_approve` remains denied with
   `project_install_physical_pointer_approval_required` and zero RECLOG or
   ARTSTOR change.
6. The owner clicks the same Genesis context-panel rectangle a second time,
   now labelled `Approve + persist program`. The existing routing priority
   remains signed W6 install, granted-candidate run, workspace-Wasm run, then
   RUIP run. `agent_protocol_project_install::approve_from_pointer` dispatches
   the NEW `PendingAction::UiProgramInstall` branch to NEW
   `validate_current_ui_program_install_preview` and NEW
   `program_persistence::install_approved_from_pointer`. Immediately before
   the first write they recompute canonical bytes/hash, the activation binding,
   envelope/action hashes, W6 signature, physical approval, generation, and
   RECLOG tail.
7. The durable commit reuses the B1.2c three-link order and the same physical
   media path. Existing `durable_store::append_install_authorization` writes a
   `raios.install_authorization.v0` record with `subject_kind=ui_program` after
   existing `validate_signed_install_authorization` dispatches to NEW
   `validate_ui_program_install_authorization`. Existing
   `durable_store::append_promotion_transaction` writes the linked
   `raios.promotion_transaction.v0` promote with NEW
   `PromotionSubject::UiProgram`; existing
   `promotion_transaction_payload_bytes` emits program fields instead of fake
   M6/service evidence. NEW `artifact_store::persist_ui_program` delegates to
   NEW shared `persist_authorized_payload`, the same ARTSTOR
   plan/write/readback/reparse/rescan body used by changed existing
   `persist_promoted_artifact`, then appends a linked
   `raios.artifact_persist.v0` record with `subject_kind=ui_program`. Orphan
   authorization, promote, or blob frames grant nothing. State is marked
   installed and the commit marker is emitted only after all three RECLOG
   frames and the ARTSTOR blob read back exactly. The install click does not
   invoke or rerun `svc.user.shell`.
8. Program bytes cannot live in one RECLOG frame: `MAX_PROGRAM_BYTES` is
   16,384; `RECLOG_FRAME_HEADER_LEN` is 88; therefore a 4,096-byte frame has
   only `4096 - 88 = 4008` payload bytes, already 12,376 bytes short before
   record metadata. ARTSTOR is mandatory. Its existing 48-byte blob header and
   512-byte alignment make a maximum RUIP frame
   `round_up(16,384 + 48, 512) = 16,896` bytes. RECLOG stores only the signed
   authorization, promote/unpromote decision, and exact ARTSTOR link.
9. On reboot NEW `program_persistence::run_boot_autoload` runs after existing
   `project_app_autoload::run_boot_autoload` and existing
   `agent_protocol::run_provider_autoload`, but still before `input::init` and
   every serial command. NEW `resolve_installed_program` folds program
   authorization/promote/unpromote/persist records in RECLOG order. It accepts
   only the newest complete exact link. Existing
   `artifact_store::read_verified_artstor_payload` verifies blob-frame and
   payload hashes; NEW `program_workspace::restore_persisted_program` then
   runs `Program::parse`, requires `canonical_bytes()==payload`, recomputes
   `Program::identity`, compares it with the persisted identity, and restores
   the program into the ordinary workspace as `Source::Durable`. Autoload does
   not start the shell. Genesis shows `Approve + run program`; that fresh
   physical click uses the unchanged path from step 2 and renders the exact
   restored hash through the same signed guest.
10. Existing-style program protocol responses add
    `program.rollback_preview sha256:{program_sha256}` and
    `program.rollback_apply sha256:{program_sha256}` through NEW
    `agent_protocol_program::emit_rollback_preview` and
    `emit_rollback_apply`, both delegating to NEW
    `program_persistence::rollback_preview`/`rollback_apply`. Apply is allowed
    only for the exact active installed hash while the personal shell is not
    running. It reuses `append_promotion_transaction(Unpromote)`, links the
    original install authorization, writes a readback-verified tombstone, and
    calls NEW `program_workspace::remove_restored_program` only when that exact
    durable source is resident. A second apply is denied. The next boot resolves
    `rolled_back`, reads no ARTSTOR payload, and leaves the workspace empty.

Exact new markers, emitted as one logical line (the harness must tolerate the
observed CR-CR-LF transport ending), are:

`PROGRAM_INSTALL_READY result=accepted physical_approval=genesis_pointer program_sha256=sha256:{program} activation_approval_sha256=sha256:{activation} engine=svc.user.shell persistence_authority=false reason=program_current_boot_approved`

`PROGRAM_INSTALL_COMMIT result=accepted physical_approval=genesis_pointer subject_kind=ui_program program_sha256=sha256:{program} activation_approval_sha256=sha256:{activation} install_envelope_sha256=sha256:{envelope} install_action_sha256=sha256:{action} promotion_transaction_sha256=sha256:{promote} program_persist_frame_sha256=sha256:{persist} generation={generation} sequence={sequence} engine=svc.user.shell guest_installed=false durable_writes=true reason=program_installed`

`PROGRAM_AUTOLOAD result={accepted|denied} phase={autoloaded|not_installed|rolled_back|denied} reason={reason} posture={Normal|Probation|Safe|PersistenceUnavailable} program_sha256={sha256:{program}|none} promotion_transaction_sha256={sha256:{promote_or_unpromote}|none} program_persist_frame_sha256={sha256:{persist}|none} w6_signature_verified={true|false} canonical_verified={true|false} workspace_reloaded={true|false} shell_started=false cross_reboot_proven={true|false}`

`PROGRAM_ROLLBACK_COMMIT result=accepted program_sha256=sha256:{program} promotion_transaction_sha256=sha256:{promote} unpromote_transaction_sha256=sha256:{unpromote} workspace_removed={true|false} durable_writes=true reason=program_unpromoted`

Denials use the existing typed response with an exact `reason` and no commit
marker, matching B1.2c. `PROJECT_INSTALL_PREVIEW` stays byte-for-byte:
`PROJECT_INSTALL_PREVIEW kind=install result=accepted signature_verified={true|false} action_signature_message_sha256=sha256:{hash} physical_approval_sha256=sha256:{hash} generation={generation} sequence={sequence} approval={owner_signature_required|genesis_pointer_required}`.

# 2. Invariants

- No durable write precedes the second Genesis pointer approval. Serial can
  deliver, prepare, sign, inspect, and request rollback, but cannot install or
  substitute for either physical click. A changed draft/revision/hash, stale
  activation, changed RECLOG tail, bad signature, replay, or secure-attention
  transition denies before media mutation.
- There is one persistence path: the existing W6 `ProjectInstallAction`
  ceremony, existing RECLOG append/readback chain, and existing ARTSTOR blob
  format/writer. No program partition, workspace file store, second log,
  second blob allocator, or project-install-store copy is added.
- RUIP is bounded UI data. `svc.user.shell` Wasm bytes, descriptor identities,
  six imports, memory/fuel limits, loader/runtime facts, and service inventory
  semantics remain byte-identical. No guest rebuild is required.
- Reboot autoload restores an inert workspace program before commands; it does
  not execute or render automatically. A post-reboot physical click is still
  required before `PersonalSurface::enter_program` and the signed guest run.
- SHA-256 authority is over exact canonical RUIP bytes. Install rechecks the
  retained bytes; autoload verifies ARTSTOR frame hash, payload hash,
  `Program::parse`, canonical round-trip, and recomputed identity before the
  workspace or shell can observe the program. Persisted hashes are never
  trusted as assertions.
- The calculator pin remains 5,372 bytes / SHA-256
  `7ca0aa21d69baae072675c20f7b44d0e2d9f99ac4e72d6aa64e7a25586dfcd6e`;
  the editor pin remains 176 bytes / SHA-256
  `34f726d13818d174e23ef0614ca183a2967b9449c8cf4447151aef13d277d815`.
  RUIP ABI v1 and both canonical encoders are unchanged.
- Rollback is durable and one-shot: the exact unpromote is newest, read back,
  survives reboot, prevents fallback to an older promote, and removes only a
  matching `Source::Durable` workspace entry. It never deletes a newer serial
  or provider draft.
- Existing B1.2c response shapes stay fixed: `module.loader_runtime` remains a
  root v1 denial with 54 evidence entries and no `body.result` or
  `live_granted_load_projection`; service-slot presence remains under
  `facts.runtime`; inventory rows retain no `run_count`; lifecycle
  `body.result` carve-outs remain; marker readers trim CR-CR-LF safely.
- Only program definition bytes persist. `ProgramState`, including text typed
  into the editor, remains current-boot and is not represented as document or
  file persistence.

# 3. File plan

- `raios-core/src/project_install.rs`: add NEW `UiProgramInstallEnvelope` and
  NEW `seal_ui_program_install_envelope`,
  `validate_ui_program_install_envelope`, and
  `ui_program_install_envelope_hash`. Reuse `ProjectInstallAction` and its
  existing signature encoding. Host tests pin sealing, field tamper, wrong
  engine/ABI/hash/length/activation, and action binding.
- `raios-core/src/scoped_promotion_transaction_append.rs`: extend existing
  `ScopedPromotionTransactionAppendInput` and
  `evaluate_scoped_promotion_transaction_append` with a typed `ui_program`
  branch that requires W6 authorization, canonical verification, consumed
  activation/install approvals, exact link geometry, and readback, without
  inventing an M6 grant. Existing granted-service evaluation and reasons stay
  unchanged; add focused host tests for every missing program pin.
- `seed-kernel/src/program_workspace.rs`: add `Source::Durable` and NEW
  `approve_retained_program`, `approved_program_install`,
  `restore_persisted_program`, and `remove_restored_program`; retain canonical
  bytes under their current field instead of adding another cache. Change
  `snapshot` to expose truthful durable source/retention/install hashes while
  preserving current-boot fields for serial/provider drafts. Existing
  `retained_program` still grants no persistence authority by itself.
- `seed-kernel/src/program_persistence.rs` (NEW): own only RUIP install state,
  exact-link resolver, rollback, and the four marker emitters. Add NEW
  `install_approved_from_pointer`, `run_boot_autoload`,
  `resolve_installed_program`, `rollback_preview`, `rollback_apply`,
  `emit_install_ready_marker`, `emit_install_commit_marker`,
  `emit_autoload_marker`, and `emit_rollback_commit_marker`. It never calls a
  Wasm loader or renderer.
- `seed-kernel/src/agent_protocol_project_install.rs`: add NEW
  `PendingAction::UiProgramInstall`, `prepare_ui_program_install`, and
  `validate_current_ui_program_install_preview`; extend existing
  `prepare_install`, `accept_signature`, `approve_from_pointer`, `State::snapshot`,
  `PreviewSnapshot`, and `fields`. Program responses use
  `install_source=ui_program`, `receipt_kind=ruip_canonical`,
  `w4_project_receipt_present=false`, candidate fields for the program hash,
  and `service_id=svc.user.shell`. Preserve the exact `PROJECT_INSTALL_PREVIEW`
  string above and all granted-candidate/project branches.
- `seed-kernel/src/agent_protocol_program.rs` and
  `seed-kernel/src/agent_protocol.rs`: add the two NEW rollback emitters/routes;
  keep the established `raios.agent.v0` `body.result` carve-out used by current
  `program.*` readers. Neither route can install or start the shell.
- `seed-kernel/src/durable_store.rs`: add NEW `PromotionSubject::UiProgram` and
  NEW `UiProgramInstallAuthorization`; extend existing
  `InstallAuthorizationRecord`, `PromotionTransactionRecord`,
  `append_install_authorization`, `validate_signed_install_authorization`,
  `append_promotion_transaction`, and `promotion_transaction_payload_bytes`.
  Add NEW `validate_ui_program_install_authorization` and
  `ui_program_promotion_transaction_fields`. Reuse schemas
  `raios.install_authorization.v0` and `raios.promotion_transaction.v0` with
  `subject_kind=ui_program`; omit, rather than zero-fill, M6/W7/service-slot
  fields. Existing granted-candidate IDs/payloads/parsers remain byte-compatible.
- `seed-kernel/src/artifact_store.rs`: extract NEW private
  `persist_authorized_payload` from the existing writer; change existing
  `persist_promoted_artifact` to delegate without output changes. Add NEW
  `persist_ui_program` and `ui_program_persist_records_from_reclog`; generalize
  existing `read_verified_artstor_payload` over a typed ARTSTOR reference.
  Reuse `current_boot_reclog_scan`, `next_free_artstor_offset_on_disk`,
  `plan_artifact_blob_write`, and all existing write/readback/rescan checks.
- `seed-kernel/src/shell_host/genesis.rs`: change existing
  `handle_pointer_interaction` only to record approval after an accepted RUIP
  entry and to render the program W6 preview/button truthfully. Change existing
  `note_program_route` only by emitting `PROGRAM_INSTALL_READY` after the
  unchanged activation marker. Pointer priority remains unchanged.
- `seed-kernel/src/personal_shell_service.rs` and
  `seed-kernel/src/shell_host/personal_surface.rs`: no changes. Their existing
  `invoke_current_boot_program`/`enter_program` path is the proof that the
  signed guest and core-owned state remain the sole runtime.
- `seed-kernel/src/repromotion.rs` and
  `seed-kernel/src/project_app_autoload.rs`: no changes. Their existing
  `run_provider_autoload`, `resolve_granted_candidate_install`, `reverify_record`,
  and `run_boot_autoload` remain service-specific references, not RUIP loaders.
- `seed-kernel/src/main.rs`: declare the NEW module and call NEW
  `program_persistence::run_boot_autoload` after both existing autoload calls
  and before `input::init`.
- `vm-harness/shadow-vm-smoke-profile-genesis-ui.ps1`: reuse existing
  `Send-GenesisUiProgramBytes`, real `program.*` readers, editor approval/F12,
  and exact fixtures; append the W6 install proof without changing existing
  predicate names.
- `vm-harness/shadow-vm-smoke-support.ps1`: add NEW
  `Invoke-SignedUiProgramInstall`, reusing existing
  `New-ReliableDevPromotionSignatureHex`, `Send-QemuAbsolutePointerClick`, and
  the shared 200 ms wait grid. It returns activation/envelope/action/promote/
  persist hashes and click count.
- `vm-harness/shadow-vm-smoke.ps1`: give `genesis-ui` the existing valid-a
  disposable persist disk, Rust signer, monitor, and QMP setup already used by
  W6 profiles; do not add network.
- `vm-harness/shadow-vm-persistence-reboot.ps1`: add a
  `-ProgramPersistence` branch that reuses existing `Start-RaiosVm`, clean-QEMU,
  `Get-PersistInspection`, mutation, and report helpers but runs a key-free,
  network-free three-boot RUIP proof. The default B1.2c provider proof remains
  unchanged.

# 4. Harness plan

Extend `genesis-ui` with these new predicates and exact pins:

- `genesis-ui:editor-install-ready-exact-physical-binding`: existing editor
  activation succeeds first; the ready marker binds the 176-byte editor hash,
  `svc.user.shell`, and `persistence_authority=false`; RECLOG/ARTSTOR unchanged.
- `genesis-ui:editor-w6-prepare-binds-approved-ruip`: source `ui_program`,
  receipt kind `ruip_canonical`, no W4 receipt, exact editor/activation/envelope,
  owner signature required, no write.
- `genesis-ui:editor-w6-signature-separate-authority`: Rust signer accepted;
  action digest differs from activation hash; exact preview changes only to
  `signature_verified=true`/`genesis_pointer_required`.
- `genesis-ui:editor-serial-install-approval-denied-zero-effect`: exact
  physical-pointer-required reason, preview retained, RECLOG/ARTSTOR counts and
  tails unchanged, no commit marker.
- `genesis-ui:editor-second-click-persists-without-rerun`: one additional QMP
  click, accepted exact commit marker, three consecutive linked RECLOG frames,
  one ARTSTOR frame, `guest_installed=false`, and no new
  `PROGRAM_CURRENT_BOOT_ACTIVATION` or shell run.
- `genesis-ui:editor-artstor-canonical-readback`: blob payload is exactly 176
  bytes with the pinned editor SHA-256; blob frame/persist/promote hashes match
  marker and guest diagnostics.
- `genesis-ui:ruip-byte-compatibility-pins-unchanged`: calculator 5,372-byte
  hash and editor 176-byte hash remain exact, and all pre-existing calculator,
  editor, malformed-input, HID, inventory, F12, trap, and fuel predicates pass.
- `genesis-ui:b12c-response-shapes-unchanged`: loader-runtime 54-entry root
  denial, `facts.runtime` slot location, inventory without `run_count`, and
  lifecycle carve-outs remain exact.

Run `shadow-vm-persistence-reboot.ps1 -ProgramPersistence` as the dedicated
cross-boot proof with these predicates:

- `program-boot1:approved-editor-installed`: two distinct physical clicks,
  exact ready/W6/commit hashes, three linked RECLOG frames, exact ARTSTOR bytes,
  and no install-time rerun.
- `program-boot2:autoload-before-command`: exact `PROGRAM_AUTOLOAD` appears
  before the first tagged command with `phase=autoloaded`, W6/canonical true,
  workspace reloaded true, shell started false, cross-reboot true.
- `program-boot2:workspace-restored-exact`: real `program.workspace` reader
  reports `Source::Durable`, durable retention, 176 bytes, exact editor hash,
  no pending delivery, and no execution authority.
- `program-boot2:physical-click-runs-restored-editor`: one fresh pointer click
  emits the existing activation marker for the exact editor hash; inventory
  shows only current-boot `svc.user.shell`; editor HID/CLEAR/F12 behavior works.
- `program-boot2:boot1-prefix-immutable`: every boot-1 RECLOG frame is an
  identical prefix and program autoload itself appends nothing.
- `program-boot2:rollback-tombstone-readback`: after F12, preview names the
  exact active hash; apply appends one linked unpromote, commit marker hashes
  match, exact durable workspace entry is removed, second apply denied.
- `program-boot3:rollback-resolved-before-command`: autoload marker is
  `phase=rolled_back`, names the unpromote, canonical/workspace/shell flags are
  false, and precedes commands.
- `program-boot3:workspace-empty-no-shell`: real workspace and inventory
  readers show no durable program and no `svc.user.shell`; no activation/frame
  marker appears.
- `program-corrupt-blob:autoload-denied-no-workspace`: changed ARTSTOR bytes
  produce exact blob/payload hash denial, no fallback, no restored program.
- `program-tamper-link:autoload-denied-no-fallback`: a rebuilt RECLOG chain with
  a changed program-persist link is rejected before workspace restore.
- `program-safe:autoload-skipped-no-workspace`: Safe posture denies before
  record/blob intake and leaves workspace empty.
- `program-reclog-chain:authorization-promote-persist-unpromote-valid`: host
  inspection and guest scans agree on sequence, predecessor, hashes, immutable
  prefix, and newest tombstone.

All marker readers use the existing tolerant line search plus `TrimEnd()`;
none assumes a sub-200 ms event through the 200 ms grid. No new guest
diagnostic is introduced: proofs use `program.workspace`, `service.inventory`,
the existing RECLOG/ARTSTOR scans, serial markers, host persist inspection, and
the existing signed-shell UI behavior.

# 5. Risks and open questions

- Autoload ownership: extending `run_provider_autoload` would falsely require
  W7/M6/service-loader evidence; extending project `run_boot_autoload` would
  falsely require a W4 receipt and project Wasm commit. Recommended resolution:
  NEW sibling `program_persistence::run_boot_autoload`, sharing durable and
  ARTSTOR primitives, called beside both before commands. It restores data only.
- Rollback meaning: B1.3 has no prior durable RUIP version or version-selection
  UI. Recommended resolution: first-version rollback is durable removal via a
  linked unpromote tombstone; deny a second durable program install until the
  first is rolled back. Add previous-version restoration only when a real
  multi-version requirement supplies selection and retention semantics.
- Shell guest changes: none are needed. Recommended resolution: treat any
  proposed guest rebuild, descriptor resign, new import, loader call, or
  auto-start as a stop condition; restored bytes must enter the existing
  `PersonalSurface::enter_program` path after a new physical run click.
- Frame capacity and partial commits: a 16-KiB RUIP cannot fit RECLOG, and a
  crash may leave an orphan authorization/promote/blob. Recommended resolution:
  use ARTSTOR plus exact linked records; newest incomplete decisions deny with
  no fallback and no repair/GC in B1.3.
- Meaning of editor persistence: this slice preserves the editor program
  definition, not text typed into its `ProgramState`. Recommended resolution:
  keep state/documents explicitly current-boot; durable document storage needs
  its own owner-approved data model and rollback contract later.
- Compatibility risk: the shared schemas gain a typed `ui_program` variant.
  Recommended resolution: branch on `subject_kind`, preserve existing
  granted-candidate serialization byte-for-byte, and keep every B1.2c known
  response/marker reader pinned in both focused profiles.
