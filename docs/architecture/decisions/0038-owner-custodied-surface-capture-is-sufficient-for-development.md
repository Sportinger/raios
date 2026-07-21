# 0038 — Owner-custodied Surface capture is sufficient for development

Date: 2026-07-21 · Status: active

## Kontext

ADR 0027 blocks Surface-dependent lanes until the actual reference machine
provides structured CPUID, boot-memory, and device facts. Two fresh independent
read-only Codex reviews agreed that Windows-only capture is insufficient and
that the facts must be observed in one raiOS/Limine boot, then returned through
the hash-chained USB RECLOG. They disagreed about the required attestation bar.

The Owner selected the pragmatic development path: physical custody of a known
reviewed image and stick is sufficient for this unblocker. The goal is honest
machine-specific development evidence for H26, not remote or production-grade
hardware attestation.

## Entscheidung

1. One bounded in-kernel capture records CPUID, SMBIOS memory facts, the Limine
   boot-memory map, and the complete PCI inventory in the same boot epoch.
2. The capture is emitted as canonical, bounded, numbered RECLOG records with a
   shared capture ID and completeness/digest metadata. Existing RECLOG frame,
   sequence, hash-chain, payload-hash, readback, and zero-tail checks remain
   mandatory.
3. The Owner writes the reviewed capture build, cold-boots the actual Surface,
   keeps custody of the stick, and returns it for read-only extraction. This
   custody ceremony is the machine-identity binding for this development gate.
4. The extractor must fail closed on missing, duplicate, mixed, malformed, or
   truncated capture parts and may emit only a candidate manifest. The
   orchestrator separately verifies the raw capture, checker predicates, exact
   Surface identity, and manifest diff before acceptance.
5. Challenge/nonce, enrolled hardware fingerprint, trusted capture-build
   registry, TPM quote, and remote freshness proof are not required for this
   development capture. Therefore the result MUST NOT be described as remote,
   cryptographic, TPM-backed, or production-grade machine attestation.
6. `surface-pro-4.v1.json` remains `curated_context_ready:false` and H26 remains
   blocked until the real Owner-custodied capture covers every required fact,
   the manifest checker is green, and the accepted manifest digest is pinned.
7. This decision does not move the Marvell driver out of the kernel. H26 remains
   an in-kernel driver correction under the current ADR 0005/SCOPE direction;
   the separate kernel-versus-driver-domain scope conflict is unchanged.

## Alternativen & Zweitmeinungen

Both reviews rejected public Surface SKU data, QEMU facts, and a Windows-only
inventory as proof of the actual boot machine. Both selected raiOS CPUID,
SMBIOS, Limine-map, PCI, and RECLOG as the measurement path.

One review considered Owner custody plus the existing hash-chained/readback
RECLOG the smallest sufficient development solution and warned that a separate
UEFI tool would duplicate trusted boot and storage paths.

The other review required a new capture envelope with challenge freshness,
trusted build digest, enrolled machine fingerprint, replay consumption, and
possibly TPM quote. That is stronger against wrong-machine substitution,
replay, and a malicious capture producer, but materially expands the current
H26 unblocker. The Owner chose the custody model now and accepts its explicit
non-attestation limit. Stronger capture attestation remains a separate future
security slice rather than a hidden prerequisite.

## Folgen

The next implementation lane can reuse the kernel's Limine memory request, PCI
enumeration, USB RECLOG writer, and host extractor instead of creating a second
boot tool. One additional physical capture boot is required before H26.

This is faster and auditable under direct Owner custody, but it cannot prove to
a remote verifier that the capture came from an uncompromised binary or resist
an Owner-side substitution ceremony failure. Any future production or remote
hardware-provenance claim must supersede this ADR with the stronger attestation
contract and its replay/wrong-machine negatives.
