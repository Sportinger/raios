# 0033 — Physical Genesis grants only ephemeral WiFi without persistence

Date: 2026-07-20 · Status: active

## Context

The current BOOTCTL reader observes AHCI only. On the Surface, the system disk
is NVMe and a diagnostic USB stick is not an automatically authoritative
BOOTCTL source, so Genesis can correctly enter
`BootPosture::PersistenceUnavailable`. Both generic Marvell association entry
points currently deny that posture.

The existing missing-Vault fallback cannot be opened for this case. It accepts
the passphrase through the shared Console path, which is reachable from serial,
stores it in the legacy global WiFi state, and can retain it for the boot. That
would turn a physical recovery action into a serial credential and reconnect
bypass.

BOOTCTL absence also means there is no authoritative payload generation or
core-policy binding. Merely detecting USB media cannot manufacture that
authority. At the same time, a person at Genesis can safely authorize one
bounded, RAM-only association for current-boot network diagnosis if that act
cannot escape into persistence, SAFE recovery, provider state, or grants.

## Decision

Genesis may offer a separate `EphemeralPhysical` WiFi path only while the exact
posture is `PersistenceUnavailable`. It accepts only a fresh `LiveRadio` scan
target that is visible, association-ready, and WPA2-PSK/CCMP. Open, hidden,
WEP, WPA, WPA3, synthetic, and incomplete targets remain denied in this slice.

The passphrase enters only through the physically routed `SecureOverlay`.
Serial, Console credential buffers, legacy WiFi password state, Remember
controls, Vault load/save, and durable audit are not joins in this path. The UI
shows only empty/non-empty masked state for this mode so screenshots do not
disclose passphrase length.

Submission creates one non-cloneable, non-copyable, non-formattable authority.
It binds an in-boot attempt ID, the `PersistenceUnavailable` current-boot
scope, and a domain-separated SHA-256 target binding over SSID, BSSID, channel,
the WPA2-PSK/CCMP discriminator, and the complete RSN information element. Its
fields and constructor remain private to the facade. It contains the opaque
`SecretPlaintext` and has no byte accessor or serialization route.

The Marvell connection job uses mutually exclusive `Ordinary`, `SafeVault`,
and `EphemeralPhysical` secret sources. The ephemeral authority is consumed
exactly once into the Supplicant PMK command and becomes a linear non-secret
receipt. There is no Vault, SAFE, or legacy fallback after any ephemeral
failure. Posture and live target are checked at submission, driver entry,
before PMK construction, before waiting for port release, and immediately
before network attach. A busy or ready job rejects the new attempt rather than
accepting it as already satisfied.

Failure, timeout, target change, posture change, and link loss destroy any
remaining host secret, clear secret-bearing DMA at the PMK boundary, disable
bus mastering, clear data-link readiness, and detach the RAM-only network.
There is no host-side retry or reconnect. A new attempt requires a new scan
selection and new physical passphrase entry. `net::attach_wifi` supplies only
current-boot link/DHCP state; this decision creates no provider request,
transport lease, capability, install authority, or durable grant.

## Rejected alternatives and opinions

Using USB presence as a BOOTCTL fallback is rejected. Only a validated and
selected BOOTCTL record can authorize boot generation, core policy, durable
secret use, rollback, or SAFE recovery. The ephemeral token is deliberately
incapable of serving any of those roles.

Relaxing the `PersistenceUnavailable` arm in generic `start_association()` is
rejected because it exposes the serial-capable legacy path. Reusing
`ExplicitSafeWifiReconnect` is rejected because SAFE authority depends on a
stored target and durable pre-use audit. Allowing an open network is rejected
because this slice specifically requires fresh physical secret submission as
its authority-producing act.

Two fresh, independent read-only Codex reviews agreed on the separate linear
current-boot path, exact BSS binding, repeated posture/target checks, and strict
separation from USB, SAFE, Vault, legacy, provider, and grant authority. Claude
was unavailable and, independently, forbidden by the owner's Codex-only
instruction; no Claude opinion or tooling was used.

## Consequences

The Surface can use WiFi for current-boot diagnosis even when persistent boot
authority is unavailable, without claiming that persistence or recovery is
healthy. The broader isolated-Marvell-domain checkbox is not closed by this
slice.

The 88W8897 firmware may retain supplicant state internally after host PMK DMA
has been cleared. Host code never triggers reconnect and link loss quarantines
the device, but the physical deauth/AP-cycle negative remains hardware-pending:
the device must not rejoin without a new physical submission. If it does,
future work must add a documented firmware PMK/profile clear or reset sequence;
this ADR does not invent such a command.
