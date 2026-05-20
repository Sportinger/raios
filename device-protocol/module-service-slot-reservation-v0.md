# raiOS Module Service-Slot Reservation V0

`raios.module_service_slot_reservation.v0` is the first current-boot evidence
record for a future RAM-only service slot. It is a diagnostic hash reference,
not an allocator and not load authority.

## Guest Method

```text
agent module.service_slot_diagnostic
agent module.service_slot_diagnostic <reservation_hash> <retained_reference_event_id> <retained_audit_rollback_reference_event_id> <computed_grant_hash> <audit_record_hash> <rollback_plan_hash> <pre_load_service_inventory_hash> <ram_only_service_slot_id> [current_boot]
agent module.service_slot_diagnostic_selftest
```

With no arguments, the method reports an absent reservation. With arguments, it
recomputes the canonical reservation hash and validates that the retained
computed-grant and audit/rollback reference ids are current-boot events. In the
live diagnostic path it also checks the latest retained grant and audit/rollback
references before retaining the reservation.

## Canonical Hash

The reservation hash is SHA-256 over these newline-separated lines:

```text
canonicalization=raios.module_service_slot_reservation.canonical.v0
schema=raios.module_service_slot_reservation.v0
load_mode=ram_only
scope=current_boot
retained_reference_event_id=<event.current_boot.NNNNNNNN>
retained_audit_rollback_reference_event_id=<event.current_boot.NNNNNNNN>
computed_capability_grant_sha256=<hex>
audit_record_sha256=<hex>
rollback_plan_sha256=<hex>
pre_load_service_inventory_sha256=<hex>
ram_only_service_slot_id=ram_only:<service id>
service_inventory_change=none
load_attempted=false
```

The accepted event binding keeps:

```text
classification: local_only
allocates_service_slot: false
creates_service_inventory_records: false
service_inventory_change: none
can_load_now: false
load_attempted: false
```

## Selftest

`module.service_slot_diagnostic_selftest` is local-only test infrastructure. It
does not mutate the global event log, create retained reservation records,
allocate service slots, load artifacts, or change `service.inventory.v0`. The
current cases cover absent, accepted-current-boot, stale, mismatched reservation
hash, and invalid `ram_only:` slot references.
