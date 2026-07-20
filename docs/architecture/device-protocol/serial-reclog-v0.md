# raiOS Serial RECLOG V0

This document normatively defines the ASCII line-armored live COM1 evidence
transport selected by ADR 0032. The key words MUST, MUST NOT, REQUIRED, SHALL,
SHALL NOT, SHOULD, SHOULD NOT, and MAY are normative.

Serial RECLOG V0 is not durable `RAIOSRC0`. It has distinct bytes, identity,
ordering, termination, and failure semantics. It claims neither persistence nor
cryptographic authenticity.

## Validated epoch and preamble

The host capture contract MUST declare a `preamble_length` in bytes before
parsing. It MUST be in `0..=4096`, fixed before validation, and derived from the
harness's known handoff to raiOS COM1 ownership, not from searching the capture.
Exactly those bytes MAY contain firmware or pre-kernel output. The byte at that
offset MUST begin the unique session-start frame. Kernel-originated bytes MUST
NOT be classified as preamble.

Before the first COM1 byte, the trusted run orchestrator MUST generate a fresh,
nonzero, uniformly selected 128-bit `expected_sid`, record it as unique in the
run ledger, and provision that same value to the kernel as a trusted
out-of-band boot/launch fact. The exact provisioning mechanism is a harness and
kernel implementation choice; it MUST NOT be derived from serial bytes. The
validator MUST receive `expected_sid` from the run ledger, and the first frame's
`sid` MUST equal it. A validator MUST NOT infer or learn the expected identity
from the transcript itself.

From the first byte at that offset through capture termination, every byte MUST
belong to exactly one valid frame. The parser MUST consume forward only. It MUST
NOT scan for a later magic, discard an invalid line, accept a valid prefix, or
resume after any invalid byte. A graceful `session_end` frame, when present,
MUST be final; any later byte is invalid. Capture without `session_end` MAY be
complete only when it ends immediately after CRLF, all message envelopes are
complete, and the run contract permits that external termination mode.

## Physical frame

Each frame is exactly one ASCII physical line with fields in this exact order:

```text
RAIOSRS0|v=0|sid=<sid>|seq=<seq>|kind=<kind>|flags=0|method=<method>|corr=<corr>|msg=<msg>|chunk=<index>/<count>|dlen=<dlen>|elen=<elen>|tlen=<tlen>|psha=<psha>|tsha=<tsha>|prev=<prev>|payload=<payload>|fsha=<fsha>\r\n
```

No spaces, tabs, extra separators, reordered fields, unknown fields, alternate
names, unknown flags, or non-ASCII bytes are permitted. No field value may be
empty except the `payload` value. The `payload` value MUST be empty if and only
if both `dlen=0` and `elen=0`; `payload=` is the sole canonical representation
of that empty value. Every other field value MUST be nonempty.
Termination is exactly bytes `0d 0a`; bare LF, bare CR, or CR/LF inside a frame
is invalid. The complete line, including CRLF, MUST NOT exceed 4096 bytes.

Canonical scalar encodings are:

- `<sid>`, `<corr>`, and `<msg>` are 32 lowercase hexadecimal digits when
  present. `sid` is always present; an absent `corr` or `msg` is exactly `-`.
- `<seq>`, lengths, chunk index, and chunk count are minimal unsigned ASCII
  decimal without a sign or leading zero, except the value zero is `0`.
- `<method>` is `-` when absent; otherwise it is 1 through 96 bytes matching
  `[a-z][a-z0-9_.-]*`. Aliases MUST be replaced with the canonical method name
  before framing.
- `<kind>` is exactly one of `session_start`, `diagnostic`, `boot`,
  `agent_response`, `panic`, `message_chunk`, or `session_end`.
- `<psha>`, `<tsha>`, `<prev>`, and `<fsha>` are exactly 64 lowercase
  hexadecimal digits representing 32 bytes in display order.
- `<payload>` is RFC 4648 base64 with the standard alphabet, required `=`
  padding, no whitespace, and the shortest canonical encoding. An empty payload
  has an empty value (`payload=`) and `elen=0`.

