# 0039 — Surface facts use one versioned record per RECLOG frame

Date: 2026-07-21 · Status: active

## Kontext

ADR 0038 requires one bounded, canonical Surface capture in hash-chained USB
RECLOG. The accepted `surface_fact_capture` model defines and validates CPU,
SMBIOS-memory, Limine-memory-map, PCI, and completion facts, but deliberately
does not define their bytes. Kernel persistence and the host extractor cannot
proceed independently until that shared wire boundary is frozen.

Two fresh independent read-only Codex opinions derived a format from the model.
They agreed on the framing, field widths, limits, fail-closed behavior, and hash
separation. They differed on the eight-byte magic and the order of capture ID
versus part coordinates in the fixed header.

## Entscheidung

1. One `CaptureRecord` is encoded as the complete payload of exactly one
   durable `RAIOSRC0` RECLOG frame. Capture records are never concatenated and
   the RECLOG header or sector padding is never part of the inner record.
2. Surface Fact Wire V1 has a 40-byte, padding-free header:
   `magic[8] = "RAIOSSF0"`, `wire_version:u16 = 1`, `header_len:u16 = 40`,
   `record_len:u16`, `payload_len:u16`, `schema_version:u16 = 1`,
   `part_kind:u8`, `flags:u8 = 0`, `capture_id[16]`, `part_index:u16`, and
   `part_count:u16`, in that byte order. Every multibyte integer is little
   endian. `record_len` must equal both the input length and
   `header_len + payload_len`; trailing bytes are invalid.
3. Payloads are fixed by `part_kind`: CPU is 24 bytes, Limine memory region 20,
   completion 40, SMBIOS memory `19 + locator_len`, and PCI
   `16 + 19 * bar_count`. Locator bytes are the model's bounded ASCII without a
   terminator. Only active PCI BARs are encoded and they must be strictly
   ascending by index. A record is at most 170 bytes and fits one 512-byte
   RECLOG frame.
4. The codec is allocation-free. Encoding uses a caller-provided buffer;
   decoding returns the fixed-size model value. Unknown magic/version/schema,
   type, flag, BAR kind/status, noncanonical order, arithmetic overflow,
   truncation, excess length, or any model validation failure is rejected.
   There is no skip-unknown compatibility path; extensions require a new wire
   version.
5. A series is physically contiguous and ordered by `part_index` from zero.
   Every part has the same versions, capture ID, and count. Exactly one
   completion record is final; a foreign RECLOG payload inside an active series,
   EOF before completion, duplicate/missing/mixed parts, or capture bytes after
   completion rejects the selected series. The caller invokes
   `validate_capture` after decoding the bounded series.
6. Hash layers remain distinct. The existing completion `facts_sha256` binds
   canonical semantic non-completion facts. The RECLOG payload hash binds the
   exact inner wire bytes; the RECLOG frame hash also binds sequence,
   predecessor, and zero padding. The codec adds no redundant inner checksum.
7. Implementation lives in a new
   `crates/raios-core/src/surface_fact_capture_wire.rs` module, exported by
   `lib.rs`. The already 797-line model/validator is not expanded merely to
   avoid a module boundary. Wire canonicality rejects unsorted BARs without
   changing the more general in-memory validator.
8. This protocol carries development evidence under Owner custody. It does not
   assert attestation, freshness, machine identity, or manifest acceptance.

## Alternativen & Zweitmeinungen

Both opinions rejected Serde/bincode/postcard, generic skippable TLV, fixed
192/256-byte padded records, an additional inner hash, and a single capture
blob. Those choices either weaken canonical fail-closed parsing, add an ABI or
dependency, create malleable padding, duplicate existing hashes, or exceed the
4,008-byte RECLOG payload limit.

One opinion proposed magic `RAIOSSFC`, placed part index/count before capture
ID, and suggested adding BAR ordering to the semantic validator. The other
proposed `RAIOSSF0`, placed capture ID before part coordinates, and kept series
and wire canonicality at the codec boundary. We choose `RAIOSSF0` because it is
visibly distinct while following the existing `RAIOSRC0`/`RAIOSRS0` envelope
convention. Grouping the capture ID before its ordinal coordinates makes the
series identity contiguous. BAR ordering remains a wire rule because the
existing semantic digest already binds the model's explicit BAR array order.

Both opinions suggested appending the codec to the existing model file. We use
a separate module instead: this changes no protocol decision and keeps the
accepted model/validator readable and independently reviewable.

## Folgen

The next lane can implement one small no_std codec with byte-exact golden
vectors and mutation negatives. A maximal 128-part capture consumes at most
65,536 bytes of aligned durable RECLOG storage. Once the codec is accepted and
pushed, kernel capture/persistence and the host extractor have a stable shared
boundary and can run as disjoint parallel lanes.

Forward evolution is intentionally strict: a new field or type requires a new
wire version and explicit decoder support. This costs version work later but
prevents an extractor from silently accepting evidence it does not understand.
