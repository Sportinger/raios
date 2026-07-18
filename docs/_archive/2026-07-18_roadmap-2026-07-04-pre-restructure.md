# Roadmap

## Agent Handoff Cursor

Last updated: 2026-07-03 local report timestamp by Codex after focused
module-audit-rollback VM verification that the centralized typed agent command
envelope allowlist can now dispatch the existing read-only
`module.audit_rollback_write_boundary` path and denies a mismatched capability
before dispatch. Latest focused report:
`release\vm-reports\shadow-20260703-180640-6876.json` with 1610/1610
predicates and 71 executed commands. Runtime/recovery work remains on the final
architecture path. The next cursor is to repair the full Shadow VM checkpoint
harness around the non-terminal module load-gate `agent audit.events 256`
scrape, which currently closes the serial path after all reached predicates pass
and before recovery/Hello checks can continue.
Keep this section compact. The authoritative, unabridged current
state is
`docs/PROJECT_STATUS.md`; this file should describe direction and the next
cursor, not repeat the full implementation history.

Current phase: Phase 6, Ephemeral Live Services.

Active execution rule:

- keep the existing evidence gates and fail-closed posture
- stop adding schema-only boundaries unless they directly unblock the active
  runtime/recovery behavior, close a concrete trust gap, or repair verification
- prove the next slice with the smallest real observable path for the touched
  boundary: service lifecycle for service work, focused recovery commands for
  recovery work, focused provider checks for trust work, and focused UI/VM
  checks for UI or harness work
- treat the plan as an AI-parallel OS build, not a traditional serial
  big-team roadmap: split independent agents by ownership boundary, then merge
  only real verified slices
- match verification cost to slice risk: docs-only changes need targeted
  diff/whitespace checks; local UI/refactor slices need format plus the
  smallest relevant build/test; trust, storage, rollback, recovery, authority,
  descriptor, harness, provider, or boot changes need focused or quick VM
  evidence; run full VM profiles at checkpoints or before milestone claims, not
  after every tiny evidence-field hop
- batch 3-5 small same-boundary, non-authorizing evidence hops before the next
  focused VM smoke when the prior quick/focused smoke is green; do not batch
  changes that cross storage, rollback, recovery, authority, provider-trust,
  descriptor-signing, harness, or boot-risk boundaries
- do not use `README.md` or `AGENTS.md` as routine slice ledgers; update
  `docs/PROJECT_STATUS.md` for detailed state and `docs/ROADMAP.md` when the
  compact cursor changes

Latest verified implementation slice:

- the Hello rollback path now has accepted current-boot no-write durable-audit,
  rollback-store, and transaction-append writer candidates; append-engine
  readiness reports `available` / `transaction_append_engine_ready` /
  `ready: true`; the durable append-authority and durable audit-policy
  decisions bind that readiness to the media-write gate, media policy,
  target-region write/readback hash, and LBA1/512-byte span, and
  `raios.ram_only_hello_service_rollback_durable_audit_policy_candidate.v0`
  now binds the policy decision, canonical audit-record image, media policy,
  and same target span, and
  `raios.ram_only_hello_service_rollback_durable_audit_policy_acceptance_gate.v0`
  consumes that candidate, and
  `raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_candidate.v0`
  binds the acceptance gate, candidate, decision, audit image, media policy, and
  same target span as current-boot/local-only/read-only evidence, and
  `raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_aware_acceptance_result.v0`
  consumes that ledger candidate, and
  `raios.ram_only_hello_service_rollback_durable_audit_policy_write_authority_availability.v0`
  binds the result, ledger candidate, media policy, target-region
  write/readback, audit/rollback target ids/schemas, and same LBA1/512-byte
  span, and
  `raios.ram_only_hello_service_rollback_durable_policy_ledger_availability.v0`
  consumes that write-authority availability evidence while binding the same
  ledger/media/target evidence; write authority, durable policy ledger,
  durable audit policy, and durable append authority remain unavailable; and
  `raios.ram_only_hello_service_rollback_durable_audit_policy_availability.v0`
  now consumes that policy-ledger availability evidence while binding the
  policy-ledger availability hash, write-authority availability hash,
  ledger-aware result hash, ledger-candidate hash, media policy,
  target-region write/readback, audit/rollback target ids/schemas, and same
  LBA1/512-byte span, and
  `raios.ram_only_hello_service_rollback_durable_append_authority_availability.v0`
  now consumes that audit-policy availability evidence while binding the
  audit-policy availability hash, policy-ledger availability hash,
  write-authority availability hash, ledger-aware result hash, ledger-candidate
  hash, media policy, target-region write/readback, audit/rollback target
  ids/schemas, and same LBA1/512-byte span, and
  `raios.ram_only_hello_service_rollback_transaction_append_availability_decision.v0`
  now consumes that durable append-authority availability evidence while
  binding the audit-policy availability hash, append-engine readiness hash,
  writer-policy preflight hash, media policy, target-region write/readback,
  audit/rollback target ids/schemas, and same LBA1/512-byte span, and
  `raios.ram_only_hello_service_rollback_transaction_append_authority_denial_gate.v0`
  now consumes that transaction-append availability decision while binding the
  durable append-authority availability hash, audit-policy availability hash,
  append-engine readiness hash, writer-policy preflight hash, media policy,
  target-region write/readback, audit/rollback target ids/schemas, and same
  LBA1/512-byte span with `missing_transaction_append_authority: true`; and
  `raios.ram_only_hello_service_rollback_durable_policy_ledger_availability_dry_run.v0`
  now binds the durable policy-ledger availability hash, policy
  write-authority availability hash, ledger-aware acceptance result hash,
  ledger-candidate hash, media policy, target-region write/readback,
  transaction-append authority-denial gate hash, transaction append-availability
  decision hash, audit/rollback target ids/schemas, and same LBA1/512-byte span
  as current-boot test-media-only evidence while keeping durable policy ledger,
  durable audit policy, durable append authority, writes, append, transaction
  append, rollback application, and installed rollback state denied; and
  `raios.ram_only_hello_service_rollback_durable_audit_policy_availability_dry_run.v0`
  now binds the durable audit-policy availability hash, durable policy-ledger
  availability dry-run hash, durable policy-ledger availability hash, policy
  write-authority availability hash, ledger-aware acceptance result hash,
  ledger-candidate hash, media policy, target-region write/readback,
  transaction-append authority-denial gate hash, transaction append-availability
  decision hash, audit/rollback target ids/schemas, and same LBA1/512-byte span
  as current-boot test-media-only evidence while keeping durable audit policy,
  durable append authority, writes, append, transaction append, rollback
  application, and installed rollback state denied; and
  `raios.ram_only_hello_service_rollback_durable_append_authority_availability_dry_run.v0`
  now binds the durable append-authority availability hash, durable
  audit-policy availability dry-run hash, durable audit-policy availability
  hash, durable policy-ledger availability dry-run hash, durable policy-ledger
  availability hash, policy write-authority availability hash, ledger-aware
  acceptance result hash, ledger-candidate hash, media policy, target-region
  write/readback, transaction-append authority-denial gate hash, transaction
  append-availability decision hash, audit/rollback target ids/schemas, and
  same LBA1/512-byte span as current-boot test-media-only evidence while
  keeping durable append authority, durable audit policy, writes, append,
  transaction append, rollback application, and installed rollback state denied;
  and
  `raios.ram_only_hello_service_rollback_transaction_append_dry_run.v0` now
  binds that authority-denial gate hash, transaction-append availability
  decision hash, append-record and sector-plan hashes, target-region
  write/readback hash, planned/readback sector image hashes, audit/rollback
  target ids/schemas, and the same LBA1/512-byte span while proving
  append-image readiness only as current-boot test-media evidence; and
  `raios.ram_only_hello_service_rollback_target_region_sector_inspection.v0`
  now re-reads the materialized `RAIOS_AUDITRB_V0` LBA1 sector through the
  existing AHCI path, verifies the full sector hash, audit-record and
  rollback-transaction hashes, offsets 0/255/480, zero padding, target span,
  and target-region write/readback binding without granting write/append/apply
  authority; `recovery.rollback_materialize_dry_run svc.demo.hello` now
  explicitly materializes the current-boot test-sector write/readback evidence
  without changing service state, and `recovery.rollback_inspect
  svc.demo.hello` exposes the read-only sector inspection first as
  `materialized_target_region_sector_missing` and then with the verified
  hashes/offsets after that materializer; `service.rollback_apply
  svc.demo.hello` now reports explicit missing retained materializer evidence
  before the materializer runs and consumes retained materializer evidence
  afterward instead of writing the target-region sector itself, then reports
  explicit missing retained inspection evidence before
  `recovery.rollback_inspect` runs and consumes retained inspection evidence
  afterward instead of re-reading the target-region sector itself; durable
  media writes, durable audit writes, rollback-store writes,
  transaction append, rollback application, and now
  `raios.ram_only_hello_service_rollback_durable_policy_write_authority_decision.v0`
  consumes the durable append-authority availability dry-run plus
  transaction-append dry-run, target-region sector inspection, write-authority,
  durable audit-policy, durable append-authority, authority-denial, transaction
  append-availability, target id/schema, and LBA1/512-byte evidence while
  keeping durable policy ledger, write authority, durable audit policy, durable
  append authority, transaction append, media write, append, rollback
  transaction append, durable audit writes, rollback-store writes, write
  attempts, rollback application, persistence, external bytes, candidate
  execution, executable mapping, provider auto-load, broad mutation, and
  installed rollback state remain denied
- positive provider request/export binding now carries a canonical
  `provider_trust_evidence_hash` over provider host, trust state, pin kind/id,
  and TLS-bypass state; the hash is folded into the request-binding and
  export-audit binding hashes, retained through binding consumption and final
  injection authorization checks, exposed in provider gate diagnostics and
  RAM-only event bindings, and automatic context injection remains disabled
