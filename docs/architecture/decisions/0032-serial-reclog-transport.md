# 0032 — Serial RECLOG V0 is a fail-closed line-armored transport

Date: 2026-07-20 · Status: active

## Context

The kernel currently emits boot, debug, panic, and agent-response bytes through
COM1, while host evidence tooling is oriented around QEMU serial logs, TCP, and
PowerShell text processing. Marker-delimited agent JSON covers only selected
responses and permits malformed prefixes, suffixes, interleaving, or duplicated
markers to escape whole-stream validation. Durable `RAIOSRC0`, in contrast, is
a sector-oriented persistent record format and does not describe a live UART
transcript.

The literal scope boundary is all kernel-owned serial/debug output, not only
selected evidence. A transport decision is therefore required before producer
migration or a host report gate can be implemented.

## Decision

V0 uses the transport-specific ASCII line-armored protocol specified by
`docs/architecture/device-protocol/serial-reclog-v0.md`. It has its own magic,
version, session identity, sequence, bounded payload, integrity chain, and
stream failure semantics. It does not reuse the magic, sector layout, sequence,
recovery rules, or persistence meaning of durable `RAIOSRC0`.

The first kernel-owned COM1 record is one unique session-start frame. From its
first byte until capture termination, every kernel-originated COM1 byte belongs
to exactly one complete, canonical frame. A host may accept firmware or
pre-kernel bytes only through the protocol's explicit bounded preamble rule;
it may not search past an invalid byte for another magic. Boot messages,
ordinary debug output, panic text, and agent data become typed payloads rather
than unframed evidence.

The transport owner assigns a checked, strictly increasing sequence and emits
each complete physical line atomically. Early and panic paths are bounded and
allocation-free. Lock contention or reentrancy must either produce one valid
bounded emergency frame under the specified single-writer conditions or leave
the stream observably invalid. It may never silently discard damaged bytes and
resume with apparently valid evidence.

The host validates monotonically from the declared boundary through the entire
capture. It never resynchronizes after invalidity, and no command evidence can
support a green report until whole-stream validation and response correlation
succeed. Any framing, chain, session, sequence, envelope, or trailing-byte
invalidity makes the whole report red. A valid payload reporting denial remains
a command-level result; it does not make an invalid transport acceptable.

Hashes provide corruption detection, canonical linking, and run-local ordering
under the trusted harness. They do not provide cryptographic authenticity and
do not imply that serial bytes were persisted.

## Alternatives & independent opinions

R61 and R62 independently agreed on the important boundary: every
kernel-originated COM1 byte after transport start must be framed; raw text must
be typed payload; agent markers are not transport authority; durable
`RAIOSRC0` remains separate; and any malformed, truncated, replayed,
interleaved, or trailing material makes the report red without resynchronizing.

They meaningfully disagreed about encoding. R61 recommended a compact binary,
length-prefixed stream because its fixed header and bounded buffers map directly
to early and panic emission. R62 recommended ASCII line armor because the
current QEMU/TCP/PowerShell capture path and human serial tools are
line-oriented. V0 selects R62's line armor: it preserves exact framing through
the deployed text-oriented path and remains inspectable while retaining bounded
binary payloads through canonical base64. R61's binary stream is a credible
later version if measurements or a binary-clean capture path justify its
compatibility cost.

Literal durable `RAIOSRC0` reuse is rejected because sector padding, zero-tail
and torn-tail recovery, finite-region scanning, and persistence semantics are
wrong for UART. Hardened begin/end markers or newline-delimited JSON are also
rejected: they neither cover all output nor provide canonical lengths, global
ordering, chaining, replay detection, or an exclusive framing boundary.

## Consequences

Human-readable output now requires a decoder, and every COM1 producer must
ultimately pass through one framing transaction. In return, one strict parser
can establish completeness, ordering, corruption detection, and unambiguous
command attribution across boot, debug, response, and panic evidence.

This ADR and protocol document do not close either RECLOG scope checkbox.
Closure still requires centralized producer migration, a strict host parser and
report gate, host fixture negatives, a real QEMU positive, induced panic and
early-failure evidence, and independent review. Any implementation allocation
that needs inherited dirty `seed-kernel/src/main.rs` requires separate ownership
resolution; this decision does not silently allocate or adopt that file.
