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