- the trust evidence now includes explicit
  `raios.provider_trust_verifier_metadata.v0` for the real Stage-0 OpenAI
  pinned TLS verifier: verifier id, exact-host policy, configured leaf/SPKI pin
  policy, TLS 1.3 P-256 CertificateVerify policy, and explicit
  `pin_only_no_webpki_chain_validation` / `not_validated_stage0` chain/time
  policies
- provider snapshots and provider-minimal context now expose
  `raios.provider_trust_verifier_decision.v0` with verifier id, stage, outcome,
  and reason; no-pin/no-trust reports `pin_config` / `rejected` /
  `pin_config_missing`, and positive direct OpenAI pinned-trust markers bind the
  verified `certificate_verify` decision into request, export-audit, injection
  gate, and trust-evidence hashes
- the OpenAI SPKI verifier now supports one optional standby SPKI rotation pin
  supplied by `OPENAI_SPKI_SHA256_NEXT`; malformed rotation config fails closed,
  successful matches record the active or rotation pin id/slot, and the trust
  metadata still labels the path as pin-only without WebPKI chain or time
  validation
- the full Shadow VM provider-memory slice now expects all 20 provider context
  binding-gate selftest cases, including redaction/classification/budget/trust
  evidence hash mismatches, and the direct OpenAI smoke harness compares the
  trust evidence hash and verifier decision across positive request binding,
  export-audit binding, and blocked injection-gate markers when a local
  pinned-trust image is supplied
- `module.load_ephemeral svc.demo.hello` now loads/starts the built-in
  `svc.demo.hello` current-boot test service through a narrow RAM-only path
  that consumes `raios.current_boot_load_request.v0` and
  `raios.current_boot_load_descriptor.v0` from a validated current-image
  descriptor-source record
- `module.load_ephemeral host_bound:svc.demo.hello` loads/starts the same
  built-in RAM-only service through a host-produced descriptor-source candidate
  that binds the current-image source hash
- descriptor-source validation now parses the built-in source text into checked
  key/value fields for both current-image and host-bound sources instead of
  depending on a complete source-text equality check
- the current-image descriptor-source path now carries a repo-local
  P-256/SHA-256 signature envelope; the build script checks the checked-in
  public key/signature metadata, the kernel verifies the envelope before
  selecting the descriptor source, and load/inventory/health/RAM-audit evidence
  exposes the envelope id/hash and signature verification state
- `service.descriptor_source_trust_selftest` proves that the accepted envelope
  verifies and tampered payload, locator/kind, public-key hash, and signature
  cases fail closed without accepting descriptor or artifact bytes
- the built-in `builtin:svc.demo.hello` artifact now carries a signed
  `raios.builtin_artifact_identity.v0` identity/trust envelope; the build script
  checks the checked-in P-256 signature, the kernel validates it before load,
  and load/inventory/health/RAM-audit evidence exposes the identity id/hash,
  trust-envelope id/hash, signature verification state, and a signed
  `raios.builtin_artifact_content_binding.v0` content/hash binding for the
  checked-in Hello service source snapshot plus a signed repo-local artifact
  byte/reference hash for
  `seed-kernel/artifacts/svc.demo.hello.builtin.artifact`
- `service.artifact_reference_trust_selftest` proves that valid artifact
  reference evidence passes and tampered byte/content/reference/trust evidence
  fails closed without accepting artifact bytes or mutating the event log
- the Hello load path now emits
  `raios.current_boot_artifact_load_plan_preflight.v0`, binding the selected
  descriptor source, artifact identity, content binding, artifact reference,
  artifact bytes, and `ram_only:svc.demo.hello` service-slot intent into one
  accepted current-boot/local-only preflight hash visible in load, inventory,
  health, and RAM-audit evidence
- `service.artifact_load_plan_preflight_selftest` proves that valid preflight
  evidence passes and tampered descriptor/artifact/slot/denial evidence fails
  closed without mutating the event log
- the Hello load path now also emits
  `raios.ram_only_service_slot_activation.v0`, derived from the accepted
  preflight; load/start, inventory, health, stop/drop, and RAM-audit bindings
  expose activation id/hash/status/active state, and drop clears the current
  boot slot while citing the same activation hash
- the host-bound descriptor-source path remains hash-bound to the current-image
  source and does not accept arbitrary descriptor or artifact bytes
- `service.inventory` shows `svc.demo.hello` as healthy/running while loaded;
  `service.health svc.demo.hello` reports healthy, stopped, or missing from the
  same current-boot state; `service.stop svc.demo.hello` marks it stopped;
  `service.start svc.demo.hello` starts the stopped loaded generation,
  `service.restart svc.demo.hello` records its own restart lifecycle event while
  preserving that generation and activation hash, `service.hot_swap
  svc.demo.hello` validates the signed built-in v1 evidence chain before
  mutation, `service.hot_swap svc.demo.hello.v2` selects a distinct signed v2
  artifact identity with visible `version: "v2"` and its own
  identity/preflight/activation hashes, both accepted hot-swaps record lifecycle
  events and advance the loaded generation, `raios.ram_only_hello_service_state.v0`
  exposes a tiny current-boot counter in load, inventory, health, lifecycle,
  and audit records, and v1->v2 plus v2->v1 hot-swaps preserve that state
  through `raios.ram_only_hello_service_state_migration.v0` records while
  denying persistence, durable audit, and rollback install; `service.hot_swap
  svc.demo.hello.reset_state` computes a would-reset migration with
  `accepted: false` / `state_preserved: false`, records a local-only
  `capability_denied` lifecycle event, and proves the active descriptor,
  generation, state hash, and counter stay unchanged; `service.drop
  svc.demo.hello` removes it from inventory; the inventory and health records
  cite `load_descriptor.current_boot.svc.demo.hello.v0` plus the descriptor
  source locator/kind/validation/hash and bound source hash when present
- lifecycle and health actions retain
  `raios.ram_only_hello_service.lifecycle` and
  `raios.ram_only_hello_service.health` audit events in the current-boot RAM
  event log with descriptor and validated source-hash evidence
- the hello path accepts no arbitrary external artifact bytes, writes no
  persistent state, writes no durable audit log, installs no rollback plan, and
  grants no broad mutation
- wrong hello targets and external-looking hello targets remain on the denied
  module-load gate
- denied `module.load_ephemeral` / `service.load_ephemeral` remains the live
  policy surface for normal modules
- retained manifest, artifact, VM-test-report, local-attestation,
  local-approval, computed-grant, audit/rollback, service-slot, allocator, and
  loader-runtime evidence is current-boot, local-only, and non-authorizing
- the normal-module loader-runtime chain now reaches descriptor/artifact intake,
  execution authorization, service-registry mutation, live-load attempt,
  artifact-load, executable-mapping, entrypoint-transfer, service-start,
  service-health-binding, service-running-state, service-start-audit,
  service-unload-cleanup, live-load-commit, commit-audit, commit-rollback,
  commit-result, descriptor-acceptance authority, descriptor-parser contract,
  descriptor-parser result, descriptor schema-validation, descriptor
  capability-validation, descriptor load-plan, executable load-plan authority,
  executable load-plan result, executable image-layout, executable
  page-mapping plan, executable page-mapping, descriptor/executable-page
  binding, executable entrypoint binding, executable entrypoint transfer
  authorization, executable entrypoint transfer, executable entrypoint handoff,
  and executable entrypoint invocation boundaries
- all lifecycle boundaries report explicit non-authorizing reasons and keep
  descriptor intake, descriptor bytes, parsed descriptor production,
  validated descriptor production, descriptor schema validation, descriptor
  capability validation, capability-validated descriptor production,
  executable load-plan authority, executable load-plan production, executable
  image-layout production, executable page-mapping plan production, executable
  page mapping, capability-validated descriptor binding to executable pages,
  executable entrypoint binding, entrypoint transfer authorization, explicit
  entrypoint transfer, executable entrypoint handoff, executable entrypoint
  invocation, descriptor parsing, artifact bytes, artifact load, executable
  mapping, service start, health record creation, running-state marking,
  start-audit record writing,
  unload/cleanup, live-load commit, load-commit audit writing, commit rollback
  install, result recording, service inventory mutation, service-slot
  allocation, durable audit writes, rollback install, and load attempts false
- `agent command_envelope` now accepts schema
  `raios.agent_command_envelope.v0`, classification `local_only`, and the
  read-only target/capability pairs `system.describe` with
  `cap.system.describe.read`, `system.snapshot` with
  `cap.system.snapshot.read`, `system.boot_log` with
  `cap.system.boot_log.read`, `system.capabilities` with
  `cap.system.capabilities.read`, `device.graph` with
  `cap.device.graph.read`, `service.inventory` with
  `cap.service.inventory.read`, and `problem.list` with
  `cap.problem.list.read`; it emits a local-only
  `raios.agent_command_envelope.v0` response and routes to the existing
  dispatcher path. Bad-schema and over-capable envelope attempts are denied
  before dispatch; allowed read-only targets paired with the wrong allowed read
  capability are denied as `requested_capability_denied` before dispatch; and
  the boundary does not create a parallel dispatcher, provider write,
  candidate-byte load, persistence, durable audit write, rollback install, or
  broad mutation
- accepted, mismatched, bad-schema, and over-capable command-envelope decisions now retain
  current-boot/local-only `raios.agent_command_envelope.decision` events with
  `raios.agent_command_envelope.audit_binding.v0`; the envelope response
  carries matching `event_id`/`audit_event_id`, and `audit.events` proves the
  ten currently verified decision shapes
- accepted Hello hot-swaps now emit RAM-only
  `raios.ram_only_hello_service_hot_swap_probation.v0` evidence with
  `active_current_boot_probation` status, previous/new descriptor and artifact
  identity hashes, previous/new generation, preserved state hash/counter, and
  the accepted state-migration hash; the v1->v2 audit event retains the
  matching probation hash while candidate bytes, executable mapping,
  persistence, durable audit, rollback install, and rollback apply stay denied
