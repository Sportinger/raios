use spin::Mutex;

pub const EVENT_CAPACITY: usize = 64;
pub const DEFAULT_EVENT_LIMIT: usize = 32;

const READ_EVIDENCE: &[&str] = &["computed_capability_grant"];
const DENIED_EVIDENCE: &[&str] = &["missing_required_evidence", "capability_denied"];

static LOG: Mutex<EventLog> = Mutex::new(EventLog::new());

#[derive(Clone, Copy)]
pub struct EventId {
    sequence: u64,
}

impl EventId {
    pub fn sequence(self) -> u64 {
        self.sequence
    }
}

#[derive(Clone, Copy)]
pub struct Event {
    pub sequence: u64,
    pub kind: &'static str,
    pub source_method: &'static str,
    pub source_transport: &'static str,
    pub classification: &'static str,
    pub outcome: &'static str,
    pub requested_capability: &'static str,
    pub risk: &'static str,
    pub subject: &'static str,
    pub resource: &'static str,
    pub reason: &'static str,
    pub evidence: &'static [&'static str],
}

#[derive(Clone, Copy)]
pub struct EventSnapshot {
    pub events: [Option<Event>; EVENT_CAPACITY],
    pub len: usize,
    pub limit: usize,
    pub capacity: usize,
    pub total_count: u64,
    pub dropped_before_sequence: u64,
}

struct EventLog {
    events: [Option<Event>; EVENT_CAPACITY],
    next_slot: usize,
    len: usize,
    next_sequence: u64,
}

impl EventLog {
    const fn new() -> Self {
        Self {
            events: [None; EVENT_CAPACITY],
            next_slot: 0,
            len: 0,
            next_sequence: 1,
        }
    }

    fn record(&mut self, mut event: Event) -> EventId {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        event.sequence = sequence;

        self.events[self.next_slot] = Some(event);
        self.next_slot = (self.next_slot + 1) % EVENT_CAPACITY;
        self.len = usize::min(self.len + 1, EVENT_CAPACITY);

        EventId { sequence }
    }

    fn snapshot_recent(&self, requested_limit: usize) -> EventSnapshot {
        let limit = normalize_limit(requested_limit);
        let want = usize::min(self.len, limit);
        let skip = self.len.saturating_sub(want);
        let oldest = if self.len == EVENT_CAPACITY {
            self.next_slot
        } else {
            0
        };

        let mut events = [None; EVENT_CAPACITY];
        let mut out_idx = 0usize;
        let mut idx = skip;
        while idx < self.len {
            let source = (oldest + idx) % EVENT_CAPACITY;
            events[out_idx] = self.events[source];
            out_idx += 1;
            idx += 1;
        }

        let total_count = self.next_sequence.saturating_sub(1);
        let dropped_before_sequence = if total_count > self.len as u64 {
            total_count - self.len as u64 + 1
        } else {
            0
        };

        EventSnapshot {
            events,
            len: out_idx,
            limit,
            capacity: EVENT_CAPACITY,
            total_count,
            dropped_before_sequence,
        }
    }
}

pub fn record_agent_read(
    source_method: &'static str,
    requested_capability: &'static str,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "agent_protocol.read_response",
        source_method,
        source_transport: "serial-console",
        classification: "public",
        outcome: "response",
        requested_capability,
        risk: "observe",
        subject: "agent.session.serial",
        resource: "current_boot",
        reason: "granted_read",
        evidence: READ_EVIDENCE,
    })
}

pub fn record_capability_denied(
    source_method: &'static str,
    requested_capability: &'static str,
    risk: &'static str,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "agent_protocol.capability_denied",
        source_method,
        source_transport: "serial-console",
        classification: "public",
        outcome: "capability_denied",
        requested_capability,
        risk,
        subject: "agent.session.serial",
        resource: "current_boot",
        reason: "missing_evidence",
        evidence: DENIED_EVIDENCE,
    })
}

pub fn snapshot_recent(limit: usize) -> EventSnapshot {
    LOG.lock().snapshot_recent(limit)
}

fn normalize_limit(limit: usize) -> usize {
    if limit == 0 {
        DEFAULT_EVENT_LIMIT
    } else {
        usize::min(limit, EVENT_CAPACITY)
    }
}
