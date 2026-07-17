# B2 implementation notes

## B2.1a foundation

- Capability: a fixed key-free answer can become one inert, inspectable, content-addressed Rust/TOML source revision in the disposable QEMU workspace.
- The loop recomputes the request hash and project identity from the exact owner request; identity is `SHA-256("raios.agent_project_id.v1" || u64_le(request_len) || exact_request_bytes)`, truncated to 16 bytes by the core helper.
- `build_agent_answer` uses the existing revision builder and forces `local_only`; its shared path mapper derives only `text/rust` and `text/toml` and rejects non-UTF-8 content or other path types.
- The parser has one bounded-response ceiling only. The core builder remains authoritative for file/project quotas, path, case-alias, duplicate, sorting, blob, tree, parent, revision, and manifest checks.
- `commit_agent_answer` uses the existing inspect -> build -> blobs-first/manifest-last commit -> reload/compare store path and never enters the serial import parser.
- The fixture has a fixed request, request id, and two-file answer. It arms only an idle current-boot loop; replay is therefore untracked and cannot create a child revision accidentally.
- Fixture availability is enforced by the only commit backend: `open_disposable_qemu_store_port`; outside that configuration the commit returns `project_qemu_store_missing` and no answer is accepted.
- Fixture provenance binds the exact answer byte length/hash and is always `answer_origin=test_fixture`, `test_infrastructure=true`, and `provider_trust_positive=false`.
- `project.workspace` describes current-boot job state; `project.inspect` and `project.read` remain the durable file readers. `program.workspace` is unchanged.
- `writes_persistent_state=true` on fixture success means source records in the disposable QEMU structured store only; every executable/build/load/run/install/promotion field remains false.
- `project_overlay.rs` gained the compile-required exhaustive `AgentAnswer` rebuild arm so later W2 edits can verify this base through the same core builder.
- `note_provider_error` and the private shared `accept_answer` seam are present, but no live provider target, provenance, handler, console route, Genesis UI, build, runtime, approval, install, RECLOG, ARTSTOR executable, or service-inventory path is wired.
- No Cargo, rustc, build, or test command was run; the orchestrator owns compilation and VM evidence for this worker packet.

## B2.1a harness

- Capability: the focused `project-workspace` profile can now prove the fixed key-free answer becomes one inert, inspectable, content-addressed two-file source revision and survives a structured-store reboot.
- The seven B2.1a predicates are appended after every existing W1-W3 predicate; no existing predicate or shared helper changed.
- Host expectations come from the merged fixture bytes and reuse the profile's existing blob/tree/revision SHA-256 helpers with action `agent_answer`; `project.workspace`, `project.inspect`, and full-file `project.read` results are pinned as `raios.agent.v0` `body.result` carve-outs.
- The fixed-only route is replayed after success. Its exact `agent_answer_request_not_tracked` denial must preserve the parentless revision and cannot form a child; malformed/base64/path/case/quota injection is not exposed by this route and remains covered by core tests until a caller accepts variable answers.
- Service inventory plus guest RECLOG and ARTSTOR scans are captured before and after the accepted fixture, while every emitted build/load/run/install/promotion field is pinned false and the source-store posture remains `qemu_disposable_structured_store_only`; replay separately pins no storage write.
- A fourth boot proves `project.inspect` and both full-file reads reparse byte-identically; `project.workspace` correctly resets because job state is `current_boot_ram_only`.
- The profile is PowerShell 5.1 parser-checked only in this worker packet. No Cargo, build, or VM command was run; focused VM proof remains the orchestrator's responsibility.

## B2.2a foundation