`dlen` is the decoded payload byte count and MUST be at most 2048. `elen` is the
ASCII byte count of `<payload>` and MUST equal `4 * ceil(dlen / 3)` (zero when
`dlen` is zero). Decoding and re-encoding the payload MUST reproduce the field
byte-for-byte. `psha` is SHA-256 of the decoded payload, including the standard
empty-input digest when `dlen=0`.

SHA-256 always means FIPS 180-4 SHA-256. Hash displays are lowercase hex. No
text decoding, newline conversion, Unicode normalization, or terminal
translation occurs before hashing.

## Frame hash and chain

Let `H` be the exact ASCII bytes from the first `R` of `RAIOSRS0` through the
last base64 byte (or the `=` in `payload=` for an empty payload), including all
shown separators and field names, but excluding `|fsha=`, `fsha`, and CRLF.
`fsha` MUST equal `SHA-256(H)`. Thus implementations can independently build the
canonical prefix, hash it once, append `|fsha=<fsha>\r\n`, and compare exact
bytes.

The first frame MUST have `seq=0`, `prev` equal to 64 zeroes, and
`kind=session_start`. Every later frame MUST keep the same `sid`, set `prev` to
the immediately preceding frame's decoded `fsha`, and use the preceding
sequence plus one. Sequence arithmetic is checked `u64`: after sequence
`18446744073709551615`, no further frame can be represented or emitted.
Wraparound, saturation to a duplicate value, a gap, regression, or duplicate
is invalid and makes the report red. Emitters MUST deny an emission that would
overflow; if bytes are nevertheless required after exhaustion, the evidence
run cannot be green.

The `sid` MUST equal the run ledger's fresh `expected_sid` for the boot/capture
and MUST NOT be all zeroes. The run ledger MUST reject generation or allocation
of an `expected_sid` already recorded for any retained V0 evidence run; ledger
retention MUST cover every transcript that remains eligible for validation or
comparison. Reusing a session identity, inserting a frame from another session,
replaying a prior frame, or replaying a complete old transcript under a
different expected identity is invalid. A launcher that cannot provision and
retain a fresh expected identity cannot produce green V0 evidence. This is run
binding under the trusted harness, not kernel authenticity: `sid` is neither a
secret nor a signature. Hashes detect structure and accidental or adversarial
mutation only under that trusted capture model; there is no signing key or
authenticity claim.

## Kinds, correlation, and chunking

The unique `session_start` MUST be the first frame and MUST have absent method,
correlation, and message identity, `chunk=0/1`, and `tlen=dlen`. A second
session-start is invalid. `diagnostic`, `boot`, and `panic` carry opaque bytes;
their method, correlation, and message identity MUST be absent, with
`chunk=0/1` and `tlen=dlen`.

An unchunked `agent_response` MUST carry a canonical method and correlation,
absent message identity, `chunk=0/1`, and `tlen=dlen`. Every logical
`agent_response` with decoded length `0..=2048` MUST use exactly this unchunked
form. Its payload MAY remain a `raios.agent.v0` JSON compatibility envelope.
Any legacy `RAIOS_AGENT_BEGIN/END` strings are merely payload bytes and have no
transport authority.

A logical `agent_response` with decoded length `2049..=524288` MUST use exactly
`2..=256` consecutive `message_chunk` frames; `count=1` is invalid. Every chunk
MUST be nonempty, MUST have decoded length at most 2048, and MUST have the same
present `msg`, `method`, and `corr`. `index` is zero-based and strictly
increases through `count-1`; `tlen` is the common decoded logical length; and
`tsha` is SHA-256 of the complete decoded logical payload. No other frame may
intervene. Their decoded bytes, concatenated in index order, MUST have length
`tlen` and hash `tsha`. A logical payload longer than 524288 bytes is invalid
and MUST NOT be truncated implicitly. For every unchunked frame, `tsha` MUST
equal `psha`. V0 rejects unknown kinds; `message_chunk` is valid only for an
`agent_response` in the specified size range, and no other kind may be chunked.

`session_end` has absent method, correlation, and message identity,
`chunk=0/1`, `dlen=0`, `elen=0`, `tlen=0`, and the empty-input digest for both
`psha` and `tsha`. It closes transport emission.

Raw boot, debug, and panic text MUST be placed in the corresponding typed
payload. There is no out-of-band kernel text after session start.

