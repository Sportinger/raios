# 0028 — NET-8 fixtures use process-local keys and supervised startup

Date: 2026-07-20 · Status: active

## Context

Three focused rollback runs stopped before QEMU while starting the local NET-8
TLS fixture. The first exposed a Windows `PATH`/`Path` collision in child
process creation. The next two waited the full ready-file deadline after the
child was accepted, but the parent discarded the wrapper's PID and log paths,
so their reports could not distinguish a certificate/key-store failure, port
collision, argument failure, build failure, or another early child exit.

The fixture currently generates an ECDSA key and self-signed certificate, then
exports an unpassworded PFX and reimports it with `UserKeySet` before starting
the listener. That unnecessary transition makes a process-local test key depend
on the current user's Windows private-key storage and profile permissions.

Two fresh independent read-only Codex opinions agreed that the PFX/user-store
transition must be removed and that the certificate returned by
`CreateSelfSigned` can remain alive for `SslStream`. They disagreed on the
minimum coherent repair. The cryptography opinion accepted a one-file C# fix
and treated supervision as separate hardening. The portability opinion required
the wrapper and both parent harnesses to retain PID/stderr evidence, fail fast,
clean up children, and avoid fixed-port collisions before another QEMU run.

## Decision

1. The NET-8 fixture uses the certificate returned by `CreateSelfSigned`
   directly for every server handshake. It does not export or reimport PFX,
   request `UserKeySet`, persist a key container, or install a certificate in a
   user or machine store. The generated key and certificate remain alive for
   the fixture process lifetime.
2. The published SPKI pin is derived from the public key of the exact
   certificate passed to `SslStream`, not from a separately trusted key object.
   TLS 1.3, `TLS_AES_128_GCM_SHA256`, SAN/SNI `w7.test.raios`, the exact CAS
   request, and artifact length/hash remain unchanged.
3. Fixture startup is supervised. The PowerShell wrapper builds synchronously,
   starts the built DLL directly with Windows-safe arguments and a
   case-normalized environment, and returns a stable start record containing
   the actual fixture PID and stdout/stderr paths.
4. Both parent harnesses retain that start record while waiting for readiness.
   An early child exit fails immediately with machine-readable PID, exit code,
   stderr path/hash/tail, and `ready_exists=false`. Every timeout or exception
   terminates and reaps the known child even when no ready file exists.
5. Each fixture binds loopback port zero and atomically publishes its selected
   host port. The harness maps the guest's unchanged `10.0.2.100:8443`
   endpoint to that published host port. Parallel or abandoned fixture runs
   therefore cannot silently collide on host port 8443.
6. Before another focused rollback attempt, standalone predicates must prove:
   successful ready/start/stop under the supported Windows shells, exact served
   certificate SPKI and TLS/application bindings, an early-exit negative with
   retained stderr evidence and no surviving child, and two simultaneous starts
   with distinct host ports. The full QEMU predicate remains the closure gate.

## Alternatives & second opinions

- Change only `UserKeySet` to `EphemeralKeySet`: rejected. It retains an
  unnecessary unpassworded PFX copy and does not address the observed
  environment failure or the lost child-failure provenance.
- Use the direct certificate but leave wrapper/harness supervision for later:
  rejected for the next QEMU attempt. One opinion considered that the smallest
  cryptographic repair, but after three pre-QEMU failures another opaque
  60-second timeout is not acceptable evidence.
- Keep fixed host port 8443: rejected because concurrent or orphaned fixtures
  can fail before readiness without identifying the collision. The guest-visible
  endpoint remains fixed, so dynamic host binding does not change guest policy.
- Add a detailed C# startup-phase result schema now: deferred. Parent process
  supervision, stderr capture, and stable start records provide the required
  fail-fast boundary; finer phase telemetry is optional follow-up work.
- Store a reusable certificate/private key: rejected because it introduces
  durable secret material and weakens per-run SPKI evidence.

## Consequences

The repair spans the fixture program, its wrapper, and the two consumers rather
than one cryptographic line. Startup becomes slightly more complex, but every
child is attributable and recoverable, parallel runs stop sharing a host port,
and the fixture no longer depends on writable user-key storage. No TLS, pin,
request, artifact, or guest-visible network authority is relaxed.
