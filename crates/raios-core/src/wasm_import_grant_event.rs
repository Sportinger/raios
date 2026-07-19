//! Canonical durable events and the fail-closed authority fold for ADR 0023.

use alloc::{vec, vec::Vec};
use core::{cmp::Ordering, str};

use crate::{
    memory_record::{self, MemoryKind, MemoryRecord, MemoryRecordInput, MemorySource},
    record::{write_json, Field, Value},
    sha256_bytes, ByteSink,
};

pub const GRANT_PREDICATE: &str = "wasm_import.granted.v1";
pub const REVOKE_PREDICATE: &str = "wasm_import.revoked.v1";
pub const EVENT_SCHEMA: &str = "raios.wasm_import_grant_event.v1";
pub const EVENT_AUTHORITY: &str = "kernel_wasm_import_grant_fold.v1";
pub const MAX_PROJECTION_SLOTS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostImportId {
    EnvLog,
    EnvCounterGet,
}

impl HostImportId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EnvLog => "env.log",
            Self::EnvCounterGet => "env.counter_get",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "env.log" => Some(Self::EnvLog),
            "env.counter_get" => Some(Self::EnvCounterGet),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GrantScope {
    HostCall,
}

impl GrantScope {
    pub const fn as_str(self) -> &'static str {
        "host_call"
    }

    fn parse(value: &str) -> Option<Self> {
        (value == "host_call").then_some(Self::HostCall)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrantEventKind {
    Grant,
    Revoke,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrantEvent<'a> {
    pub record_id: &'a str,
    pub kind: GrantEventKind,
    pub service_id: &'a str,
    pub domain_instance: u64,
    pub binding_sha256: [u8; 32],
    pub host_import_id: HostImportId,
    pub scope: GrantScope,
    pub generation: u64,
    pub epoch: u64,
    pub parent_grant_id: &'a str,
    pub parent_grant_sha256: [u8; 32],
}

impl<'a> GrantEvent<'a> {
    pub fn grant(
        record_id: &'a str,
        service_id: &'a str,
        domain_instance: u64,
        binding_sha256: [u8; 32],
        host_import_id: HostImportId,
        generation: u64,
        epoch: u64,
    ) -> Self {
        Self {
            record_id,
            kind: GrantEventKind::Grant,
            service_id,
            domain_instance,
            binding_sha256,
            host_import_id,
            scope: GrantScope::HostCall,
            generation,
            epoch,
            parent_grant_id: "",
            parent_grant_sha256: [0; 32],
        }
    }

    pub fn revoke(
        record_id: &'a str,
        grant: &GrantEvent<'a>,
        epoch: u64,
        parent_grant_sha256: [u8; 32],
    ) -> Self {
        Self {
            record_id,
            kind: GrantEventKind::Revoke,
            service_id: grant.service_id,
            domain_instance: grant.domain_instance,
            binding_sha256: grant.binding_sha256,
            host_import_id: grant.host_import_id,
            scope: grant.scope,
            generation: grant.generation,
            epoch,
            parent_grant_id: grant.record_id,
            parent_grant_sha256,
        }
    }

    pub fn memory_record(&self) -> Result<MemoryRecord<'_>, GrantEventError> {
        if !self.shape_valid() {
            return Err(GrantEventError::Malformed);
        }
        MemoryRecord::new(MemoryRecordInput {
            id: self.record_id,
            kind: match self.kind {
                GrantEventKind::Grant => MemoryKind::CapabilityGrant.as_str(),
                GrantEventKind::Revoke => MemoryKind::CapabilityDenial.as_str(),
            },
            entity: self.service_id,
            predicate: match self.kind {
                GrantEventKind::Grant => GRANT_PREDICATE,
                GrantEventKind::Revoke => REVOKE_PREDICATE,
            },
            value: self.value(),
            classification: "local_only",
            authority: EVENT_AUTHORITY,
            boot_id: "origin_boot",
            sequence: self.epoch,
            source: MemorySource::new("wasm_import.grant_event", self.record_id),
            evidence: vec![],
            tags: vec!["wasm_import", "capability"],
            supersedes: vec![],
            created_at_ticks: 0,
        })
        .map_err(|_| GrantEventError::Malformed)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, GrantEventError> {
        let record = self.memory_record()?;
        let mut sink = VecSink(Vec::new());
        write_json(&record.to_record_value(), &mut sink, 0);
        Ok(sink.0)
    }

    pub fn record_sha256(&self) -> Result<[u8; 32], GrantEventError> {
        Ok(sha256_bytes(&self.canonical_bytes()?))
    }

    fn shape_valid(&self) -> bool {
        !self.record_id.is_empty()
            && !self.service_id.is_empty()
            && self.domain_instance != 0
            && self.binding_sha256 != [0; 32]
            && self.generation != 0
            && self.epoch != 0
            && match self.kind {
                GrantEventKind::Grant => {
                    self.parent_grant_id.is_empty() && self.parent_grant_sha256 == [0; 32]
                }
                GrantEventKind::Revoke => {
                    !self.parent_grant_id.is_empty() && self.parent_grant_sha256 != [0; 32]
                }
            }
    }

    fn value(&self) -> Value<'_> {
        Value::InlineObject(vec![
            Field::new("event_schema", Value::Str(EVENT_SCHEMA)),
            Field::new("service_id", Value::Str(self.service_id)),
            Field::new("domain_instance", Value::U64(self.domain_instance)),
            Field::new("binding_sha256", Value::Sha256(self.binding_sha256)),
            Field::new("host_import_id", Value::Str(self.host_import_id.as_str())),
            Field::new("scope", Value::Str(self.scope.as_str())),
            Field::new("generation", Value::U64(self.generation)),
            Field::new("epoch", Value::U64(self.epoch)),
            Field::new("parent_grant_id", Value::Str(self.parent_grant_id)),
            Field::new(
                "parent_grant_sha256",
                Value::Sha256(self.parent_grant_sha256),
            ),
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrantEventError {
    NotGrantEvent,
    Malformed,
    NonCanonical,
}

/// Dedicated value parser. Generic `memory_record::parse` deliberately skips the
/// value; this parser validates it and then byte-compares a canonical re-render.
pub fn parse_event(payload: &[u8]) -> Result<GrantEvent<'_>, GrantEventError> {
    let metadata = memory_record::parse(payload).map_err(|_| GrantEventError::NotGrantEvent)?;
    let kind = match (metadata.kind, metadata.predicate) {
        (MemoryKind::CapabilityGrant, GRANT_PREDICATE) => GrantEventKind::Grant,
        (MemoryKind::CapabilityDenial, REVOKE_PREDICATE) => GrantEventKind::Revoke,
        _ => return Err(GrantEventError::NotGrantEvent),
    };
    if metadata.classification.as_str() != "local_only"
        || metadata.authority != EVENT_AUTHORITY
        || metadata.boot_id != "origin_boot"
        || metadata.source.method != "wasm_import.grant_event"
        || metadata.source.record_id != metadata.id
        || metadata.evidence.len() != 0
        || metadata.tags.as_slice() != ["wasm_import", "capability"]
        || !metadata.supersedes.is_empty()
        || metadata.created_at_ticks != 0
    {
        return Err(GrantEventError::Malformed);
    }

    let value_start = find_bytes(payload, b"\"value\": ").ok_or(GrantEventError::Malformed)?
        + b"\"value\": ".len();
    let mut c = Cursor::new(&payload[value_start..]);
    c.byte(b'{')?;
    c.key("event_schema")?;
    if c.string()? != EVENT_SCHEMA {
        return Err(GrantEventError::Malformed);
    }
    c.comma()?;
    c.key("service_id")?;
    let service_id = c.string()?;
    c.comma()?;
    c.key("domain_instance")?;
    let domain_instance = c.u64()?;
    c.comma()?;
    c.key("binding_sha256")?;
    let binding_sha256 = c.sha256()?;
    c.comma()?;
    c.key("host_import_id")?;
    let host_import_id = HostImportId::parse(c.string()?).ok_or(GrantEventError::Malformed)?;
    c.comma()?;
    c.key("scope")?;
    let scope = GrantScope::parse(c.string()?).ok_or(GrantEventError::Malformed)?;
    c.comma()?;
    c.key("generation")?;
    let generation = c.u64()?;
    c.comma()?;
    c.key("epoch")?;
    let epoch = c.u64()?;
    c.comma()?;
    c.key("parent_grant_id")?;
    let parent_grant_id = c.string()?;
    c.comma()?;
    c.key("parent_grant_sha256")?;
    let parent_grant_sha256 = c.sha256()?;
    c.byte(b'}')?;

    let event = GrantEvent {
        record_id: metadata.id,
        kind,
        service_id,
        domain_instance,
        binding_sha256,
        host_import_id,
        scope,
        generation,
        epoch,
        parent_grant_id,
        parent_grant_sha256,
    };
    if metadata.entity != service_id || metadata.sequence != epoch || !event.shape_valid() {
        return Err(GrantEventError::Malformed);
    }
    if event.canonical_bytes()?.as_slice() != payload {
        return Err(GrantEventError::NonCanonical);
    }
    Ok(event)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionSlot<'a> {
    pub grant_id: &'a str,
    pub service_id: &'a str,
    pub domain_instance: u64,
    pub binding_sha256: [u8; 32],
    pub host_import_id: HostImportId,
    pub scope: GrantScope,
    pub generation: u64,
    pub grant_epoch: u64,
    pub revoked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldError {
    Malformed,
    MissingParent,
    MalformedLink,
    Fork,
    RepeatedOrNonMonotonicEpoch,
    AmbiguousHistory,
    CapacityOverflow,
}

pub struct GrantProjection<'a> {
    slots: Vec<ProjectionSlot<'a>>,
    hash: [u8; 32],
}

impl<'a> GrantProjection<'a> {
    pub fn slots(&self) -> &[ProjectionSlot<'a>] {
        &self.slots
    }
    pub const fn sha256(&self) -> [u8; 32] {
        self.hash
    }
    pub fn is_live(&self, service_id: &str, domain_instance: u64, surface: HostImportId) -> bool {
        self.slots.iter().any(|slot| {
            slot.service_id == service_id
                && slot.domain_instance == domain_instance
                && slot.host_import_id == surface
                && !slot.revoked
        })
    }
}

/// Pure ordered fold. Any ambiguity rejects the entire projection; callers must
/// install an empty/denied table on `Err`, never authority from a valid prefix.
pub fn fold_events<'a>(events: &[GrantEvent<'a>]) -> Result<GrantProjection<'a>, FoldError> {
    let mut slots: Vec<ProjectionSlot<'a>> = Vec::new();
    let mut prior_epoch = 0u64;
    let mut ids: Vec<&str> = Vec::new();
    for event in events {
        if !event.shape_valid() {
            return Err(FoldError::Malformed);
        }
        if event.epoch <= prior_epoch {
            return Err(FoldError::RepeatedOrNonMonotonicEpoch);
        }
        prior_epoch = event.epoch;
        if ids.iter().any(|id| *id == event.record_id) {
            return Err(FoldError::AmbiguousHistory);
        }
        ids.push(event.record_id);
        match event.kind {
            GrantEventKind::Grant => {
                if slots.len() == MAX_PROJECTION_SLOTS {
                    return Err(FoldError::CapacityOverflow);
                }
                if slots
                    .iter()
                    .any(|slot| same_authority(slot, event) && !slot.revoked)
                {
                    return Err(FoldError::AmbiguousHistory);
                }
                if slots
                    .iter()
                    .any(|slot| same_authority(slot, event) && slot.generation >= event.generation)
                {
                    return Err(FoldError::RepeatedOrNonMonotonicEpoch);
                }
                slots.push(ProjectionSlot {
                    grant_id: event.record_id,
                    service_id: event.service_id,
                    domain_instance: event.domain_instance,
                    binding_sha256: event.binding_sha256,
                    host_import_id: event.host_import_id,
                    scope: event.scope,
                    generation: event.generation,
                    grant_epoch: event.epoch,
                    revoked: false,
                });
            }
            GrantEventKind::Revoke => {
                let Some(index) = slots
                    .iter()
                    .position(|slot| slot.grant_id == event.parent_grant_id)
                else {
                    return Err(FoldError::MissingParent);
                };
                if slots[index].revoked {
                    return Err(FoldError::Fork);
                }
                if !same_authority(&slots[index], event)
                    || slots[index].generation != event.generation
                {
                    return Err(FoldError::MalformedLink);
                }
                let parent = events
                    .iter()
                    .find(|candidate| candidate.record_id == event.parent_grant_id)
                    .ok_or(FoldError::MissingParent)?;
                if parent.kind != GrantEventKind::Grant
                    || parent.record_sha256().map_err(|_| FoldError::Malformed)?
                        != event.parent_grant_sha256
                {
                    return Err(FoldError::MalformedLink);
                }
                slots[index].revoked = true;
            }
        }
    }
    slots.sort_by(compare_slots);
    let hash = projection_hash(&slots);
    Ok(GrantProjection { slots, hash })
}