- `service.rollback_preview svc.demo.hello` now reads retained hot-swap
  probation evidence into
  `raios.ram_only_hello_service_rollback_preview.v0`, exposes previous/current
  descriptor, artifact identity, generation, state hash/counter, and migration
  facts plus a preview hash, records a RAM-only rollback-preview audit event,
  and proves the active v2 service stays unchanged while rollback apply and
  durable/persistent/external execution surfaces stay denied
- `service.rollback_apply svc.demo.hello` now returns structured
  `capability_denied`, binds the current rollback-preview hash, probation hash,
  Hello state hash/counter, rollback target, current candidate, and migration
  hash, and now exposes
  `raios.ram_only_hello_service_rollback_transaction_preflight.v0` binding the
  apply-denial hash, requested capability, missing rollback-transaction,
  durable-audit-write, and persistent-install authorities, target/current
  descriptor and artifact identity facts, and no-side-effect flags, plus
  `raios.ram_only_hello_service_rollback_write_authority_gate.v0` binding the
  preflight hash, required audit/rollback-transaction schemas, unavailable
  durable-audit-write, rollback-store-write, and transaction-append authority,
  and disabled write/apply side effects, plus
  `raios.ram_only_hello_service_rollback_append_intent_gate.v0` binding the
  write-authority gate hash, preflight hash, apply-denial hash,
  preview/probation/state evidence, target/current candidate facts, required
  schemas, unavailable append/durable-store authority, and disabled
  append/write/apply side effects, plus
  `raios.ram_only_hello_service_rollback_payload_envelope_gate.v0` binding the
  append-intent gate hash, write-authority gate hash, preflight hash,
  apply-denial hash, preview/probation/state evidence, target/current candidate
  facts, proposed `raios.rollback_transaction.v0` payload schema/id/hash,
  payload provenance hash, required schemas, unavailable
  transaction-writer/durable-store authority, and disabled append/write/apply
  side effects, plus
  `raios.ram_only_hello_service_rollback_transaction_writer_storage_authority_gate.v0`
  binding the payload-envelope gate hash, payload/provenance hashes,
  append-intent gate hash, write-authority gate hash, preflight hash,
  apply-denial hash, preview/probation/state evidence, target/current candidate
  facts, required schemas, and unavailable transaction-writer,
  durable-audit-store, rollback-store, and append authority, plus the shared
  `raios.module_audit_rollback_append_contract.v0` foundation status for
  `storage.authority.audit_rollback.current_boot`,
  `append.audit_ledger.current_boot`, and `append.rollback_store.current_boot`
  as the `raios.rollback_transaction.v0` append target, plus
  `raios.audit_rollback_append_target_owner.v0` and
  `raios.audit_rollback_transaction_writer_readiness.v0` denied with
  `persistence_device_write_path_missing` after a current-boot
  `raios.pci_mass_storage_controller_probe.v0` observes QEMU AHCI at
  `00:1f.2` and `raios.ahci_controller_probe.v0` maps the ABAR, reads AHCI
  version/port registers, issues one read-only AHCI IDENTIFY DEVICE command on
  the active first SATA port, exposes QEMU HARDDISK identity, and completes one
  read-only Sector-0 read with MBR signature evidence plus empty MBR partition
  inventory plus `raios.read_only_block_driver.v0` readiness, then binds the
  missing media write path through `raios.block_write_path_authority_gate.v0`
  with `block_write_path.authority.audit_rollback.current_boot`,
  `block_driver.ahci.read_only.current_boot`, and `mbr_empty` partition
  evidence while still denying any media write or append authority; the storage
  authority now also emits
  `raios.audit_rollback_target_region_discovery.v0` with id
  `target_region.audit_rollback.current_boot`, source
  `dedicated_audit_rollback_label_scan`, status `available`, reason
  `dedicated_audit_rollback_region_discovered_read_only`, a read-only
  non-scratch LBA1/512-byte candidate region discovered from the
  `RAIOS_AUDITRB_V0` VM-harness label, scratch rejected as durable authority,
  no boot/partition-metadata or scratch overlap, and append/write authority
  still false; the Hello
  durable append-authority preflight and rollback-apply RAM audit binding retain
  that discovery under the same denied preflight hash while still opening no
  append/write authority; the
  shared writer-readiness path now also emits
  `raios.audit_rollback_transaction_writer_scratch_dry_run.v0` with id
  `transaction_writer.scratch_dry_run.audit_rollback.current_boot`, names the
  required audit-ledger and rollback-store record schemas, binds the verified
  scratch authority and LBA1/512-byte scratch target range, proves it is
  scratch-owned, within device bounds, and free of boot/partition metadata
  overlap, and still sets `authorizes_append: false`,
  `writes_durable_audit_log: false`, `writes_rollback_store: false`,
  `appends_rollback_transaction: false`, and `write_attempted: false`; the same
  readiness path now emits
  `raios.audit_rollback_target_region_writer_contract.v0` with id
  `target_region_writer_contract.audit_rollback.current_boot`, status
  `target_region_ready_not_write_authority`, reason
  `target_region_read_only_missing_media_write_authority`, the read-only
  non-scratch LBA1/512-byte target span and audit-ledger/rollback-store
  target ids/schemas, while keeping media write authority, durable audit policy,
  append authority, durable writes, rollback-store writes, transaction append,
  and write attempts false; nested under that contract,
  `raios.audit_rollback_target_region_media_write_policy_preflight.v0` now
  verifies the source contract plus owner/target/span/schema ids, reports
  missing media write authority and durable audit policy as structured denial
  facts, and still keeps media writes, append authority, durable writes,
  rollback-store writes, transaction append, and write attempts false; nested
  beside it, the Hello rollback durable append preflight and RAM audit binding
  now retain
  `raios.ram_only_hello_service_rollback_media_write_authority_gate.v0`, binding
  the durable append preflight hash, policy preflight hash, source-contract and
  target-span facts, missing media-write authority and durable-audit-policy
  reasons, and all media-write/append/durable-write/target-region-write flags
  false; the
  Hello-specific writer/storage gate now also emits
  `raios.ram_only_hello_service_rollback_append_record_dry_run.v0` with
  canonicalization `raios.rollback_append_record_image.canonical.v0`, audit
  record and rollback-transaction image hashes, exact byte lengths
  255/225/480, LBA1/512-byte target span, source payload/provenance hashes,
  and all append/write flags false; the
  RAM-only rollback-apply denial audit event retains the
  same preflight, write-authority gate, append-intent gate, payload-envelope
  gate, payload, provenance, writer/storage authority gate hashes, shared
  foundation status/reason fields, scratch writer dry-run binding fields,
  append-record dry-run image/hash fields, append-sector plan/image-hash
  fields, scratch append-sector write/readback planned/readback image hashes,
  durable writer-policy preflight fields, durable append/transaction
  authorization gate fields, append-engine readiness decision fields, durable
  append-authority decision fields, durable audit-policy decision and
  candidate fields with the no-write durable-audit, rollback-store, and transaction-append writer
  candidates accepted, readiness `available` /
  `transaction_append_engine_ready` / `ready: true`, durable audit policy and
  durable append authority still missing, and
  block-write-path gate binding fields and proves the active v2
  descriptor, generation, running state, and
  RAM-only
  state stay unchanged while real rollback application, persistence, durable
  audit writes, rollback-store writes, transaction append, external bytes,
  candidate execution, executable mapping, provider auto-load, and broad
  mutation stay denied

Previous full verification before the verifier-decision slice:

```text
release\vm-reports\shadow-20260702-042431-24536.json
6632/6632 predicates, 243 executed commands, duration_ms: 609828
```

Latest full verification after adding target-region discovery:

```text
release\vm-reports\shadow-20260702-174421-7208.json
6789/6789 predicates, 243 executed commands, duration_ms: 553963
report SHA-256 80be25579114eb7f23e7501134948e6a36728b4af258e44280968c2f8ccf77ea
```

Previous full verification after adding the durable append-authority preflight:

```text
release\vm-reports\shadow-20260702-171942-19692.json
6780/6780 predicates, 243 executed commands, duration_ms: 559199
report SHA-256 a119e937b60e85868ee6e2cba6461787e5426e72db2cfea1a9523c13d704374b
```

Previous full verification after adding the block write-path authority gate:

```text
release\vm-reports\shadow-20260702-132452-3944.json
6721/6721 predicates, 243 executed commands, duration_ms: 544891
report SHA-256 0040df230ad765590bf3d28704af1eedca4ece87a41a0d71700d176ad82a8938
```

Previous full verification after adding read-only block-driver readiness:

```text
release\vm-reports\shadow-20260702-130028-34200.json
6706/6706 predicates, 243 executed commands, duration_ms: 541557
report SHA-256 8a864a261d52ff2fdd8dc2afc3ac6ab163d12f26c0521baddae4a3455fa754d6
```

Previous full verification after adding read-only MBR partition inventory:

```text
release\vm-reports\shadow-20260702-124055-34392.json
6698/6698 predicates, 243 executed commands, duration_ms: 538777
report SHA-256 cf686419c69dfee6052c7c3efb81bcb604da7feaec5f99e0f274d73e13f1a39c
```

Previous full verification after adding read-only AHCI Sector-0 evidence:

```text
release\vm-reports\shadow-20260702-122430-29116.json
6691/6691 predicates, 243 executed commands, duration_ms: 533741
report SHA-256 0ec4661050cbb927f9760ebcc5e3683dc56708c1cd9fc827ae105a7e5c10172c
```

Previous full verification after the explicit hello `service.start` slice:

```text
release\vm-reports\shadow-20260702-053820-28640.json
6640/6640 predicates, 243 executed commands, duration_ms: 610100
```

Latest focused verification after adding target-region discovery:

```text
release\vm-reports\shadow-20260702-174245-3640.json
297/297 quick predicates, 56 executed commands, duration_ms: 84876
report SHA-256 c4e00135dcfc66cbb1cc23898197ee882fbaa0dceefc8d20bb0f6c800c4c3b40
```

Previous focused verification after adding the durable append-authority preflight:

