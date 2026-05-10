# VM Harness

QEMU/QMP orchestration, fixtures, golden transcripts, and framebuffer hashing utilities for deterministic end-to-end testing.

Current smoke entry points:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\openai-direct-smoke.ps1
```

This starts QEMU headless with TCP serial, sends one `ask <text>` command, and
verifies that Stage-0 resolves `api.openai.com`, opens TCP 443 directly from the
guest, completes TLS/HTTPS, and prints an OpenAI Responses API answer. It uses
the local OpenAI-default image at `release\seedos-stage0-local-openai.img`.
