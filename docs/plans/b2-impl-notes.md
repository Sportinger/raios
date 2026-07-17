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
