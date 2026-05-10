# Invariant Choices Sheet

This sheet captures any clarifications or default assumptions made while locking the hard invariants for the seed OS build.

- **VM platform**: QEMU x86_64 with OVMF/UEFI firmware, Intel e1000 networking, USB-xHCI keyboard/mouse, and RDRAND entropy. Timer source pinned to TCG for CI; KVM allowed locally for speed.
- **Framebuffer**: GOP-provided BGRA8888 surface; a dedicated backbuffer will be allocated at boot and presented atomically. All blit sources come from immutable atlases.
- **Input batching**: Event batching window defaults to 12 ms (midpoint of 8–16 ms range) with jitter-bound timers ensuring compliance.
- **Networking**: DHCPv4 for address, DNS from DHCP, TLS sessions pinned via SHA-256 SPKI hash of the online signer cert, WebSocket overlay for all control traffic.
- **Runtime**: Single Wasm module loaded at a time; outbound network allowed under host API `ws_open`/`ws_send` and generic TCP/UDP sockets when granted.
- **OTA**: Slots `A` and `B` on ESP; OTA manifest signed by online key whose cert is signed by the offline root. Each chunk hashed with BLAKE3; whole-image verification before marking pending. `kexec` used for handoff; rollback triggered if Stage-1 fails to set success flag in DATA partition.
- **Safety controls**: Panic switch implemented as boot flag in DATA (`/data/SAFE`) that prevents module launch and OTA writes; remote lockdown closes module and outbound sockets while maintaining WS control channel.
- **Logging**: Serial console treated as authoritative log sink; all runtime logs mirrored to WebSocket channel for cloud ingest.

No deviations from the mandated invariants are introduced at this stage.