```text
release\vm-reports\shadow-20260702-171752-2052.json
293/293 quick predicates, 56 executed commands, duration_ms: 83559
report SHA-256 80e2af5244c0c9c88522ebf76df65170235bbd412ce039e178ceca60db55489d
```

Previous focused verification after adding read-only block-driver readiness:

```text
release\vm-reports\shadow-20260702-125633-29632.json
277/277 quick predicates, 56 executed commands, duration_ms: 75446
report SHA-256 59841bb5cf9682c93ff7ed50b01afa697a00f3c082c9e5410ec0d0b07edd8018
```

Previous focused verification after adding read-only MBR partition inventory:

```text
release\vm-reports\shadow-20260702-123758-19068.json
277/277 quick predicates, 56 executed commands, duration_ms: 82906
report SHA-256 f2aade1a2a1f9d7cbb0d470bba5a0a1a431319ce37454b3161b10bf787c59849
```

Previous focused verification after adding read-only AHCI Sector-0 evidence:

```text
release\vm-reports\shadow-20260702-121906-31632.json
277/277 quick predicates, 56 executed commands, duration_ms: 73721
report SHA-256 5b8eb2c9d69d5cf618a35b9d973faf91a8f1e6bd1043ce73d72c4b7ff2aaa919
```

Previous focused verification after adding read-only AHCI IDENTIFY DEVICE evidence:

```text
release\vm-reports\shadow-20260702-121149-29136.json
277/277 quick predicates, 56 executed commands, duration_ms: 72693
report SHA-256 64744c702d5df3244e7b8f25c92d1b8acf6f954b939ec38640e35fb5a6421054
```

Previous focused verification after adding read-only AHCI IDENTIFY DEVICE evidence before
the storage selftest fixture fix:

```text
release\vm-reports\shadow-20260702-115628-29252.json
277/277 quick predicates, 56 executed commands, duration_ms: 75877
report SHA-256 1f449f0c39ac68bf70f20c89283223c0d3cbe9671bb533f4c93b62af552aacc4
```

Previous focused verification after adding read-only AHCI controller/port evidence:

```text
release\vm-reports\shadow-20260702-114454-34128.json
277/277 quick predicates, 56 executed commands, duration_ms: 74270
report SHA-256 f7dbbaae80f16443bf53c3f78f47a3157d15e0752119182f5f528fa223313682
```

Previous focused verification after adding the PCI mass-storage controller probe:

```text
release\vm-reports\shadow-20260702-113454-11512.json
277/277 quick predicates, 56 executed commands, duration_ms: 82585
```

Previous focused verification after binding Hello rollback to concrete writer/storage authority IDs:

```text
release\vm-reports\shadow-20260702-111358-16356.json
274/274 quick predicates, 55 executed commands, duration_ms: 126008
```

Previous focused verification after adding concrete writer/storage authority IDs:

```text
release\vm-reports\shadow-20260702-105942-11872.json
273/273 quick predicates, 54 executed commands, duration_ms: 81508
```

Previous focused verification after binding Hello rollback to the shared writer/storage foundation:

```text
release\vm-reports\shadow-20260702-104527-2836.json
271/271 quick predicates, 54 executed commands, duration_ms: 80942
```

Previous focused verification after the Hello rollback payload-envelope gate:

```text
release\vm-reports\shadow-20260702-091057-17852.json
266/266 quick predicates, 54 executed commands, duration_ms: 83454
```

Previous focused verification after the Hello rollback append-intent gate:

```text
release\vm-reports\shadow-20260702-090105-12232.json
263/263 quick predicates, 54 executed commands, duration_ms: 84226
```

Previous focused verification after the Hello rollback write-authority gate:

```text
release\vm-reports\shadow-20260702-085049-8956.json
260/260 quick predicates, 54 executed commands, duration_ms: 72086
```

Previous focused verification after the Hello rollback transaction/durable-audit
preflight:

```text
release\vm-reports\shadow-20260702-084240-14784.json
257/257 quick predicates, 54 executed commands, duration_ms: 73613
```

Previous focused verification after the fail-closed Hello rollback-apply gate:

```text
release\vm-reports\shadow-20260702-082918-20728.json
254/254 quick predicates, 54 executed commands, duration_ms: 77410
```

Previous focused verification after the read-only Hello rollback preview:

```text
release\vm-reports\shadow-20260702-081302-27580.json
247/247 quick predicates, 52 executed commands, duration_ms: 81372
```

Previous focused verification after accepted Hello hot-swap probation evidence:

```text
release\vm-reports\shadow-20260702-075957-15956.json
243/243 quick predicates, 50 executed commands, duration_ms: 77226
```

Previous focused verification after the fail-closed Hello reset-state migration gate:

```text
release\vm-reports\shadow-20260702-074900-3852.json
241/241 quick predicates, 50 executed commands, duration_ms: 79318
```

Previous focused verification after the Hello state migration slice:

```text
release\vm-reports\shadow-20260702-073742-10256.json
237/237 quick predicates, 48 executed commands, duration_ms: 74841
```

Previous focused verification after the signed Hello v2 hot-swap slice:

```text
release\vm-reports\shadow-20260702-072537-6980.json
235/235 quick predicates, 48 executed commands, duration_ms: 75346
```

Previous focused verification after the Hello hot-swap slice:

```text
release\vm-reports\shadow-20260702-071540-15484.json
231/231 quick predicates, 46 executed commands, duration_ms: 72031
```

Previous focused verification after the `system.boot_log` command-envelope slice:

```text
release\vm-reports\shadow-20260702-070530-20712.json
226/226 quick predicates, 43 executed commands, duration_ms: 69882
```

Previous focused verification after the `device.graph` command-envelope slice:

```text
release\vm-reports\shadow-20260702-065801-25136.json
224/224 quick predicates, 42 executed commands, duration_ms: 95235
```

Previous focused verification after the `system.capabilities` command-envelope slice:

```text
release\vm-reports\shadow-20260702-065202-7476.json
222/222 quick predicates, 41 executed commands, duration_ms: 95768
```

Previous focused verification after the `system.snapshot` command-envelope slice:

```text
release\vm-reports\shadow-20260702-064636-24876.json
220/220 quick predicates, 40 executed commands, duration_ms: 67908
```

Previous focused verification after the `problem.list` command-envelope slice:

```text
release\vm-reports\shadow-20260702-063508-18024.json
219/219 quick predicates, 40 executed commands, duration_ms: 94555
```

Previous focused verification after the command-envelope mismatch denial slice:

```text
release\vm-reports\shadow-20260702-063057-5156.json
217/217 quick predicates, 39 executed commands, duration_ms: 67751
```

Previous focused verification after the `service.inventory` command-envelope slice:

```text
release\vm-reports\shadow-20260702-062447-11572.json
214/214 quick predicates, 38 executed commands, duration_ms: 92411
```

Previous focused verification after the agent-command envelope audit slice:

```text
release\vm-reports\shadow-20260702-061909-10304.json
212/212 quick predicates, 37 executed commands, duration_ms: 65117
```

Older focused verification after the first agent-command envelope slice:

```text
release\vm-reports\shadow-20260702-061129-8152.json
207/207 quick predicates, 36 executed commands, duration_ms: 64609
```

Previous focused verification after the explicit hello `service.restart` slice:

```text
release\vm-reports\shadow-20260702-055608-6288.json
203/203 quick predicates, 33 executed commands, duration_ms: 62948
```

Previous focused verification after the explicit hello `service.start` slice:

```text
release\vm-reports\shadow-20260702-053445-10792.json
201/201 quick predicates, 32 executed commands, duration_ms: 61451
```

Latest focused verification after the artifact identity slice:

```text
release\vm-reports\shadow-20260702-021750-7868.json
172/172 quick predicates, 29 executed commands, duration_ms: 51268
```

Latest focused verification after the artifact content binding slice:

```text
release\vm-reports\shadow-20260702-022858-26440.json
174/174 quick predicates, 29 executed commands, duration_ms: 51671
```

Latest focused verification after the artifact byte/reference slice:

```text
release\vm-reports\shadow-20260702-023832-25068.json
177/177 quick predicates, 29 executed commands, duration_ms: 53786
```

Latest focused verification after the artifact-reference trust selftest:

```text
release\vm-reports\shadow-20260702-025252-23928.json
178/178 quick predicates, 30 executed commands, duration_ms: 53295
```

Latest focused verification after the artifact load-plan preflight:

```text
release\vm-reports\shadow-20260702-030513-27840.json
181/181 quick predicates, 30 executed commands, duration_ms: 59868
```

Latest focused verification after the artifact load-plan preflight selftest:

```text
release\vm-reports\shadow-20260702-032107-16036.json
182/182 quick predicates, 31 executed commands, duration_ms: 38186
```

Latest focused verification after the service-slot activation slice:

```text
release\vm-reports\shadow-20260702-033352-9800.json
185/185 quick predicates, 31 executed commands, duration_ms: 60174
```

Latest focused verification after the provider context hash-binding slice:

```text
release\vm-reports\shadow-20260702-034303-24400.json
191/191 quick predicates, 31 executed commands, duration_ms: 60437
```

Latest focused verification after adding the Hello rollback
media-write-authority gate over the target-region policy preflight:

```text
release\vm-reports\shadow-20260702-192027-4988.json
306/306 quick predicates, 56 executed commands, duration_ms: 67009
report SHA-256 cb77021e5e6aec6ac3b9fe919c1777a2bfc684fb8bb41ef297d451acc6a1290e
base image SHA-256 23aa783ada0d690c94c09b9167c1129785b4d42b10c39db729917a78ad3c08dd
```

Previous focused verification:

```text
release\vm-reports\shadow-20260703-020305-30364.json
149/149 hello-rollback-dry-run predicates, 19 executed commands, duration_ms: 50434
report SHA-256 1af9a9b43bd9f2b3049a13e2ad68c912b34071b4eabb8454115e08359274bdad
base image SHA-256 9cec085cdfbd547679fc168605a806cb4a22ad42c1d6ebd3c58fc4a9e9d38249
```

Latest focused verification:

```text
release\vm-reports\shadow-20260703-052003-28604.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 82145
report SHA-256 e4f509a05e47b1ecab54852c9a2ded7ff16607f80473ff32d3bc545aceebd9ef
base image SHA-256 98afce8ca591e11bc6e5db4e89ab6cd4e6311d1a142e731f926439c7f4e90327
```

Latest recovery harness verification for the source-bound side-effect gate
image:

```text
release\vm-reports\shadow-20260703-072638-30256.json
2799/2799 recovery predicates, 142 executed commands, duration_ms: 222896
report SHA-256 4f4df2c6c44f4c5a75d63d30e6bcf4ff5b54091eb6ac720a48c4039fed4a3751
base image SHA-256 dc8db0b397ed84915f36cb759a4a971ef09ab7c04413fd60e376b1834f096fb7
```

Latest focused Hello rollback verification after that recovery binding:

```text
release\vm-reports\shadow-20260703-073031-21880.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 82102
report SHA-256 4bb4d16d1b1ebdfbf06d42951980f1b807b01482c0622c29e5342762d6a0a91c
base image SHA-256 363327e257b448eee58263cdfa23f03d5c15fe672e72234982e91cb544284bb9
```

Previous recovery harness verification for the source-bound executor
capability-table image:

```text
release\vm-reports\shadow-20260703-071243-14856.json
2793/2793 recovery predicates, 142 executed commands, duration_ms: 223571
report SHA-256 7038b842c55a30442dce3af0629d91c6cecec0f4299e5d759808975186f12699
base image SHA-256 023fe7ef056f99ac2fd53e470181ce4575488d844109812c9f432449328ec709
```

Previous focused Hello rollback verification after that recovery binding:

```text
release\vm-reports\shadow-20260703-071631-28124.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 81316
report SHA-256 c2330af8ff9331c3f30d1d110518fa6f4abf58f447d0abf26eae8f53d04595b6
base image SHA-256 644997917a4c6f3f5472eda1f8e3947c82b476d4757d744787d4acb9679f7401
```

Previous recovery harness verification for the source-bound command-dispatch
behavior image:

```text
release\vm-reports\shadow-20260703-070020-19184.json
2787/2787 recovery predicates, 142 executed commands, duration_ms: 222220
report SHA-256 57424c7ff566d505cf012ed785e2b02fcd04d6f8aeed6e6b5837af90b09e0403
base image SHA-256 62550e2f675e0dc38e3f974d040c47a3d382d9ba6a5658ce673021e34b140770
```

Previous focused Hello rollback verification after that recovery binding:

```text
release\vm-reports\shadow-20260703-070408-13684.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 80756
report SHA-256 d805c998b68b80f6e68d1dce958ea752bd5eb262b717c2c843e930bce70a83f0
base image SHA-256 7cf1beab1c349a2b1f369bb584eb6349a9967b925526522d3c6cf45fbc179f62
```

Previous recovery harness verification for the source-bound service-inventory
side-effect boundary image:

```text
release\vm-reports\shadow-20260703-064708-18720.json
2781/2781 recovery predicates, 142 executed commands, duration_ms: 223126
report SHA-256 5521c70ec182d5f37dd67d0041e422a8ecad92cb745522470701455f79591ff1
base image SHA-256 d11b8d5ff5cc2bae433664730c6997e4cf4d7046dc7c82240263cee6ee1de3a6
```

Previous focused Hello rollback verification after that recovery binding:

```text
release\vm-reports\shadow-20260703-065115-22780.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 81111
report SHA-256 81212860e351693cc7882e162eadb8a7bda926f37d46e0065a2f0cc3d8ebe74d
base image SHA-256 93a22948850c4a21bc38ac26f3e05387a13f5cf7046b3c9caffc53dd07904ea9
```

Previous recovery harness verification for the source-bound durable
audit/rollback write-authority image:

```text
release\vm-reports\shadow-20260703-063434-2636.json
2775/2775 recovery predicates, 142 executed commands, duration_ms: 222507
report SHA-256 4d23b1b68f92e3168c531723c99f3b74c33bb3033340db2eb202c3cd257c4c9b
base image SHA-256 ac521d5ac0f623076166024c90c55176cfa520787bd378e0586ce960f50cfb4c
```

Previous focused Hello rollback verification after that recovery binding:

```text
release\vm-reports\shadow-20260703-063821-29368.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 81653
report SHA-256 01303019d31c0447dd0aa909786b89d342e1ccb5b09ca4ee105aa44356e90518
base image SHA-256 e3c1a339c21d10ff829aa4abb336162201f078daaff72e18aa675136c8e44488
```

Previous recovery harness verification for the source-bound recovery
memory-write-authority image:

```text
release\vm-reports\shadow-20260703-062243-28232.json
2769/2769 recovery predicates, 142 executed commands, duration_ms: 221548
report SHA-256 4d5507516041dfa7654a128da78ae80a43e85e2e114f6d77c6b8a035c34aeb85
base image SHA-256 9d267ad6075e1b103c12f1c584c5b2e9ce4108b9ef043db3c14109e33cf6018e
```

Previous focused Hello rollback verification after that recovery binding:

```text
release\vm-reports\shadow-20260703-062629-22400.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 82030
report SHA-256 13b3dff9dcec68f8f59ddc7f6fd665748018a9e2b9de468b1af67dfa90b3b722
base image SHA-256 7f9b52526e35097967317fb8a2672c236314caf50410f91a35f91631ab63fe65
```

Previous recovery harness verification for the source-bound
load-artifact-by-hash target binding image:

```text
release\vm-reports\shadow-20260703-061056-21276.json
2763/2763 recovery predicates, 142 executed commands, duration_ms: 220839
report SHA-256 115699853c24f6fe1a7f68ea1386b57a6abbd93339a1e384797913bb3c890dbf
base image SHA-256 e63ef69837fa2cbf3ec422894e067a6e3f5a8725cd62acb8f319279db817df6c
```

Previous focused Hello rollback verification after that recovery binding:

```text
release\vm-reports\shadow-20260703-061447-20744.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 81218
report SHA-256 53f13ad3eb2facdeba9fa688fa98c4016278488415f65d5bfcf057ee95bfe53b
base image SHA-256 2e31653470f16be41fe3670b68fc6b534986f0ebc87eb87ee909296153ee1315
```

Previous recovery harness verification for the source-bound restart-last-good
target binding image:

```text
release\vm-reports\shadow-20260703-055846-23996.json
2757/2757 recovery predicates, 142 executed commands, duration_ms: 221888
report SHA-256 55a4f41e0d3d024c304d74feea6255b97815073fd7452cf60fef5238d462e798
base image SHA-256 a4fadf358c9b10754cd3ff6b58addbdd3a2ca37bed1e260cccbba96a90c393cc
```

Previous focused Hello rollback verification after that recovery binding:

```text
release\vm-reports\shadow-20260703-060242-30420.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 82333
report SHA-256 a116554104bcf28a257081f6db12d6a8d6cc4a8ec4b0ef0faa0f52b8194d3975
base image SHA-256 5842556d057806890d20bd5755424e03de26c4fb3e7a14e20338c667b044f2b2
```

Previous recovery harness verification for the source-bound disable-module
target binding image:

```text
release\vm-reports\shadow-20260703-054624-28472.json
2751/2751 recovery predicates, 142 executed commands, duration_ms: 221169
report SHA-256 6a19cd7f27603d183049300a63c6c6de90b6e7dbd8691768cf5cec0a253479bf
base image SHA-256 938e1bc95d4308f2b9c3e6627ae1b4e640e6630e1bde3de9f991414d1ddbebbc
```

Previous focused Hello rollback verification after that recovery binding:

```text
release\vm-reports\shadow-20260703-055018-27340.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 81969
report SHA-256 1a519d4da662c3fe4342102f6d6bace8ca3eddeea2e324c265e32770bb2366ca
base image SHA-256 60e7daa38f70c64f5bb7613fe07b7faedba62608ac51e744eefd19044656d0a2
```

Previous quick harness verification for the sourced rollback-apply denial image:

```text
release\vm-reports\shadow-20260703-052134-30240.json
404/404 quick predicates, 59 executed commands, duration_ms: 148672
report SHA-256 8cf2496d99becbbeaaa73632c5b56fe39690ad67d8a8c34daf4a97fc573dee57
base image SHA-256 6b543c73b0a7a2fe2a37a4fbd12759b0695e418df095cc4c7ae8885f3763bce0
```

Latest Hello rollback command-envelope preview verification:

```text
release\vm-reports\shadow-20260703-144044-22112.json
197/197 hello-rollback-dry-run predicates, 26 executed commands, duration_ms: 100716
report SHA-256 e0acd8bd07e4abda57d9f95fdcd641e91093d9b05f089347466a5c310a6c8122
base image SHA-256 61692613d30515dfb7b57b22ec66d7761181816f1b9b40cb9b1a0a4fdf26a040
```

Previous provider-memory/full verification cleanup:

```text
release\vm-reports\shadow-20260703-142658-22296.json
258/258 provider-memory-full predicates, 21 executed commands, duration_ms: 86345
report SHA-256 b5c69047eb76cfc22f1fac5f0d17f4e4c623937920b6c0ff5a8d00791a489fce
base image SHA-256 b470ba5d2bc3b275c09d7ceda6c63e748921a17660c2fd9718f7ff8697b3dab5
```

Previous provider context injection-gate omission negative selftest verification:

```text
release\vm-reports\shadow-20260703-141539-23964.json
169/169 provider-memory predicates, 12 executed commands, duration_ms: 73231
report SHA-256 bfe3f93b043a5385bee753f988f9e64c12b8e4539ae5ccf8bf807a0e5a361cc6
base image SHA-256 1df763d1db5798468eb23dd3bdcca2c8a4ad0dc4e3a35dfe2c7fd1c34c7fd6c5
```

Previous provider context injection-gate recovery-status omission verification:

```text
release\vm-reports\shadow-20260703-140416-24496.json
159/159 provider-memory predicates, 11 executed commands, duration_ms: 62815
report SHA-256 f971fbb182fce2574843b5950076c694a240868b46491161f5c3b1b38de24fd0
base image SHA-256 c4d8a6f9ba317213d59b71aa75b63b9b9c298f6e26853d56a84d1efbdf4d2182
```

Latest provider context gate/export recovery-status omission verification:

```text
release\vm-reports\shadow-20260703-135340-24628.json
417/417 quick predicates, 59 executed commands, duration_ms: 169446
report SHA-256 dee6d94fd7c865abe13760c6261927e0711c6970f6e2a6fee47e074d8674104b
base image SHA-256 d5148c70ec56787f65f9642c71bb68fc83369d9b8d004a33f8aaf8e36cce2c11
```

Latest provider-minimal recovery status omission verification:

```text
release\vm-reports\shadow-20260703-134738-27480.json
415/415 quick predicates, 59 executed commands, duration_ms: 185349
report SHA-256 768419ef893172f42fbab42a3bede18b90ca3e02054834828af7bc2a3973d615
base image SHA-256 d1dc57bda91e4cc1becaff02e249a8218d3af9f359b35655b8ed421fd21e6886
```

Latest recovery agent-context status fact verification:

```text
release\vm-reports\shadow-20260703-133613-9360.json
3634/3634 recovery predicates, 184 executed commands, duration_ms: 311351
report SHA-256 2a7253e34aea055bdfcdd05a8772e548ef3bf3e1b776aad8b8050dca0d5c0771
base image SHA-256 dc9d1aab855f39161413bde3e612b1dc0d6b24f805349463d22771d901801399
```

Previous recovery command-envelope status-read verification:

```text
release\vm-reports\shadow-20260703-132135-17152.json
3623/3623 recovery predicates, 181 executed commands, duration_ms: 313612
report SHA-256 c0213b4df9f7578e0d6e12b56e57abb44b77d66b44d6108b31870edd0d6fa7eb
base image SHA-256 a7e4d985fd440b08cd57658864a5593c64c966f8d226ed8e0e2ea7be2cc363e7
```

Latest recovery execution-stage source-diagnostic verification:

```text
release\vm-reports\shadow-20260703-121942-14876.json
3478/3478 recovery predicates, 172 executed commands, duration_ms: 297242
report SHA-256 e2ca038c5d743612109c1eab3a6e4c38cf77e8e1228f7e01ef97b3db1d63e1b4
base image SHA-256 55e11a8783b89996ee79894b4d2171299d6e2e25f4e5ca78a2e84b409d5319a2
```

Latest hello rollback guard:

```text
release\vm-reports\shadow-20260703-074606-9508.json
181/181 hello-rollback-dry-run predicates, 24 executed commands, duration_ms: 83725
report SHA-256 e0e0e287da3e936546abc696dfacc04c165e179421ce33e7436f92b5eedb4495
base image SHA-256 9fdc804342b97fe26a8c90862a8f4d23a43803cf7ef5c10f8ae2982e322e4558
```

Exact next task:

```text
Repair the full Shadow VM checkpoint harness around the non-terminal module
load-gate audit scrape. Preserve the full-audit module source-method evidence,
but avoid depending on one giant mid-profile `agent audit.events 256` response
that closes the serial path before recovery and Hello checks can continue.
Prefer splitting audit evidence by ownership boundary or moving bounded audit
checks closer to the records they prove. Do not add runtime schemas, relax
predicates, or grant authority. Verify the repair with the full profile using
delayed serial writes, while keeping persistence, durable audit writes,
rollback-store writes, transaction append, rollback application, external
unsigned artifact intake, executable candidate-byte mapping, provider
auto-load, broad mutation, and installed rollback state denied. After the full
checkpoint is green, resume the smallest runtime/recovery behavior slice that
moves live-load, recovery-lifeline, durable-audit, or rollback behavior toward
real authority on the final architecture path.
```

AI-parallel next wave:

1. VM harness/evidence track: repair the full-profile audit scrape so module,
   recovery, and Hello evidence can be checkpointed in one run without weakening
   predicates.
2. Provider trust/context track: harden the direct provider path toward
   SPKI/WebPKI trust and keep context injection gated by typed request/export
   authorization evidence; do not claim WebPKI/time validation before trusted
   roots, intermediate-chain handling, and trusted time exist.
3. Runtime artifact track: keep the Hello activation record green; only add
   narrow follow-ups that prove cleanup or trust evidence without executing
   candidate bytes.
4. UI/input track: improve response wrapping, scrolling, and settings controls
   while keeping UI state derived from typed system facts.
5. VM harness/evidence track: keep focused smokes fast and add predicates only
   when they prove positive behavior or necessary fail-closed denials.
6. Recovery/persistence track: keep lifeline, durable audit, rollback, and
   persistence designed from the final trust model; do not implement fake
   persistence or rollback before the evidence chain exists.

Only after provider trust/context and the live-load execution/audit/rollback
evidence chain are real should a later integration cursor consider loading
candidate bytes. Execution must stay built-in/current-boot until those gates
exist.

Documentation ownership:

- `README.md`: product thesis, quickstart, concise current reality only.
- `AGENTS.md`: stable startup checklist, standing engineering rules, and durable
  current facts only.
- `docs/ROADMAP.md`: phase direction and compact active cursor only.
- `docs/PROJECT_STATUS.md`: authoritative detailed status, exact next task,
  verification evidence, known gaps, and unabridged implementation history.
- `docs/DEBUGGING.md`: commands, smoke profiles, protocol probes, and failure
  modes.

Current blockers and non-goals:

- Do not add fake persistent memory. V0 memory is `current_boot` and read-only.
- Do not send raw `system.snapshot` or boot logs to a provider.
- Do not grant module/service/config mutation before the evidence chain exists.
- Do not add another non-authorizing loader boundary before the hello-service
  slice unless it is the smallest blocker for load/start/list/stop/drop.
- Do not treat the direct OpenAI provider path as the recovery lifeline.
- Do not overwrite `release/raios-stage0.img` unless the replacement has booted
  in QEMU.

## Product Thesis

raiOS should be a tiny bootable environment whose primary interface is an
AI agent host. The OS should be small enough to understand, boot quickly in a VM,
and expose narrow, auditable capabilities to an AI provider through native
provider adapters.

This is not a Linux distribution and not a place to run the full Codex CLI in the
kernel. Codex is useful as a development tool and as a product reference; the OS
should implement its own minimal protocol surface.

## North Star Architecture

The long-term target is stronger than a small OS with a provider client. raiOS
should become an always-on core plus a live-rebuildable world:

```text
permanent core -> recovery agent lifeline -> live service graph
-> agent workspace -> shadow VM/test world -> persistence/rollback
```

The permanent core should only contain the survival mechanisms: minimal
scheduling, memory/object ownership, IPC, capabilities, service loading, crash
detection, rollback supervision, root system snapshots, and a tiny recovery
control path.

The normal OS surface should be replaceable services: UI, console, input, USB,
networking, Wi-Fi, provider adapters, diagnostics, agent tools, builder service,
and eventually driver experiments. The provider/OpenAI path is therefore a
service, not the core identity of the OS.

System memory is part of this north star. raiOS should not grow a large prompt
dump or generic RAG database. It should make the system itself the memory:
typed facts, events, decisions, problems, capability denials, service state,
test evidence, and rollback records with provenance. Agents should receive
task-scoped `agent_context.v0` packets selected by a local context broker under
token, redaction, and provider-trust budgets. See
`docs/architecture-decisions/0004-system-memory-and-agent-context.md`.

For the final system, most evolution should happen without a visible reboot:

```text
load service v2 next to v1
migrate state
switch handles
watch health
rollback to v1 if needed
persist only after tests and approval
```

If the live world crashes, the core should still be able to report a snapshot,
disable bad modules, restart last-good services, roll back persistent state, and
use a protected recovery agent lifeline. See
`docs/architecture-decisions/0003-always-on-core-and-live-rebuildable-world.md`.

## Planning Gates

The current Stage-0 code proves that direct provider access is possible, but it
does not yet prove the live-rebuildable architecture. The next planning gates are
therefore intentionally narrow:

```text
fail-closed TLS/provider trust
-> read-only agent protocol
-> typed system.snapshot.v0
-> static service.inventory.v0
-> capability policy v0
-> read-only memory.context over real typed facts
-> RAM-only event.log.v0 over reads and denials
-> module_manifest.v0
-> vm_test_report.v0
-> raios.local_attestation.v0
-> live loading remains denied until evidence matches
```

The direct OpenAI path is a normal provider-service candidate. It is not the
recovery lifeline and must not become the trusted control plane for persistence,
OTA, or recovery without the separate gates above.

## Phase 0: Bootable Visual MVP

Status: done for the current VM MVP.

Goal:

```text
UEFI -> Limine -> Rust kernel -> framebuffer overlay -> serial diagnostics
```

Done:

- Limine UEFI boot path working.
- Higher-half kernel linking fixed.
- Limine HHDM request available for kernel mappings.
- Limine framebuffer request working.
- Direct framebuffer drawing working.
- Serial diagnostics working.
- RDRAND entropy path working in the bare-metal-style VM profile.
- Chat-first double-buffered framebuffer UI with compact status for entropy,
  USB-xHCI, network, and input.
- Minimal Windows image packaging path.

## Phase 1: Minimal Agent Host UI

Goal:

```text
Boot -> status UI -> command input -> visible responses
```

Scope:

- framebuffer text UI
- serial command input (`help`, `status`, `devices`, `log`)
- optional keyboard input
- device/status model in memory
- commands: `help`, `status`, `devices`, `log`

Definition of done:

- QEMU window shows live state, not only a fixed splash.
- Serial input can request status.
- State transitions are mirrored in serial logs.

Current status: framebuffer UI, serial commands, entropy, e1000 network
bring-up, DHCP configuration, USB keyboard input, and USB mouse input are
implemented. The remaining work here is mostly UI polish and richer command
behavior.

## Phase 2: Network Visibility