## Emitter requirements

One transport owner MUST reserve the next sequence, chain predecessor, and UART
for the complete encoded line. A frame is committed only by emitting all bytes
through CRLF in one non-interleavable transaction. Producers MUST NOT assemble
a frame from separately locked writes. Partial lines, concurrent splicing, and
bytes written around the transport owner invalidate the stream.

Session start, early boot, diagnostic, and panic emission MUST use static or
bounded caller-provided storage and MUST NOT require heap allocation. Formatting
MUST fail before UART emission if the canonical line would exceed its bounds.
Normal logical messages may stream bounded chunks; an implementation need not
allocate the complete message if it can supply the declared total length and
hash without weakening the contract.

Panic emission MUST disable local interruption and use a bounded,
allocation-free payload. It MAY acquire the normal writer only without
blocking. If reentrancy or lock ownership prevents a guaranteed complete
frame, it MAY use an architecture-proven emergency single-writer path only when
no other writer can continue. Otherwise it MUST emit no purportedly valid
replacement evidence and the incomplete/missing required panic evidence makes
the report red. If an emergency attempt emits even one byte and then fails, the
partial frame makes the stream red; later frames MUST NOT be used to
resynchronize or recover a green result. Explicit payload truncation is allowed
only when represented inside a complete typed panic payload.

## Host validation and command binding

The host MUST retain the original capture bytes and validate from the declared
boundary through the complete epoch before any decoded payload or command can
support green evidence. It MUST obtain `expected_sid` from the trusted run
ledger before validation and compare the first frame against it; missing,
learned-from-transcript, reused, or mismatched expected identity is report-red.
Validation order is physical line, exact grammar and canonical encoding,
lengths, payload hash, frame hash, expected session and sequence chain, kind
rules, chunk reconstruction, then payload-layer contracts. A failure at any
stage is report-level red and terminal for parsing.

For every required command, the run ledger supplies one expected canonical
method and correlation. Exactly one complete validated response with both
values MUST exist. Zero responses, duplicate responses (even byte-identical),
an unexpected response, method or correlation mismatch, a response split
across an incomplete envelope, or an otherwise unaccounted correlation makes
the report red. Payload JSON, when used, MUST be complete and its method and
correlation MUST agree with the transport fields. A fully valid response whose
payload says `denied` is a command-level denial, not transport corruption; run
policy decides whether that command result is acceptable.

## Report-wide rejection matrix

Every condition below invalidates the entire report; none may be skipped,
warned away, or repaired by a later frame:

| Class | Required rejection |
|---|---|
| Truncation | truncated field/header, base64 payload, physical line/CRLF, chunk set, envelope, or final frame |
| Grammar | bad magic or version; unknown/reordered/duplicate field; unknown kind or flag; non-ASCII; non-canonical decimal, hex, method, base64, or line ending |
| Length/integrity | encoded or decoded length mismatch; empty non-payload value; noncanonical empty payload; oversize frame, chunk, chunk count, or logical message; one-chunk or small-payload chunking; bad payload, total, frame, or previous-frame hash |
| Identity/order | missing/duplicate session start; missing, transcript-learned, reused, wrong, or changed expected session; sequence gap, regression, duplicate, overflow/wrap; cross-session, same-session, or whole-transcript replay |
| Exclusivity | interleaved frame bytes; any unframed kernel byte; forged magic or marker treated as framing; valid prefix plus garbage; bytes after session end |
| Commands | method/correlation mismatch; missing, duplicate, unexpected, replayed, or incomplete response; payload-envelope mismatch |
| Capture end | partial final frame, incomplete chunk/envelope, unresolved required correlation, or an invalid final frame |

In particular, a valid prefix never makes an invalid suffix acceptable, and a
later magic never restores validity after a malformed prefix.

## Closure boundary

This specification and ADR 0032 do not close either scope checkbox. Closure
requires centralized migration of every kernel COM1 producer, a strict host
parser and report gate, independent host fixtures for every rejection class, a
real QEMU positive, induced early-failure and panic evidence, and independent
review. Any successor allocation that requires inherited dirty
`seed-kernel/src/main.rs` MUST first resolve ownership separately; this document
does not allocate or adopt that file.
