# VM Harness

QEMU/QMP orchestration, fixtures, golden transcripts, and framebuffer hashing utilities for deterministic end-to-end testing.

Current smoke entry points:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1
```

`shadow-vm-smoke.ps1` is the first Shadow-VM sandbox gate. It packages a
temporary Stage-0 image unless `-Image` is provided, boots QEMU headless with no
shared folders and networking disabled by default, sends read-only agent
protocol commands over TCP serial, verifies mutating module load remains
`capability_denied`, kills QEMU, and writes a `seedos.vm_test_report.v0` JSON
report under `release\vm-reports`.

The current Shadow-VM profile is fixed and recorded in each report as
`hardware_profile`: Q35, 512M RAM, EDK2 firmware, raw IDE boot image, headless
display, TCP serial logging, qemu-xhci with USB keyboard/tablet, and either no
network by default or QEMU user-mode e1000 when `-Network` is passed. The report
binds that profile by hash alongside the image hash, candidate hashes, QEMU args
hash, predicate counts, and serial log hash.

Optional candidate evidence can be attached to the report without mounting or
executing it in the guest yet. The manifest is validated first, and the artifact
hash in the manifest must match the provided bytes:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\shadow-vm-smoke.ps1 -ManifestPath .\candidate.json -ArtifactPath .\candidate.bin
```

Validate just a manifest/artifact pair:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\validate-module-manifest.ps1 -ManifestPath .\candidate.json -ArtifactPath .\candidate.bin
```

Record local attestation after a passing Shadow-VM report:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\create-local-attestation.ps1 -ManifestPath .\candidate.json -ArtifactPath .\candidate.bin -VmReportPath .\release\vm-reports\shadow-....json -Approval "APPROVE RAM_ONLY <approval-tuple-prefix>"
```

The expected approval phrase is printed by the tool on mismatch. It is derived
from the tuple of manifest hash, artifact hash, VM report hash, base image hash,
and load mode, not from the artifact hash alone.

`openai-direct-smoke.ps1` starts QEMU headless with TCP serial against the local
OpenAI-default image at `release\seedos-stage0-local-openai.img`. By default it
sends one `ask <text>` command and verifies the current trust gate denies the
request with `pin_config_missing` before any HTTPS write. With
`-ExpectPinnedTrust`, it expects an image built with `-EmbedOpenAiCertPinFromEnv`
and verifies the normal OpenAI leaf-certificate pin path through TLS 1.3,
positive `pinned_cert` trust, HTTPS write, and provider HTTP response/error.
With `-ExpectPinMismatch`, it expects an intentionally wrong configured pin and
verifies that the request fails during the pinned verifier path before HTTPS
write.
With `-ExpectProviderResponse`, it expects a development image built with
`-AllowUnverifiedOpenAiTls` and verifies the old unverified DNS/TCP/TLS/HTTPS
provider-response path.