Goal:

```text
e1000 visible -> DHCP attempt -> IP/DNS/gateway state shown
```

Scope:

- network status in UI
- DHCP progress and timeout states
- packet counters
- DNS stub visibility if already present in code

Definition of done:

- UI shows whether network is unavailable, probing, configured, or failed.
- Serial log gives enough data to debug without a graphical screenshot.

Current status: QEMU user-mode DHCP configures `10.0.2.15/24`, gateway
`10.0.2.2`, and DNS `10.0.2.3` locally. Packet counters, failure/timeout states,
and DNS command visibility remain.

## Phase 3: Direct Provider Transport With Trust Gate

Goal:

```text
VM agent protocol -> in-OS DNS/TCP/TLS/HTTPS -> provider API -> verified peer
```

Scope:

- tiny provider request state machine inside Stage-0
- DNS/TCP visibility for provider endpoints
- TLS/HTTPS client small enough to audit
- fail-closed certificate verification or provider/SPKI pinning
- API key entry in RAM first, stronger storage later
- every agent action maps to an explicit tool/capability

Definition of done:

- VM can submit a prompt to the provider without a host-side helper.
- The normal provider path does not use certificate verification bypass.
- Provider trust state is visible through status/snapshot output and VM smoke
  tests check for a verified or pinned TLS marker.
- The framebuffer and serial console show missing-auth, network, TLS, and
  provider errors clearly.

Current status: the host relay has been removed from the runtime path. The VM
command `ask <text>` stays in the guest and fails closed in the normal build
when provider trust is not positively verified. The default visible trust state
is `pin_config_missing`, and the Shadow VM smoke checks that problem. The first
positive verifier slice is implemented for OpenAI SPKI SHA-256 pinning: a local
image built with `-EmbedOpenAiSpkiPinFromEnv` checks the configured pin and the
TLS 1.3 P-256 ECDSA `CertificateVerify` proof before API key copy or HTTPS
write, and `openai-direct-smoke.ps1 -ExpectSpkiPinnedTrust` verifies the marker.
The earlier leaf-certificate SHA-256 pin path remains available through
`-EmbedOpenAiCertPinFromEnv` and `-ExpectPinnedTrust` for compatibility. A local
development image built with
`-AllowUnverifiedOpenAiTls` can still exercise the old unverified path for
transport debugging only. WebPKI, broader certificate algorithm support, and
redacted context projection remain the next trust hardening gates before
provider context injection, tool schemas, or capability policy can be treated as
safe.

## Phase 4: Provider Integration And Redacted Context

Goal:

```text
Prompt + redacted read-only context -> provider adapter -> response rendered in raiOS
```

Scope:

- provider config flow
- OpenAI/ChatGPT/Codex-style adapter first
- API key/pairing handled through a visible VM flow first, with persistence and
  stronger secret storage later
- rendered response in framebuffer UI
- `system.snapshot.v0` context may be attached only after TLS trust and field
  redaction are defined
- no mutating provider tools in this phase

Definition of done:

- User can boot the VM and get one AI response rendered in the OS.
- Failure modes are visible: missing auth, network unavailable, provider error.
- Snapshot fields that can leave the machine are classified as `public`,
  `local_only`, or `secret`, and provider requests include only explicitly
  allowed redacted context.

## Phase 5: Static Service Inventory And Snapshot V0

Goal:

```text
running kernel facts -> typed snapshot -> static service graph -> machine-readable system model
```

Scope:

- define which code belongs to the permanent core and which belongs to services
- expose `system.snapshot.v0`
- expose service inventory, health state, and last error per service
- model the current statically linked kernel components as services before any
  dynamic service loading
- include service id, kind, health, last error, capabilities, `replaceable`, and
  `core_owned`
- make UI/console/provider/network status consume the same structured model
- add capability names for observation and service lifecycle operations

Definition of done:

- The agent can ask what is running, what is degraded, and which capabilities
  exist without scraping human logs.
- The codebase has an explicit boundary between survival-core responsibilities
  and replaceable service responsibilities.
- Existing framebuffer and console status are derived from typed facts, not from
  a second status source.

Initial service names should be stable even while everything is still linked
into the kernel:

```text
core.boot
core.memory
core.serial
core.scheduler
core.entropy
core.snapshot_root
svc.ui.framebuffer
svc.console
svc.input
drv.usb.xhci
drv.net.e1000
svc.net.ipv4
drv.wifi.avastar_probe
svc.provider.openai_direct
```

The first agent protocol methods are read-only:

```text
system.describe
system.snapshot
system.capabilities
system.boot_log
device.graph
problem.list
service.inventory
```

Mutating methods may be documented, but they must initially return
`capability_denied` until manifest, VM-test-report, local attestation, and audit
records exist.

## Phase 5.5: Read-Only System Memory Context

Goal:

```text
typed facts -> bounded context broker -> agent_context.v0
```

Scope:

- expose `memory.profile`
- expose read-only `memory.context` over current snapshot, service inventory,
  problem list, capabilities, boot log summaries, and ADR metadata
- expose `memory.query` and `memory.trace` for included records
- enforce token profiles such as `provider_minimal`, `diagnostic`, and
  `planning`
- make summaries and semantic/RAG hits locators only, never authority
- keep all memory mutation denied until event log, audit, policy, persistence,
  and rollback records exist

Definition of done:

- The agent can ask for task-relevant context without receiving the whole memory
  store or raw logs.
- Context packets report profile, budget, included records, and omitted classes.
- Provider-bound context still obeys provider trust and redaction gates.

## Phase 5.6: RAM-Only Current-Boot Event Log

Goal:

```text
agent protocol behavior -> bounded event.log.v0 -> denial/event evidence ids
```

Status: implemented for agent protocol reads and known denials.

Scope:

- expose `memory.recent_events [limit]`
- expose `audit.events [limit]` as an alias
- record read-only protocol responses with method, capability, classification,
  outcome, and compact evidence
- record `capability_denied` outcomes for memory/module/service/config methods
- include current-boot `event_id` and `audit_event_id` in denial responses
- keep the log RAM-only, bounded, non-secret, and non-provider-exported

Definition of done:

- Shadow VM proves `event.log.v0` and `audit.event.v0` over serial.
- Denied memory and module methods cite event ids.
- No persistent memory, durable audit ledger, or provider export is implied.

## Phase 5.7: Provider-Minimal Redaction Projection

Goal:

```text
agent_context.v0 -> classified provider_minimal projection -> export still denied
```

Status: implemented as a local read-only projection.

Scope:

- mark `provider_minimal` available as a local projection in `memory.profile`
- include local `context_event_id` and `audit_event_id` handles on
  `memory.context` responses
- emit `raios.provider_context_projection.v0` for
  `memory.context provider_minimal`
- classify provider-bound fields as `public`, `local_only`, or `secret`
- include only public product/stage identity, coarse subsystem states, provider
  state markers, capability ids, service ids, stable problem metadata, and
  public record summaries in the nested projected packet
- omit raw `system.snapshot`, boot logs, local-only details, provider prompt
  text, request ids, network topology, Wi-Fi secrets, TCP diagnostics, and
  unclassified context
- keep provider export disabled with explicit blockers for provider trust and
  provider export audit binding

Definition of done:

- Shadow VM proves the projection schema, field classification, explicit
  omissions, local event ids, provider export denial, and query/trace locator.
- OpenAI requests still do not receive automatic context injection.

## Phase 5.8: Provider Context Export Gate

Goal:

```text
provider_minimal projection -> provider_context_export gate -> provider write denied
```

Status: implemented as a denied-by-default protocol gate.

Scope:

- expose `provider.context_export [provider_minimal]` and
  `provider.export_context [provider_minimal]` as provider-boundary methods
- add `cap.provider.context_export` with risk `export` and no V0 grant
- return `raios.provider_context_export.v0` with current-boot `event_id` and
  `audit_event_id`
- report provider trust state, projection presence, field-classification
  presence, packet evidence state, missing request binding, missing export
  audit binding, and `provider_write: not_attempted`
- record the denial in `event.log.v0` as `cap.provider.context_export`
- keep OpenAI requests free of automatic context attachment

Definition of done:

- Shadow VM proves the export schema, capability denial, export risk event,
  missing evidence list, and no provider write attempt.

## Phase 5.9: Provider Context Packet Evidence

Goal:

```text
provider_minimal packet -> canonical evidence hashes -> export still denied
```

Status: implemented for the local projection and denied export gate.

Scope:

- define `raios.provider_minimal.packet.canonical.v0`
- hash the canonical provider-minimal `raios.agent_context.v0` packet
- hash the exported field list separately
- hash the omitted field list separately
- expose those hashes through `raios.provider_context_projection.v0`
- expose those hashes through `raios.provider_context_export.v0`
- report packet and field-list bindings as present while provider writes remain
  `not_attempted`
- keep OpenAI requests free of automatic context attachment

Definition of done:

- Shadow VM proves the projection and export gate both expose
  `projected_packet_hash`, `exported_field_list_hash`, and
  `omitted_field_list_hash`, while request binding and export audit binding
  remain missing.

## Phase 5.10: Provider Export Denial Audit

Goal:

```text
failed provider export -> distinct denial evidence -> export gates still fail
```

Status: implemented for the denied `provider.context_export` path.

Scope:

- keep positive `raios.provider_request_binding.v0` missing until a real
  provider request envelope exists
- keep positive `raios.provider_context_export_audit_binding.v0` missing until
  structured hash-valued audit evidence exists
- emit `raios.provider_request_binding_denial.v0` for the failed binding
  attempt
- emit `raios.provider_context_export_denial_audit.v0` for the no-write export
  decision
- record separate current-boot event ids for the capability denial, request
  binding denial, and export denial audit
- mark denial-audit records with `satisfies_export_gate: false`
- carry hash-valued structured `event.log.v0` bindings on the denial events
  while keeping `satisfies_current_boot_export_gate: false`
- keep `provider_write: not_attempted` and automatic provider context injection
  disabled

Definition of done:

- Shadow VM proves the positive binding gates remain missing, denial records are
  present but cannot satisfy export gates, and the event log contains
  `provider_context_export.request_binding_denied` plus
  `provider_context_export.denial_audit` with packet/field-list hashes.

## Phase 5.11: Provider Request Envelope

Goal:

```text
real provider request path -> local pre-write envelope -> positive binding candidate
```

Status: implemented for the real direct OpenAI `ask` path.

Scope:

- create `raios.provider_request_envelope.v0` only from the real OpenAI request
  path, not from `provider.context_export`
- bind the envelope to the exact request body hash prepared for HTTPS write
- keep raw prompt text, API keys, Authorization values, and Content-Length out
  of the envelope
- keep provider-minimal context attachment blocked unless positive provider
  trust and a positive export audit binding both exist
- fail closed if envelope hashes, packet hashes, boot scope, or event retention
  do not match

Definition of done:

- Shadow VM proves `provider.context_export` does not create a fake request
  envelope.
- Direct OpenAI pin-mismatch smoke proves the envelope schema appears on a real
  provider request path, omits prompt/Content-Length/Authorization values, and
  still fails before HTTPS write on pin mismatch.
- Denied export remains denied until a positive request binding and positive
  export audit binding exist.

## Phase 5.12: Positive Provider Context Binding

Goal:

```text
provider_minimal packet hash -> real request envelope -> positive export audit binding
```

Status: implemented for local-only current-boot binding records; automatic
context injection remains disabled.

Scope:

- create `raios.provider_request_binding.v0` only for a retained current-boot
  `raios.provider_request_envelope.v0`
- bind request-envelope hash, request-body hash, provider-minimal packet hash,
  exported-field-list hash, and omitted-field-list hash
- reject denial schemas, development TLS bypass, stale or dropped event ids,
  previous-boot ids, consumed bindings, and hash mismatches
- create `raios.provider_context_export_audit_binding.v0` only after positive
  provider trust and matching request binding exist
- set `satisfies_request_binding_gate: true` only on the request binding
- set `positive_export_authorization: true` only on the export audit binding
- keep `satisfies_current_boot_export_gate: false`,
  `automatic_context_injection: disabled`, and
  `context_attached_to_provider_body: false`

Definition of done:

- Shadow VM proves standalone `provider.context_export` still cannot fake
  request envelopes or positive bindings.
- Direct OpenAI pin-mismatch smoke proves positive binding markers remain absent
  when provider trust fails.
- Direct OpenAI SPKI pinned-trust smoke proves the real `ask` path emits the
  request envelope, positive request binding, and positive export audit binding
  markers before HTTPS write.
- The OpenAI request body still does not receive automatic provider-minimal
  context.

## Phase 5.13: Checked Current-Boot Binding Consumption Gate

Goal:

```text
positive binding pair -> checked retained chain -> consumed for local gate evaluation
```

Status: implemented for local gate evaluation and negative predicate selftests;
automatic context injection remains disabled.

Scope:

- expose `provider.context_gate provider_minimal` as a read-only diagnostic
  over retained current-boot binding evidence
- validate one `raios.provider_request_binding.v0` with one matching
  `raios.provider_context_export_audit_binding.v0`
- require matching request id, request-envelope event id, request-body hash,
  request-envelope hash, request-binding hash, and provider-minimal
  packet/exported/omitted field-list hashes plus redaction,
  field-classification, token-budget, provider-trust evidence hashes, and
  provider trust verifier metadata inside the retained binding pair
- reject development TLS bypass records, non-positive trust records, stale or
  dropped referenced events, wrong variants, already consumed pairs, and body
  attachment records
- expose `provider.context_gate_selftest provider_minimal` as local-only test
  infrastructure that exercises stale/dropped ids,
  previous-boot-or-unretained ids, substituted denial schemas, substituted
  positive records, request/body/binding/context hash mismatches, and
  redaction/classification/budget/trust-evidence hash mismatches without
  mutating global event state
- consume a valid pair once through `provider.context_export provider_minimal`
  and record `raios.provider_context_binding_consumption.v0`
- keep `satisfies_current_boot_export_gate: false`,
  `automatic_context_injection: disabled`, `provider_write: not_attempted`, and
  `context_attached_to_provider_body: false`

Definition of done:

- Shadow VM proves the read-only gate reports missing binding evidence without
  creating request envelopes or positive bindings.
- Shadow VM proves the selftest cases reject stale/dropped ids,
  previous-boot-or-unretained ids, substituted schemas, substituted positive
  records, mismatched request/body/binding/context/redaction/classification,
  budget, and trust-evidence hashes, and trust-bypass records while creating no
  provider request envelopes or positive binding records.
- Direct OpenAI pin-mismatch smoke proves positive binding and consumption
  remain absent when trust fails.
- Direct OpenAI SPKI pinned-trust smoke proves marker hashes match, the retained
  pair validates, the first export-gate evaluation consumes it without body
  attachment, and a second attempt is rejected as `binding_already_consumed`.

## Phase 5.14: Final Provider Context Injection Gate

Goal:

```text
checked binding evidence -> explicit injection authorization -> one request body may attach context
```

Status: fail-closed diagnostic and negative authorization selftests implemented;
no context injection is implemented in the current slice.

Scope:

- define a distinct schema for the final injection authorization, separate from
  request binding, export-audit binding, and binding consumption
- expose `provider.context_injection_gate provider_minimal` as a read-only
  diagnostic over the current gate state
- expose `provider.context_injection_gate_selftest provider_minimal` as
  local-only test infrastructure for missing, stale, substituted, body-hash
  mismatched, trust-downgraded, and unauthorized body-attachment final
  authorization candidates
- emit a blocked `OPENAI_PROVIDER_CONTEXT_INJECTION_GATE` marker on positive
  pinned/WebPKI OpenAI request paths before API-key copy or HTTPS write
- require positive provider trust, retained current-boot binding evidence,
  redaction projection hashes, provider-trust evidence hash, single-use
  consumption, and a final local policy decision before
  `context_attached_to_provider_body` may become true
- evaluate the current direct OpenAI gate synchronously before HTTPS write; a
  future provider-adapter service boundary may replace that direct path after it
  has equivalent evidence and tests
- require fail-closed harness coverage for missing final authorization, stale
  final authorization, hash mismatch, trust bypass, and body attachment attempts
  without authorization
- keep raw prompt text, API keys, Authorization values, local-only network
  details, and unclassified memory out of all provider context

Definition of done:

- `context_attached_to_provider_body` becomes true only when the final injection
  gate's own schema and evidence pass.
- Direct and Shadow VM harnesses prove denied and positive paths separately.
- The request body contains only the redacted `provider_minimal` projection and
  never raw local-only or secret fields.

## Phase 6: Ephemeral Live Services

Status: started with a denied-by-default `raios.module_load_gate.v0`, a
host-side `raios.computed_capability_grant.v0` diagnostic, and a guest-side
read-only computed-grant hash-reference diagnostic. No artifact loader,
ram-only service slot allocator, durable audit ledger, rollback state, or
positive loading grant exists yet.

Goal:

```text
AI proposes artifact -> capability check -> load for current boot -> drop/kill
```

Scope:

- module/service manifest v0
- ram-only service slot
- service registry
- capability grants are computed by local policy, not self-declared by modules
- health checks and crash records
- audit log for load, start, kill, and unload
- denied-by-default behavior for missing manifest, missing grant, missing test
  report, or missing local attestation

Definition of done:

- A low-risk service can be loaded without reboot, expose one new console command
  or UI panel, then be removed without corrupting the rest of the system.
- Loading requires service inventory, manifest, computed capability grants,
  health reporting, audit records, and an explicit denial path.

## Phase 7: Hot-Swap And State Migration

Goal:

```text
service v1 keeps running -> service v2 loads -> state migrates -> handles switch
```

Scope:

- versioned service state objects
- first state migrator
- handle indirection for service clients
- atomic switch and rollback
- watchdog during the probation period after a switch

Definition of done:

- A simple service can be upgraded live while preserving its state.
- A failed upgrade rolls back to the previous service version without a full
  system restart.

## Phase 8: Recovery Agent Lifeline

Goal:

```text
live world down -> core still reports state -> AI can trigger recovery actions
```

Scope:

- tiny recovery control protocol
- separate from the normal rich provider service
- separate from the direct OpenAI chat path
- restart last-good service set
- disable bad module ids
- load recovery artifact by hash
- optional pinned minimal provider route or local physical link

Definition of done:

- If UI, provider service, or another non-core service crashes, the core can
  still expose a snapshot and accept bounded recovery commands.
- The current `svc.provider.openai_direct` path is not treated as the recovery
  lifeline unless a separate minimal recovery protocol and trust state exist.

## Phase 9: Shadow VM Acceptance

Goal:

```text
candidate artifact -> shadow boot/test -> report hash -> live/persist decision
```

Scope:

- machine-readable VM test report
- image hash, artifact hash, hardware profile, and snapshot precondition binding
- serial/protocol/screenshot predicates
- acceptance policy by risk level
- first implementation may extend the existing serial smoke test before adding
  QMP, power fault injection, or screenshot diffs

Definition of done:

- Risky service changes and all persistent changes require a matching test
  report before activation.
- The first report includes image hash, QEMU args hash, hardware profile,
  commands, predicates, result, and serial log reference.

## Phase 10: Persistence, Rollback, And Core Handoff

Goal:

```text
tested service set -> persist -> boot-success mark -> rollback or core generation handoff
```

Scope:

- image/state layout specification before implementation
- persistent service set
- last-good pointer
- safe mode that disables non-core modules and persistent writes
- boot-success marker
- rollback on crash or missing success mark
- experimental core-generation handoff for deep core updates

Definition of done:

- raiOS can persist a tested live change, recover from a bad persistent change,
  and eventually replace even core generations without a normal user-visible
  reinstall cycle.
- The current single-FAT Stage-0 image remains explicitly documented as the MVP
  layout until an A/B or DATA-backed layout is specified and tested.
