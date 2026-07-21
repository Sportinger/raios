# 0041 - PCI u16 writes use word cycles to preserve adjacent Status

Date: 2026-07-21 - Status: active

## Kontext

ADR 0040 requires BAR sizing to restore the prior PCI configuration state.
`PciAddress::write_u16` currently reads an aligned dword, replaces one half,
and writes the whole dword through CF8/CFC. At offset `0x04`, the upper half is
PCI Status; writing observed write-one-to-clear bits back as ones can erase
status while Command appears correctly restored. The host fake models a real
word write and therefore cannot validate that current production transport.

## Entscheidung

1. The generic PCI `write_u16` primitive performs a real 16-bit mechanism-one
   data cycle. CF8 receives the dword-aligned configuration address; the word
   is written to `CONFIG_DATA + (offset & 0x2)` under the existing PCI lock.
2. Odd u16 offsets are rejected. The public signature remains unchanged and
   no read-modify-write of the neighboring halfword is permitted.
3. The existing checked Command writer remains separate because its identity,
   expected-value, and readback semantics are stronger than a generic write.
4. Production-adjacent host tests must prove the planned CF8 value, CFC/CFE
   data port, word width, odd-offset rejection, W1C-sensitive fake behavior,
   and the existing BAR transaction/restore cases.
5. This decision does not make the multi-operation BAR sizing sequence one
   global transaction. Before the Surface capture and targeted driver sizing,
   initialization is serialized and no concurrent PCI-config owner is active.
   General runtime transaction locking is separate hardening and cannot be
   silently added to this unblocker slice.

## Alternativen & Zweitmeinungen

Two fresh independent read-only opinions agreed with high confidence that the
current dword read-modify-write can clear PCI Status W1C bits. Both preferred
correcting the generic primitive: its name already promises u16 semantics, all
current generic production callers write Command, and an existing checked path
already uses `outw` successfully.

A BAR-only Command writer was rejected because it would duplicate authority
and leave `enable_bus_master`, `disable_bus_master`, and `quiesce_function` on
the same generic trap. Changing only the fake or accepting the current code
would preserve a false-green restore proof. The opinions noted that per-access
locking does not serialize an entire sizing sequence; that independent risk is
bounded here by the pre-driver boot order rather than expanded scope.

## Folgen

The Surface gate now requires a small production correction in `pci.rs`, not
only stronger assertions. Command writes no longer replay adjacent Status bits,
upper-half u16 writes select CFE, and existing callers gain the same corrected
semantics without a new public API or unsafe mechanism. Real bridge/device
partial-write behavior and later concurrent runtime access remain hardware and
hardening risks; they are not claimed by the host proof.
