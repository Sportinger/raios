# VM Harness

QEMU/QMP orchestration, fixtures, golden transcripts, and framebuffer hashing utilities for deterministic end-to-end testing.

Current smoke entry points:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File vm-harness\host-bridge-smoke.ps1
```

This starts QEMU headless with TCP serial, runs `scripts\host-bridge.ps1`, sends
one `ask <text>` command, verifies the bridge response in the serial log, and
stops QEMU.