- Capability: a key-free fixture can now carry one system-computed source-preflight failure into an exact immutable child revision and re-run the same preflight over that child; it still cannot compile, test, run, install, or export anything.
- `verify_source_revision` stays in `seed-kernel/src/project_build.rs` because `snapshot_exact` and the exact disposable-store/dependency loads live there. The two-outcome source-shape predicate is the public cross-crate `raios_core::project_build::source_preflight` helper so the missing-lock and manifest-plus-lock cases have host tests without opening a kernel build session.
- The verifier is a deterministic local source preflight, not a compiler or test runner. It records `project.source_preflight.v1`, the exact revision/tree hashes, the system-selected reason, and `passed|failed`; provider text supplies none of those fields.
- The existing B2.1a fixture is revision 1 and intentionally lacks `Cargo.lock`, so its exact preflight reason is `build_cargo_lock_missing`. The child fixture is a complete three-file answer with the same `Cargo.toml` and `src/main.rs` plus a fixed `Cargo.lock`, so its exact preflight reason is `source_preflight_ok`; the existing source mapper now admits only the exact root `Cargo.lock` addition and classifies it as `text/toml`.
- `build_feedback_packet` stores only four cited values: check id, revision SHA-256, tree SHA-256, and reason. The protocol classifies the packet `local_only` and reports source/secret/log/unclassified bytes absent; no provider submission path consumes it.
- `accept_revision_answer` accepts only the fixed fixture, requires the tracked failed verifier result and matching feedback packet for the exact parent, arms a fixture-only child request, and delegates parsing/building/commit/readback to the existing private `accept_answer` seam.
- `commit_agent_answer` now requires the caller's exact expected parent instead of discovering and silently adopting whatever revision is current. A mismatch denies before commit; revision 2 therefore has parent exactly revision 1.
- `project_store::load_revision` reads an immutable manifest by exact project/revision identity. Before a child commit, the child route reparses and hash-validates revision 1's source blobs; failure consumes the request without a write. A successful child therefore carries a guaranteed parent-readback result plus both lineage entries.
- New `raios.agent.v0` body-result routes are `project.verify_revision`, `project.feedback_packet`, and `project.revision_answer_fixture`; `project.workspace` also exposes the current verifier result, retained feedback packet, and revision lineage. All build-session/compiler/test/run/load/install/W6/executable-record/service-mutation fields remain false.
- B2.2 remains partial: scoped feedback export, live provider submission, W4 compilation, B3 on-device compilation, W5 execution, W6 installation, and Genesis UI are deliberately not wired.
- No Cargo, rustc, build, test, or VM command was run; the orchestrator owns compilation and focused VM evidence for this worker packet.

## B2.2a harness

- Capability: the focused `project-workspace` profile can prove revision 1's system-owned missing-lock result drives one exact inert child revision whose local preflight passes while both revisions remain bound and readable.
- Seven stable B2.2a predicates pin the merged-kernel `raios.agent.v0` `body.result` fields for verification, the four-field local-only feedback packet, the exact three-file child, re-verification, retained revision-1 bytes/lineage, zero executable effect, and replay denial.
- The host reuses the profile's canonical blob/tree/revision SHA-256 helpers over the exact merged fixture bytes; the child is `agent_answer`, parents revision 1 exactly, and adds only root `Cargo.lock` as `text/toml`.
- B2.2a uses a second empty disposable QEMU store/boot and replays the same fixed revision-1 setup there, so all committed B2.1a predicate bytes and its revision-1 reboot proof remain untouched; the seven B2.2a predicate records are appended after that complete existing set.
- The fixed-only replay is denied as `agent_revision_verifier_result_mismatch`; unchanged `project.workspace`, `project.inspect`, and two-entry lineage prove that the route cannot form a third revision.
- Fresh service-inventory, RECLOG, and ARTSTOR scans bracket the verify/feedback/child/reverify loop. Every emitted build-session, compiler, test, run, load, execution, install, promotion, Wasm, W6, executable-record, and service-mutation field stays inert, and provider export remains unattempted.
- Failures dump the full relevant response set at JSON depth 16. The profile remains PowerShell 5.1 parser-clean, uses the existing marker reader, and adds no wall-clock assertion or stream merge.
- Concurrent smokes previously shared port 4565 and `-StopExisting`, so overlapping sessions replaced each other's QEMU and produced false store-stall signatures. The shared shadow runner now rejects a concurrent same-port run through a named per-port mutex before packaging or QEMU startup.
- Orchestrator verification: `project-workspace` `shadow-20260717-142445-27836.json`, 654/654 predicates, including all seven B2.2a predicates; report SHA-256 `da93b159bdf356faef2bdb80e333538dce54c9bba9dc5b60e391cfa3751c12ea`.

## B2.1b live provider

- Capability: a real OpenAI response can now enter the existing B2.1a answer-to-files seam as one inert source revision when, and only when, it belongs to the exact pending project request and the observed live verifier is positive pinned trust without the development bypass.
- `AgentRequest::project` and `RequestTarget::ProjectWorkspace` reuse the existing OpenAI transport. Conversation and RUIP targets keep their existing request bodies and handling; `/build` is the explicit source-project route, `/program` is the explicit RUIP fast lane, and raw `program.ask` remains compatible.
- `AnswerProvenance` carries the actual request-body and request-envelope SHA-256 values plus the post-response verifier id/state/outcome, chain policy, time policy, and bypass flag. It is constructed only after a successful TLS/HTTP response and does not transform or re-encode the extracted answer text.
- Live source ingestion accepts only `pinned_cert_verified` or `pinned_spki_verified` with verifier outcome `verified`, the exact pinned-verifier id/policies, bound nonzero request hashes, and `development_tls_bypass=false`. The verifier remains honestly `pin_only_no_webpki_chain_validation` with time `not_validated_stage0`; no WebPKI or trusted-time claim was added.
- A matching live answer delegates to the same private `accept_answer` parser/build/commit/readback seam as B2.1a with `answer_origin=live`, `provider_trust_positive=true`, and `test_infrastructure=false`. A matching bypassed or non-positive answer is consumed and rejected before parsing/storage; mismatched, late, duplicate, and untracked answers cannot replace the last accepted revision.
- The project authoring prompt requests exactly one `RAIOS_SOURCE_FILES_V1` response and gives the provider no classification, media-type, hash, identity, capability, trust, or pass/fail authority. The core still forces `LocalOnly`, derives media type from the validated path, recomputes identity, and commits only to the disposable QEMU structured store.
- Console outcome text reports request id, project id, revision hash, and file count without printing source bytes. Provider errors close only the matching pending project request.
- Automatic provider context injection remains disabled and no scoped feedback export, Genesis source-status drawing, compiler, test, run, load, install, promotion, W6, physical approval, executable RECLOG/ARTSTOR record, or service mutation was added.
- Implementation status: implemented; live path not demonstrated in this worker packet. The orchestrator still owns compilation and the pinned live `openai-direct-smoke` closure. No Cargo, rustc, build, test, or VM command was run.