fn same_authority(slot: &ProjectionSlot<'_>, event: &GrantEvent<'_>) -> bool {
    slot.service_id == event.service_id
        && slot.domain_instance == event.domain_instance
        && slot.binding_sha256 == event.binding_sha256
        && slot.host_import_id == event.host_import_id
        && slot.scope == event.scope
}

fn compare_slots(a: &ProjectionSlot<'_>, b: &ProjectionSlot<'_>) -> Ordering {
    (
        a.service_id,
        a.domain_instance,
        a.binding_sha256,
        a.host_import_id,
        a.scope,
        a.generation,
        a.grant_id,
    )
        .cmp(&(
            b.service_id,
            b.domain_instance,
            b.binding_sha256,
            b.host_import_id,
            b.scope,
            b.generation,
            b.grant_id,
        ))
}

fn projection_hash(slots: &[ProjectionSlot<'_>]) -> [u8; 32] {
    let mut bytes = Vec::new();
    for slot in slots {
        bytes.extend_from_slice(slot.service_id.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&slot.domain_instance.to_le_bytes());
        bytes.extend_from_slice(&slot.binding_sha256);
        bytes.extend_from_slice(slot.host_import_id.as_str().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(slot.scope.as_str().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&slot.generation.to_le_bytes());
        bytes.extend_from_slice(slot.grant_id.as_bytes());
        bytes.push(0);
        bytes.push(u8::from(slot.revoked));
    }
    sha256_bytes(&bytes)
}

struct VecSink(Vec<u8>);
impl ByteSink for VecSink {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn byte(&mut self, expected: u8) -> Result<(), GrantEventError> {
        if self.bytes.get(self.pos) == Some(&expected) {
            self.pos += 1;
            Ok(())
        } else {
            Err(GrantEventError::Malformed)
        }
    }
    fn comma(&mut self) -> Result<(), GrantEventError> {
        self.byte(b',')?;
        self.byte(b' ')
    }
    fn key(&mut self, expected: &str) -> Result<(), GrantEventError> {
        self.byte(b'"')?;
        let end = self.bytes[self.pos..]
            .iter()
            .position(|b| *b == b'"')
            .ok_or(GrantEventError::Malformed)?
            + self.pos;
        if self.bytes.get(self.pos..end) != Some(expected.as_bytes()) {
            return Err(GrantEventError::Malformed);
        }
        self.pos = end + 1;
        self.byte(b':')?;
        self.byte(b' ')
    }
    fn string(&mut self) -> Result<&'a str, GrantEventError> {
        self.byte(b'"')?;
        let start = self.pos;
        let end = self.bytes[start..]
            .iter()
            .position(|b| *b == b'"')
            .ok_or(GrantEventError::Malformed)?
            + start;
        if self.bytes[start..end].contains(&b'\\') {
            return Err(GrantEventError::Malformed);
        }
        self.pos = end + 1;
        str::from_utf8(&self.bytes[start..end]).map_err(|_| GrantEventError::Malformed)
    }
    fn u64(&mut self) -> Result<u64, GrantEventError> {
        let start = self.pos;
        let mut value = 0u64;
        while let Some(b'0'..=b'9') = self.bytes.get(self.pos) {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add((self.bytes[self.pos] - b'0') as u64))
                .ok_or(GrantEventError::Malformed)?;
            self.pos += 1;
        }
        if self.pos == start || (self.pos - start > 1 && self.bytes[start] == b'0') {
            Err(GrantEventError::Malformed)
        } else {
            Ok(value)
        }
    }
    fn sha256(&mut self) -> Result<[u8; 32], GrantEventError> {
        let text = self.string()?;
        crate::parse_sha256_ref(text).ok_or(GrantEventError::Malformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grant<'a>(id: &'a str, epoch: u64, generation: u64) -> GrantEvent<'a> {
        GrantEvent::grant(
            id,
            "svc.echo",
            7,
            [0x42; 32],
            HostImportId::EnvCounterGet,
            generation,
            epoch,
        )
    }
    fn revoke<'a>(id: &'a str, grant: &GrantEvent<'a>, epoch: u64) -> GrantEvent<'a> {
        GrantEvent::revoke(id, grant, epoch, grant.record_sha256().unwrap())
    }

    #[test]
    fn canonical_round_trip_and_valid_grant_revoke_fold() {
        let g = grant("cap.grant.1", 1, 1);
        let r = revoke("cap.revoke.2", &g, 2);
        let bytes = g.canonical_bytes().unwrap();
        assert_eq!(parse_event(&bytes).unwrap(), g);
        let projection = fold_events(&[g, r]).unwrap();
        assert_eq!(projection.slots().len(), 1);
        assert!(projection.slots()[0].revoked);
        assert!(!projection.is_live("svc.echo", 7, HostImportId::EnvCounterGet));
        assert_eq!(projection.sha256(), fold_events(&[g, r]).unwrap().sha256());
    }

    #[test]
    fn malformed_link_fork_missing_parent_and_epoch_fail_closed() {
        let g = grant("cap.grant.1", 1, 1);
        let r = revoke("cap.revoke.2", &g, 2);
        let mut bad = r;
        bad.parent_grant_sha256 = [9; 32];
        assert!(matches!(
            fold_events(&[g, bad]),
            Err(FoldError::MalformedLink)
        ));
        let orphan = GrantEvent {
            parent_grant_id: "missing",
            ..r
        };
        assert!(matches!(
            fold_events(&[g, orphan]),
            Err(FoldError::MissingParent)
        ));
        let r2 = GrantEvent {
            record_id: "cap.revoke.3",
            epoch: 3,
            ..r
        };
        assert!(matches!(fold_events(&[g, r, r2]), Err(FoldError::Fork)));
        let repeated = GrantEvent {
            record_id: "cap.revoke.x",
            epoch: 1,
            ..r
        };
        assert!(matches!(
            fold_events(&[g, repeated]),
            Err(FoldError::RepeatedOrNonMonotonicEpoch)
        ));
    }

    #[test]
    fn duplicate_ambiguous_and_capacity_overflow_fail_closed() {
        let g = grant("cap.grant.1", 1, 1);
        let duplicate = GrantEvent { epoch: 2, ..g };
        assert!(matches!(
            fold_events(&[g, duplicate]),
            Err(FoldError::AmbiguousHistory)
        ));
        let r = revoke("cap.revoke.2", &g, 2);
        let stale_generation = GrantEvent {
            record_id: "cap.grant.3",
            kind: GrantEventKind::Grant,
            epoch: 3,
            ..g
        };
        assert!(matches!(
            fold_events(&[g, r, stale_generation]),
            Err(FoldError::RepeatedOrNonMonotonicEpoch)
        ));
        let mut events = Vec::new();
        for index in 0..=MAX_PROJECTION_SLOTS {
            events.push(GrantEvent::grant(
                "unique",
                "svc.echo",
                index as u64 + 1,
                [index as u8 + 1; 32],
                HostImportId::EnvCounterGet,
                1,
                index as u64 + 1,
            ));
        }
        // Make ids unique without allocation by using a static list.
        let ids = ["g0", "g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"];
        for (event, id) in events.iter_mut().zip(ids) {
            event.record_id = id;
        }
        assert!(matches!(
            fold_events(&events),
            Err(FoldError::CapacityOverflow)
        ));
    }

    #[test]
    fn parser_rejects_noncanonical_and_tampered_value() {
        let g = grant("cap.grant.1", 1, 1);
        let mut bytes = g.canonical_bytes().unwrap();
        let pos = find_bytes(&bytes, b"env.counter_get").unwrap();
        bytes[pos] = b'E';
        assert!(parse_event(&bytes).is_err());
        let mut whitespace = g.canonical_bytes().unwrap();
        whitespace.insert(0, b' ');
        assert_eq!(parse_event(&whitespace), Err(GrantEventError::NonCanonical));
    }
}
