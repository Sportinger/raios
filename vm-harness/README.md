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
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\create-local-attestation.ps1 -ManifestPath .\candidate.json -ArtifactPath .\candidate.bin -VmReportPath .\release\vm-reports\shadow-....json -Approval "APPROVE RAM_ONLY <artifact-hash-prefix>"
```

This starts QEMU headless with TCP serial, sends one `ask <text>` command, and
verifies that Stage-0 resolves `api.openai.com`, opens TCP 443 directly from the
guest, completes TLS/HTTPS, and prints an OpenAI Responses API answer. It uses
the local OpenAI-default image at `release\seedos-stage0-local-openai.img`.
