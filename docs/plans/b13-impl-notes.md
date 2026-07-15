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
