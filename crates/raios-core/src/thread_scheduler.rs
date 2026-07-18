//! Deterministic green-thread scheduling policy without engine or clock types.

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use core::cmp::Ordering;

pub const DEFAULT_THREAD_CAP: u32 = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(u32);

impl ThreadId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemSlot(u32);

impl MemSlot {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Addr(u64);

impl Addr {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Runde(u64);

impl Runde {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    fn checked_add(self, rounds: u64) -> Option<Self> {
        self.0.checked_add(rounds).map(Self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadState {
    Runnable,
    ParkedWait {
        mem: MemSlot,
        addr: Addr,
        deadline: Option<Runde>,
        seq: u64,
    },
    ParkedHost,
    Woken {
        result: u32,
    },
    Exited,
}

impl ThreadState {
    fn is_schedulable(self) -> bool {
        matches!(self, Self::Runnable | Self::Woken { .. })
    }

    fn kind(self) -> ThreadStateKind {
        match self {
            Self::Runnable => ThreadStateKind::Runnable,
            Self::ParkedWait { .. } => ThreadStateKind::ParkedWait,
            Self::ParkedHost => ThreadStateKind::ParkedHost,
            Self::Woken { .. } => ThreadStateKind::Woken,
            Self::Exited => ThreadStateKind::Exited,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadStateKind {
    Runnable,
    ParkedWait,
    ParkedHost,
    Woken,
    Exited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobState {
    Running,
    JobExited { code: u32 },
    JobDeadlocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerOperation {
    ParkWait,
    ParkHost,
    WakeHost,
    TakeWakeResult,
    QuantumEnd,
    ThreadExit,
    KillThread,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerError {
    ThreadCap {
        cap: u32,
    },
    ThreadIdExhausted,
    WaitSequenceExhausted,
    UnknownThread {
        thread: ThreadId,
    },
    IllegalTransition {
        thread: ThreadId,
        from: ThreadStateKind,
        operation: SchedulerOperation,
    },
    NotCurrentThread {
        expected: Option<ThreadId>,
        actual: ThreadId,
    },
    RoundWentBackwards {
        current: Runde,
        requested: Runde,
    },
    RoundOverflow {
        base: Runde,
        timeout_rounds: u64,
    },
    JobExited {
        code: u32,
    },
    JobAlreadyExited {
        code: u32,
    },
    JobDeadlocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeReason {
    Notify,
    Host,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadExitReason {
    Returned,
    Killed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceEvent {
    Spawn {
        thread: ThreadId,
    },
    ParkWait {
        thread: ThreadId,
        mem: MemSlot,
        addr: Addr,
        deadline: Option<Runde>,
        seq: Option<u64>,
    },
    ParkHost {
        thread: ThreadId,
    },
    Wake {
        thread: ThreadId,
        result: u32,
        reason: WakeReason,
    },
    Timeout {
        thread: ThreadId,
        deadline: Runde,
        seq: Option<u64>,
    },
    Switch {
        from: Option<ThreadId>,
        to: ThreadId,
    },
    ThreadExit {
        thread: ThreadId,
        reason: ThreadExitReason,
    },
    JobExit {
        code: u32,
    },
    Deadlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Waiter {
    thread: ThreadId,
    seq: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Expiry {
    thread: ThreadId,
    mem: MemSlot,
    addr: Addr,
    deadline: Runde,
    seq: u64,
}

impl Ord for Expiry {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.deadline, self.seq, self.thread).cmp(&(other.deadline, other.seq, other.thread))
    }
}

impl PartialOrd for Expiry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
pub struct JobThreadScheduler {
    thread_cap: u32,
    next_tid: u64,
    next_wait_seq: u64,
    current_round: Runde,
    threads: BTreeMap<ThreadId, ThreadState>,
    wait_queues: BTreeMap<(MemSlot, Addr), VecDeque<Waiter>>,
    current: Option<ThreadId>,
    last_run: Option<ThreadId>,
    job_state: JobState,
    trace: Vec<TraceEvent>,
}

impl Default for JobThreadScheduler {
    fn default() -> Self {
        Self::with_thread_cap(DEFAULT_THREAD_CAP)
    }
}

impl JobThreadScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_thread_cap(thread_cap: u32) -> Self {
        Self {
            thread_cap,
            next_tid: 0,
            next_wait_seq: 0,
            current_round: Runde::new(0),
            threads: BTreeMap::new(),
            wait_queues: BTreeMap::new(),
            current: None,
            last_run: None,
            job_state: JobState::Running,
            trace: Vec::new(),
        }
    }

    pub const fn thread_cap(&self) -> u32 {
        self.thread_cap
    }

    pub const fn current_round(&self) -> Runde {
        self.current_round
    }

    pub const fn job_state(&self) -> JobState {
        self.job_state
    }

    pub fn trace(&self) -> &[TraceEvent] {
        &self.trace
    }

    pub fn thread_state(&self, thread: ThreadId) -> Result<ThreadState, SchedulerError> {
        self.threads
            .get(&thread)
            .copied()
            .ok_or(SchedulerError::UnknownThread { thread })
    }

    pub fn waiter_count(&self, mem: MemSlot, addr: Addr) -> usize {
        self.wait_queues.get(&(mem, addr)).map_or(0, VecDeque::len)
    }

    pub fn spawn(&mut self) -> Result<ThreadId, SchedulerError> {
        self.ensure_running()?;
        let active = self
            .threads
            .values()
            .filter(|state| **state != ThreadState::Exited)
            .count();
        if active >= self.thread_cap as usize {
            return Err(SchedulerError::ThreadCap {
                cap: self.thread_cap,
            });
        }
        if self.next_tid > u32::MAX as u64 {
            return Err(SchedulerError::ThreadIdExhausted);
        }

        let thread = ThreadId::new(self.next_tid as u32);
        self.next_tid += 1;
        self.threads.insert(thread, ThreadState::Runnable);
        self.trace.push(TraceEvent::Spawn { thread });
        Ok(thread)
    }

    pub fn next_runnable(&mut self) -> Result<Option<ThreadId>, SchedulerError> {
        self.ensure_running()?;
        if let Some(current) = self.current {
            if self
                .threads
                .get(&current)
                .is_some_and(|state| state.is_schedulable())
            {
                return Ok(Some(current));
            }
            self.current = None;
        }

        let next = self
            .threads
            .iter()
            .find(|(thread, state)| {
                self.last_run.map_or(true, |last| **thread > last) && state.is_schedulable()
            })
            .or_else(|| {
                self.threads
                    .iter()
                    .find(|(_, state)| state.is_schedulable())
            })
            .map(|(thread, _)| *thread);

        if let Some(next) = next {
            self.trace.push(TraceEvent::Switch {
                from: self.last_run,
                to: next,
            });
            self.current = Some(next);
        }
        Ok(next)
    }

    pub fn on_quantum_end(&mut self, thread: ThreadId) -> Result<(), SchedulerError> {
        self.ensure_running()?;
        let state = self.thread_state(thread)?;
        if !state.is_schedulable() {
            return Err(self.illegal(thread, state, SchedulerOperation::QuantumEnd));
        }
        if self.current != Some(thread) {
            return Err(SchedulerError::NotCurrentThread {
                expected: self.current,
                actual: thread,
            });
        }
        self.current = None;
        self.last_run = Some(thread);
        Ok(())
    }

    pub fn park_wait(
        &mut self,
        thread: ThreadId,
        mem: MemSlot,
        addr: Addr,
        timeout_rounds: Option<u64>,
    ) -> Result<(), SchedulerError> {
        self.ensure_running()?;
        let state = self.thread_state(thread)?;
        if state != ThreadState::Runnable {
            return Err(self.illegal(thread, state, SchedulerOperation::ParkWait));
        }

        if timeout_rounds == Some(0) {
            self.threads
                .insert(thread, ThreadState::Woken { result: 2 });
            self.trace.push(TraceEvent::ParkWait {
                thread,
                mem,
                addr,
                deadline: Some(self.current_round),
                seq: None,
            });
            self.trace.push(TraceEvent::Timeout {
                thread,
                deadline: self.current_round,
                seq: None,
            });
            return Ok(());
        }

        let deadline = timeout_rounds
            .map(|rounds| {
                self.current_round
                    .checked_add(rounds)
                    .ok_or(SchedulerError::RoundOverflow {
                        base: self.current_round,
                        timeout_rounds: rounds,
                    })
            })
            .transpose()?;
        let seq = self.next_wait_seq;
        self.next_wait_seq = self
            .next_wait_seq
            .checked_add(1)
            .ok_or(SchedulerError::WaitSequenceExhausted)?;

        self.threads.insert(
            thread,
            ThreadState::ParkedWait {
                mem,
                addr,
                deadline,
                seq,
            },
        );
        self.wait_queues
            .entry((mem, addr))
            .or_default()
            .push_back(Waiter { thread, seq });
        self.detach_current(thread);
        self.trace.push(TraceEvent::ParkWait {
            thread,
            mem,
            addr,
            deadline,
            seq: Some(seq),
        });
        Ok(())
    }

    pub fn notify(&mut self, mem: MemSlot, addr: Addr, count: u32) -> Result<u32, SchedulerError> {
        self.ensure_running()?;
        let key = (mem, addr);
        let mut woken = 0u32;
        while woken < count {
            let waiter = self.wait_queues.get_mut(&key).and_then(VecDeque::pop_front);
            let Some(waiter) = waiter else {
                break;
            };
            let still_waiting = matches!(
                self.threads.get(&waiter.thread),
                Some(ThreadState::ParkedWait {
                    mem: queued_mem,
                    addr: queued_addr,
                    seq,
                    ..
                }) if *queued_mem == mem && *queued_addr == addr && *seq == waiter.seq
            );
            if !still_waiting {
                continue;
            }
            self.threads
                .insert(waiter.thread, ThreadState::Woken { result: 0 });
            self.trace.push(TraceEvent::Wake {
                thread: waiter.thread,
                result: 0,
                reason: WakeReason::Notify,
            });
            woken += 1;
        }
        if self.wait_queues.get(&key).is_some_and(VecDeque::is_empty) {
            self.wait_queues.remove(&key);
        }
        Ok(woken)
    }

    pub fn begin_round(&mut self, round: Runde) -> Result<(), SchedulerError> {
        self.ensure_running()?;
        if round < self.current_round {
            return Err(SchedulerError::RoundWentBackwards {
                current: self.current_round,
                requested: round,
            });
        }
        self.current_round = round;

        let mut expired = Vec::new();
        for (thread, state) in &self.threads {
            if let ThreadState::ParkedWait {
                mem,
                addr,
                deadline: Some(deadline),
                seq,
            } = *state
            {
                if deadline <= round {
                    expired.push(Expiry {
                        thread: *thread,
                        mem,
                        addr,
                        deadline,
                        seq,
                    });
                }
            }
        }
        expired.sort_unstable();

        for expiry in expired {
            let still_waiting = matches!(
                self.threads.get(&expiry.thread),
                Some(ThreadState::ParkedWait {
                    mem,
                    addr,
                    deadline: Some(deadline),
                    seq,
                }) if *mem == expiry.mem
                    && *addr == expiry.addr
                    && *deadline == expiry.deadline
                    && *seq == expiry.seq
            );
            if !still_waiting {
                continue;
            }
            self.remove_waiter(expiry.thread, expiry.mem, expiry.addr, expiry.seq);
            self.threads
                .insert(expiry.thread, ThreadState::Woken { result: 2 });
            self.trace.push(TraceEvent::Timeout {
                thread: expiry.thread,
                deadline: expiry.deadline,
                seq: Some(expiry.seq),
            });
        }
        Ok(())
    }

    pub fn park_host(&mut self, thread: ThreadId) -> Result<(), SchedulerError> {
        self.ensure_running()?;
        let state = self.thread_state(thread)?;
        if state != ThreadState::Runnable {
            return Err(self.illegal(thread, state, SchedulerOperation::ParkHost));
        }
        self.threads.insert(thread, ThreadState::ParkedHost);
        self.detach_current(thread);
        self.trace.push(TraceEvent::ParkHost { thread });
        Ok(())
    }

    pub fn wake_host(&mut self, thread: ThreadId, result: u32) -> Result<(), SchedulerError> {
        self.ensure_running()?;
        let state = self.thread_state(thread)?;
        if state != ThreadState::ParkedHost {
            return Err(self.illegal(thread, state, SchedulerOperation::WakeHost));
        }
        self.threads.insert(thread, ThreadState::Woken { result });
        self.trace.push(TraceEvent::Wake {
            thread,
            result,
            reason: WakeReason::Host,
        });
        Ok(())
    }

    pub fn take_wake_result(&mut self, thread: ThreadId) -> Result<u32, SchedulerError> {
        self.ensure_running()?;
        let state = self.thread_state(thread)?;
        let ThreadState::Woken { result } = state else {
            return Err(self.illegal(thread, state, SchedulerOperation::TakeWakeResult));
        };
        self.threads.insert(thread, ThreadState::Runnable);
        Ok(result)
    }

    pub fn thread_exit(&mut self, thread: ThreadId) -> Result<(), SchedulerError> {
        self.teardown_thread(
            thread,
            ThreadExitReason::Returned,
            SchedulerOperation::ThreadExit,
        )
    }

    pub fn kill_thread(&mut self, thread: ThreadId) -> Result<(), SchedulerError> {
        self.teardown_thread(
            thread,
            ThreadExitReason::Killed,
            SchedulerOperation::KillThread,
        )
    }

    pub fn proc_exit(&mut self, code: u32) -> Result<(), SchedulerError> {
        match self.job_state {
            JobState::Running => {}
            JobState::JobExited { code } => {
                return Err(SchedulerError::JobAlreadyExited { code });
            }
            JobState::JobDeadlocked => return Err(SchedulerError::JobDeadlocked),
        }

        self.job_state = JobState::JobExited { code };
        self.current = None;
        self.wait_queues.clear();
        for state in self.threads.values_mut() {
            *state = ThreadState::Exited;
        }
        self.trace.push(TraceEvent::JobExit { code });
        Ok(())
    }

    pub fn check_deadlock(&mut self, has_open_host_op: bool) -> Result<(), SchedulerError> {
        self.ensure_running()?;
        if has_open_host_op {
            return Ok(());
        }

        let mut non_exited = 0usize;
        let mut all_parked = true;
        let mut has_finite_deadline = false;
        for state in self.threads.values().copied() {
            match state {
                ThreadState::Exited => {}
                ThreadState::ParkedWait { deadline, .. } => {
                    non_exited += 1;
                    has_finite_deadline |= deadline.is_some();
                }
                ThreadState::ParkedHost => non_exited += 1,
                ThreadState::Runnable | ThreadState::Woken { .. } => {
                    non_exited += 1;
                    all_parked = false;
                }
            }
        }

        if non_exited != 0 && all_parked && !has_finite_deadline {
            self.job_state = JobState::JobDeadlocked;
            self.current = None;
            self.trace.push(TraceEvent::Deadlock);
            return Err(SchedulerError::JobDeadlocked);
        }
        Ok(())
    }

    fn ensure_running(&self) -> Result<(), SchedulerError> {
        match self.job_state {
            JobState::Running => Ok(()),
            JobState::JobExited { code } => Err(SchedulerError::JobExited { code }),
            JobState::JobDeadlocked => Err(SchedulerError::JobDeadlocked),
        }
    }

    fn illegal(
        &self,
        thread: ThreadId,
        state: ThreadState,
        operation: SchedulerOperation,
    ) -> SchedulerError {
        SchedulerError::IllegalTransition {
            thread,
            from: state.kind(),
            operation,
        }
    }

    fn detach_current(&mut self, thread: ThreadId) {
        if self.current == Some(thread) {
            self.current = None;
            self.last_run = Some(thread);
        }
    }

    fn remove_waiter(&mut self, thread: ThreadId, mem: MemSlot, addr: Addr, seq: u64) {
        let key = (mem, addr);
        if let Some(queue) = self.wait_queues.get_mut(&key) {
            if let Some(index) = queue
                .iter()
                .position(|waiter| waiter.thread == thread && waiter.seq == seq)
            {
                queue.remove(index);
            }
            if queue.is_empty() {
                self.wait_queues.remove(&key);
            }
        }
    }

    fn teardown_thread(
        &mut self,
        thread: ThreadId,
        reason: ThreadExitReason,
        operation: SchedulerOperation,
    ) -> Result<(), SchedulerError> {
        self.ensure_running()?;
        let state = self.thread_state(thread)?;
        if state == ThreadState::Exited {
            return Err(self.illegal(thread, state, operation));
        }
        self.remove_all_waiters(thread);
        self.threads.insert(thread, ThreadState::Exited);
        self.detach_current(thread);
        self.trace.push(TraceEvent::ThreadExit { thread, reason });
        Ok(())
    }

    fn remove_all_waiters(&mut self, thread: ThreadId) {
        self.wait_queues.retain(|_, queue| {
            queue.retain(|waiter| waiter.thread != thread);
            !queue.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::vec;

    fn tid(value: u32) -> ThreadId {
        ThreadId::new(value)
    }

    fn mem(value: u32) -> MemSlot {
        MemSlot::new(value)
    }

    fn addr(value: u64) -> Addr {
        Addr::new(value)
    }

    fn spawn_three(scheduler: &mut JobThreadScheduler) -> [ThreadId; 3] {
        [
            scheduler.spawn().unwrap(),
            scheduler.spawn().unwrap(),
            scheduler.spawn().unwrap(),
        ]
    }

    #[test]
    fn tids_are_ascending_and_default_cap_is_48() {
        let mut scheduler = JobThreadScheduler::new();
        assert_eq!(scheduler.thread_cap(), 48);
        for expected in 0..48 {
            assert_eq!(scheduler.spawn(), Ok(tid(expected)));
        }
        assert_eq!(
            scheduler.spawn(),
            Err(SchedulerError::ThreadCap { cap: 48 })
        );

        let mut capped = JobThreadScheduler::with_thread_cap(2);
        assert_eq!(capped.spawn(), Ok(tid(0)));
        assert_eq!(capped.spawn(), Ok(tid(1)));
        assert_eq!(capped.spawn(), Err(SchedulerError::ThreadCap { cap: 2 }));
    }

    #[test]
    fn round_robin_switch_trace_is_fair_only_at_quantum_boundaries() {
        let mut scheduler = JobThreadScheduler::new();
        spawn_three(&mut scheduler);
        let mut observed = Vec::new();

        for _ in 0..6 {
            let selected = scheduler.next_runnable().unwrap().unwrap();
            assert_eq!(scheduler.next_runnable(), Ok(Some(selected)));
            observed.push(selected);
            scheduler.on_quantum_end(selected).unwrap();
        }
        assert_eq!(
            observed,
            vec![tid(0), tid(1), tid(2), tid(0), tid(1), tid(2)]
        );

        let traced = scheduler
            .trace()
            .iter()
            .filter_map(|event| match event {
                TraceEvent::Switch { to, .. } => Some(*to),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(traced, observed);
    }

    #[test]
    fn notify_wakes_fifo_and_empty_notify_returns_zero() {
        let mut scheduler = JobThreadScheduler::new();
        let threads = spawn_three(&mut scheduler);
        for thread in threads {
            scheduler.park_wait(thread, mem(7), addr(9), None).unwrap();
        }

        assert_eq!(scheduler.notify(mem(7), addr(9), 2), Ok(2));
        assert_eq!(
            scheduler.thread_state(threads[0]),
            Ok(ThreadState::Woken { result: 0 })
        );
        assert_eq!(
            scheduler.thread_state(threads[1]),
            Ok(ThreadState::Woken { result: 0 })
        );
        assert!(matches!(
            scheduler.thread_state(threads[2]),
            Ok(ThreadState::ParkedWait { .. })
        ));
        assert_eq!(scheduler.notify(mem(7), addr(9), 5), Ok(1));
        assert_eq!(scheduler.notify(mem(7), addr(9), 1), Ok(0));
        assert_eq!(scheduler.notify(mem(88), addr(99), 1), Ok(0));
    }

    #[test]
    fn wait32_and_wait64_share_mem_and_address_queue() {
        let mut scheduler = JobThreadScheduler::new();
        let [wait32, wait64, _] = spawn_three(&mut scheduler);

        scheduler.park_wait(wait32, mem(3), addr(64), None).unwrap();
        scheduler.park_wait(wait64, mem(3), addr(64), None).unwrap();
        assert_eq!(scheduler.waiter_count(mem(3), addr(64)), 2);
        assert_eq!(scheduler.notify(mem(3), addr(64), 2), Ok(2));
        assert_eq!(
            scheduler.thread_state(wait32),
            Ok(ThreadState::Woken { result: 0 })
        );
        assert_eq!(
            scheduler.thread_state(wait64),
            Ok(ThreadState::Woken { result: 0 })
        );
    }

    #[test]
    fn zero_timeout_is_immediately_schedulable_without_queueing() {
        let mut scheduler = JobThreadScheduler::new();
        let thread = scheduler.spawn().unwrap();
        scheduler
            .park_wait(thread, mem(1), addr(2), Some(0))
            .unwrap();

        assert_eq!(
            scheduler.thread_state(thread),
            Ok(ThreadState::Woken { result: 2 })
        );
        assert_eq!(scheduler.waiter_count(mem(1), addr(2)), 0);
        assert_eq!(scheduler.next_runnable(), Ok(Some(thread)));
        assert_eq!(scheduler.take_wake_result(thread), Ok(2));
        assert_eq!(scheduler.thread_state(thread), Ok(ThreadState::Runnable));
    }

    #[test]
    fn finite_timeouts_expire_by_deadline_then_park_sequence() {
        let mut scheduler = JobThreadScheduler::new();
        let [first, second, third] = spawn_three(&mut scheduler);
        scheduler.begin_round(Runde::new(10)).unwrap();
        scheduler
            .park_wait(first, mem(1), addr(1), Some(2))
            .unwrap();
        scheduler
            .park_wait(second, mem(1), addr(2), Some(1))
            .unwrap();
        scheduler
            .park_wait(third, mem(1), addr(3), Some(1))
            .unwrap();

        scheduler.begin_round(Runde::new(12)).unwrap();
        let timed_out = scheduler
            .trace()
            .iter()
            .filter_map(|event| match event {
                TraceEvent::Timeout {
                    thread,
                    seq: Some(_),
                    ..
                } => Some(*thread),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(timed_out, vec![second, third, first]);

        scheduler.begin_round(Runde::new(12)).unwrap();
        assert_eq!(
            scheduler
                .trace()
                .iter()
                .filter(|event| matches!(event, TraceEvent::Timeout { seq: Some(_), .. }))
                .count(),
            3
        );
    }

    #[test]
    fn notify_before_next_round_beats_timeout_sweep() {
        let mut scheduler = JobThreadScheduler::new();
        let thread = scheduler.spawn().unwrap();
        scheduler.begin_round(Runde::new(5)).unwrap();
        scheduler
            .park_wait(thread, mem(1), addr(5), Some(1))
            .unwrap();

        assert_eq!(scheduler.notify(mem(1), addr(5), 1), Ok(1));
        scheduler.begin_round(Runde::new(6)).unwrap();
        assert_eq!(
            scheduler.thread_state(thread),
            Ok(ThreadState::Woken { result: 0 })
        );
        assert!(!scheduler.trace().iter().any(|event| matches!(
            event,
            TraceEvent::Timeout {
                thread: timed_out,
                seq: Some(_),
                ..
            } if *timed_out == thread
        )));
    }

    #[test]
    fn infinite_wait_survives_arbitrarily_later_rounds() {
        let mut scheduler = JobThreadScheduler::new();
        let thread = scheduler.spawn().unwrap();
        scheduler.park_wait(thread, mem(2), addr(8), None).unwrap();
        scheduler.begin_round(Runde::new(1)).unwrap();
        scheduler.begin_round(Runde::new(1_000_000)).unwrap();
        scheduler.begin_round(Runde::new(u64::MAX)).unwrap();
        assert!(matches!(
            scheduler.thread_state(thread),
            Ok(ThreadState::ParkedWait { deadline: None, .. })
        ));
    }

    #[test]
    fn proc_exit_first_wins_and_freezes_all_threads() {
        let mut scheduler = JobThreadScheduler::new();
        let [first, second, _] = spawn_three(&mut scheduler);
        scheduler.park_wait(second, mem(1), addr(1), None).unwrap();
        assert_eq!(scheduler.next_runnable(), Ok(Some(first)));

        assert_eq!(scheduler.proc_exit(17), Ok(()));
        assert_eq!(scheduler.job_state(), JobState::JobExited { code: 17 });
        assert_eq!(
            scheduler.proc_exit(99),
            Err(SchedulerError::JobAlreadyExited { code: 17 })
        );
        assert_eq!(
            scheduler.next_runnable(),
            Err(SchedulerError::JobExited { code: 17 })
        );
        assert_eq!(scheduler.waiter_count(mem(1), addr(1)), 0);
        assert!(scheduler
            .threads
            .values()
            .all(|state| *state == ThreadState::Exited));
    }

    #[test]
    fn killed_waiter_is_removed_and_not_counted_by_notify() {
        let mut scheduler = JobThreadScheduler::new();
        let [killed, survivor, _] = spawn_three(&mut scheduler);
        scheduler.park_wait(killed, mem(1), addr(7), None).unwrap();
        scheduler
            .park_wait(survivor, mem(1), addr(7), None)
            .unwrap();

        assert_eq!(scheduler.kill_thread(killed), Ok(()));
        assert_eq!(scheduler.waiter_count(mem(1), addr(7)), 1);
        assert_eq!(scheduler.notify(mem(1), addr(7), 5), Ok(1));
        assert_eq!(scheduler.thread_state(killed), Ok(ThreadState::Exited));
        assert_eq!(
            scheduler.thread_state(survivor),
            Ok(ThreadState::Woken { result: 0 })
        );
    }

    #[test]
    fn deadlock_requires_all_parked_no_deadline_and_no_open_host_op() {
        let mut scheduler = JobThreadScheduler::new();
        let [atomic, host, _] = spawn_three(&mut scheduler);
        scheduler.kill_thread(tid(2)).unwrap();
        scheduler.park_wait(atomic, mem(1), addr(1), None).unwrap();
        scheduler.park_host(host).unwrap();

        assert_eq!(scheduler.check_deadlock(true), Ok(()));
        assert_eq!(scheduler.job_state(), JobState::Running);
        assert_eq!(
            scheduler.check_deadlock(false),
            Err(SchedulerError::JobDeadlocked)
        );
        assert_eq!(scheduler.job_state(), JobState::JobDeadlocked);
        assert_eq!(
            scheduler.next_runnable(),
            Err(SchedulerError::JobDeadlocked)
        );
    }

    #[test]
    fn finite_deadline_and_runnable_thread_each_prevent_deadlock() {
        let mut finite = JobThreadScheduler::new();
        let thread = finite.spawn().unwrap();
        finite.park_wait(thread, mem(1), addr(1), Some(1)).unwrap();
        assert_eq!(finite.check_deadlock(false), Ok(()));

        let mut runnable = JobThreadScheduler::new();
        runnable.spawn().unwrap();
        assert_eq!(runnable.check_deadlock(false), Ok(()));
    }

    #[test]
    fn illegal_transitions_are_typed_and_never_panic() {
        let mut scheduler = JobThreadScheduler::new();
        let thread = scheduler.spawn().unwrap();
        scheduler.park_wait(thread, mem(1), addr(1), None).unwrap();
        assert_eq!(
            scheduler.park_wait(thread, mem(1), addr(1), None),
            Err(SchedulerError::IllegalTransition {
                thread,
                from: ThreadStateKind::ParkedWait,
                operation: SchedulerOperation::ParkWait,
            })
        );
        assert_eq!(
            scheduler.on_quantum_end(thread),
            Err(SchedulerError::IllegalTransition {
                thread,
                from: ThreadStateKind::ParkedWait,
                operation: SchedulerOperation::QuantumEnd,
            })
        );
    }

    fn replay_trace() -> alloc::string::String {
        let mut scheduler = JobThreadScheduler::new();
        let [first, second, third] = spawn_three(&mut scheduler);
        let selected = scheduler.next_runnable().unwrap().unwrap();
        scheduler.on_quantum_end(selected).unwrap();
        scheduler.begin_round(Runde::new(4)).unwrap();
        scheduler
            .park_wait(first, mem(2), addr(10), Some(2))
            .unwrap();
        scheduler.park_wait(second, mem(2), addr(10), None).unwrap();
        scheduler.park_host(third).unwrap();
        scheduler.notify(mem(2), addr(10), 1).unwrap();
        scheduler.wake_host(third, 73).unwrap();
        scheduler.begin_round(Runde::new(6)).unwrap();
        scheduler.thread_exit(second).unwrap();
        format!("{:?}", scheduler.trace())
    }

    #[test]
    fn identical_event_replay_has_byte_identical_debug_trace() {
        let first = replay_trace();
        let second = replay_trace();
        assert_eq!(first.as_bytes(), second.as_bytes());
    }
}