## B2.1b live harness

- `openai-direct-smoke.ps1 -ExpectProjectWorkspaceAnswer` sends the headless source-lane command `project.ask <plain description>` after the existing provider/network readiness markers; it never uses `ask /build`.
- It requires `PROJECT SOURCE REQUEST <id> STARTED`, the existing positive pinned-cert-or-SPKI transport/binding/injection markers with no TLS bypass or envelope leak, then accepts exactly one honest terminal marker: `PROJECT SOURCE READY request=<id> project=<hex> revision=sha256:<64hex> files=<n> inert` or `PROJECT SOURCE REJECTED request=<id> project=<hex|none> revision=<sha256:...|none> files=0 reason=<reason>`.
- Both outcomes are cross-checked against the `RAIOS_AGENT_BEGIN project.workspace` / `RAIOS_AGENT_END project.workspace` `raios.agent.v0` result and must remain `local_only`, `untrusted_agent_candidate`, context-injection-disabled, and free of build/load/run/install/promotion, candidate-intake, Wasm/W6, service-start, or executable RECLOG/ARTSTOR effects.
- Serial reachability: the source lane had no headless entry point (`submit_project_prompt` was only reachable from the on-screen Genesis composer `/build`). Added the serial console command `project.ask <description>` (mirrors `program.ask`) so the ProjectWorkspace request is drivable over serial.
- Store substrate: ProjectWorkspace commits require the disposable QEMU-only C1 structured store (an AHCI disk at 00:1f.2, `bus=ide.4` on q35) plus a valid-a BOOTCTL SEED_DATA persist disk for Normal boot posture. The network-only openai-direct image booted `PersistenceUnavailable` and rejected with `project_qemu_store_missing`; the `-ExpectProjectWorkspaceAnswer` branch now builds both fixtures (`make-structured-store-image.py create --size-mib 16`, `make-gpt-persist-image.py --self-check --seed-bootctl valid-a`) and passes `-StructuredStoreDiskPath` + `-PersistDiskPath` to the runner. The context-export gate re-query (`Invoke-PositiveBindingGateChecks`) was moved to after the outcome resolves so its fresh `agent provider.context_gate` never races the in-flight provider request.

### B2.1b LIVE PROOF DEMONSTRATED (orchestrator run 2026-07-17)

- Pinned trust: computed api.openai.com's live P256 leaf-cert DER SHA-256 (`76e49dae8cbc1f012fc9fb6e822060f8ed3a56dd3ea33ba3fb2b78cbe8cce64c`) via a .NET TLS 1.3 probe; packaged the image with `-EmbedOpenAiApiKeyFromEnv -EmbedOpenAiCertPinFromEnv` (release, core-policy signed). The live handshake verified `pinned_cert`, `outcome=verified`, `reason=leaf_cert_pin_and_certificate_verify_valid`, `development_tls_bypass=false`.
- Outcome: `PROJECT SOURCE READY request=1 project=05428c0d1ba1945ad55f5220a10a05b5 revision=sha256:17a8b9a87226464381e588793bfce35ee05ee7a3357d67d73b6956acd4f67f2e files=2 inert`. A real OpenAI answer became two inert Rust source files (Cargo.toml + src/main.rs), committed `answer_origin=live`, `provider_trust_positive=true`, `test_infrastructure=false`, `local_only`, `untrusted_agent_candidate`, zero executable effect.
- All six predicates passed, harness exit 0: `b2-live-provider-ready`, `b2-build-request-sent`, `b2-live-answer-positive-pinned-provenance`, `b2-answer-outcome` (CONFORMING-committed-inert), `b2-workspace-provenance-exact`, `b2-inert-zero-executable-effect`. No prompt/Authorization leak; automatic context injection stayed disabled.
- Reproduce: `powershell -File vm-harness/openai-direct-smoke.ps1 -ExpectProjectWorkspaceAnswer -TimeoutSeconds 120` after packaging `release/raios-stage0-local-openai.img` with a fresh live cert pin (leaf certs rotate; recompute if the handshake reports `PIN MISMATCH`).
