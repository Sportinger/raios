//! Manifest-bound process inputs and typed non-filesystem host effects.

use alloc::vec::Vec;

use crate::determinism::{clock_time_get, DeterministicRandom};
use crate::{ClockId, Errno, EPOCH_2000_NS};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobContext {
    args: Vec<Vec<u8>>,
    environment: Vec<(Vec<u8>, Vec<u8>)>,
    job_manifest_sha256: [u8; 32],
}

impl JobContext {
    /// Constructs the complete process input without consulting a host environment.
    pub fn new(
        args: Vec<Vec<u8>>,
        environment: Vec<(Vec<u8>, Vec<u8>)>,
        job_manifest_sha256: [u8; 32],
    ) -> Result<Self, Errno> {
        if args.iter().any(|argument| argument.contains(&0))
            || environment.iter().any(|(key, value)| {
                key.is_empty() || key.contains(&0) || key.contains(&b'=') || value.contains(&0)
            })
        {
            return Err(Errno::Inval);
        }
        Ok(Self {
            args,
            environment,
            job_manifest_sha256,
        })
    }

    pub fn args(&self) -> &[Vec<u8>] {
        &self.args
    }

    pub fn environment(&self) -> &[(Vec<u8>, Vec<u8>)] {
        &self.environment
    }

    pub const fn job_manifest_sha256(&self) -> [u8; 32] {
        self.job_manifest_sha256
    }

    /// Looks up only manifest-declared environment. There is no host fallback.
    pub fn environment_value(&self, key: &[u8]) -> Result<&[u8], Errno> {
        self.environment
            .iter()
            .find(|(candidate, _)| candidate.as_slice() == key)
            .map(|(_, value)| value.as_slice())
            .ok_or(Errno::Notcapable)
    }

    fn args_sizes(&self) -> Result<StringListSizes, Errno> {
        checked_list_sizes(
            self.args.len(),
            self.args.iter().map(|value| value.len().saturating_add(1)),
        )
    }

    fn environment_sizes(&self) -> Result<StringListSizes, Errno> {
        checked_list_sizes(
            self.environment.len(),
            self.environment.iter().map(|(key, value)| {
                key.len()
                    .checked_add(1)
                    .and_then(|size| size.checked_add(value.len()))
                    .and_then(|size| size.checked_add(1))
                    .unwrap_or(usize::MAX)
            }),
        )
    }

    fn serialize_args(&self) -> Result<SerializedStrings, Errno> {
        serialize_strings(
            self.args_sizes()?,
            self.args.iter().map(|value| {
                let mut serialized = Vec::with_capacity(value.len() + 1);
                serialized.extend_from_slice(value);
                serialized.push(0);
                serialized
            }),
        )
    }

    fn serialize_environment(&self) -> Result<SerializedStrings, Errno> {
        serialize_strings(
            self.environment_sizes()?,
            self.environment.iter().map(|(key, value)| {
                let mut serialized = Vec::with_capacity(key.len() + value.len() + 2);
                serialized.extend_from_slice(key);
                serialized.push(b'=');
                serialized.extend_from_slice(value);
                serialized.push(0);
                serialized
            }),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StringListSizes {
    pub count: u32,
    pub buffer_size: u32,
}

/// Preview1 string data. `pointer_offsets` are relative to `buffer`; kernel
/// glue adds the checked guest buffer base when writing the pointer array.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializedStrings {
    pub pointer_offsets: Vec<u32>,
    pub buffer: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessError {
    Wasi(Errno),
    Exited(u32),
}

impl From<Errno> for ProcessError {
    fn from(value: Errno) -> Self {
        Self::Wasi(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostEffect {
    Exit(u32),
    Yield,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockTimeout {
    Relative,
    Absolute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockSubscription {
    pub userdata: u64,
    pub clock_id: u32,
    pub timeout_ns: u64,
    pub timeout: ClockTimeout,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Subscription {
    Clock(ClockSubscription),
    FdRead { userdata: u64, fd: u32 },
    FdWrite { userdata: u64, fd: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PollEvent {
    pub userdata: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PollResult {
    pub events: Vec<PollEvent>,
    /// Earliest logical fuel value at which a non-due timer can become due.
    /// It is absent when this call already has events and must return immediately.
    /// An adapter must suspend a zero-event result until this fuel value; it
    /// must not expose that intermediate result as a successful preview1 call.
    pub next_wake_fuel: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessContext {
    job: JobContext,
    random: DeterministicRandom,
    exited: Option<u32>,
}

impl ProcessContext {
    pub fn new(job: JobContext) -> Self {
        let random = DeterministicRandom::from_manifest_sha256(job.job_manifest_sha256());
        Self {
            job,
            random,
            exited: None,
        }
    }

    pub const fn exit_code(&self) -> Option<u32> {
        self.exited
    }

    pub fn args_sizes_get(&self) -> Result<StringListSizes, ProcessError> {
        self.ensure_active()?;
        self.job.args_sizes().map_err(Into::into)
    }

    pub fn args_get(&self) -> Result<SerializedStrings, ProcessError> {
        self.ensure_active()?;
        self.job.serialize_args().map_err(Into::into)
    }

    pub fn environ_sizes_get(&self) -> Result<StringListSizes, ProcessError> {
        self.ensure_active()?;
        self.job.environment_sizes().map_err(Into::into)
    }

    pub fn environ_get(&self) -> Result<SerializedStrings, ProcessError> {
        self.ensure_active()?;
        self.job.serialize_environment().map_err(Into::into)
    }

    pub fn clock_time_get(&self, clock_id: u32, fuel_used: u64) -> Result<u64, ProcessError> {
        self.ensure_active()?;
        clock_time_get(clock_id, fuel_used).map_err(Into::into)
    }

    pub fn random_get(&mut self, len: usize) -> Result<Vec<u8>, ProcessError> {
        self.ensure_active()?;
        self.random.random_get(len).map_err(Into::into)
    }

    pub fn sched_yield(&self) -> Result<HostEffect, ProcessError> {
        self.ensure_active()?;
        Ok(HostEffect::Yield)
    }

    /// Evaluates only clock subscriptions. Any fd subscription rejects the
    /// complete call with `INVAL`, without returning partial clock events.
    pub fn poll_oneoff(
        &self,
        fuel_used: u64,
        subscriptions: &[Subscription],
    ) -> Result<PollResult, ProcessError> {
        self.ensure_active()?;
        if subscriptions.is_empty()
            || subscriptions
                .iter()
                .any(|subscription| !matches!(subscription, Subscription::Clock(_)))
        {
            return Err(Errno::Inval.into());
        }

        let mut events = Vec::new();
        let mut next_wake_fuel: Option<u64> = None;
        for subscription in subscriptions {
            let Subscription::Clock(clock) = subscription else {
                unreachable!();
            };
            let clock_id = ClockId::try_from(clock.clock_id)?;
            let deadline_fuel = match clock.timeout {
                ClockTimeout::Relative => fuel_used
                    .checked_add(clock.timeout_ns)
                    .ok_or(Errno::Inval)?,
                ClockTimeout::Absolute => match clock_id {
                    ClockId::Monotonic => clock.timeout_ns,
                    ClockId::Realtime => clock.timeout_ns.saturating_sub(EPOCH_2000_NS),
                },
            };
            if deadline_fuel <= fuel_used {
                events.push(PollEvent {
                    userdata: clock.userdata,
                });
            } else {
                next_wake_fuel = Some(match next_wake_fuel {
                    Some(current) => current.min(deadline_fuel),
                    None => deadline_fuel,
                });
            }
        }
        if !events.is_empty() {
            next_wake_fuel = None;
        }
        Ok(PollResult {
            events,
            next_wake_fuel,
        })
    }

    /// Records an exit as a host effect. It never terminates or panics in the host.
    pub fn proc_exit(&mut self, code: u32) -> Result<HostEffect, ProcessError> {
        self.ensure_active()?;
        self.exited = Some(code);
        Ok(HostEffect::Exit(code))
    }

    fn ensure_active(&self) -> Result<(), ProcessError> {
        match self.exited {
            Some(code) => Err(ProcessError::Exited(code)),
            None => Ok(()),
        }
    }
}

fn checked_list_sizes(
    count: usize,
    serialized_lengths: impl IntoIterator<Item = usize>,
) -> Result<StringListSizes, Errno> {
    let count = u32::try_from(count).map_err(|_| Errno::Inval)?;
    let mut buffer_size = 0usize;
    for length in serialized_lengths {
        buffer_size = buffer_size.checked_add(length).ok_or(Errno::Inval)?;
    }
    Ok(StringListSizes {
        count,
        buffer_size: u32::try_from(buffer_size).map_err(|_| Errno::Inval)?,
    })
}

fn serialize_strings(
    sizes: StringListSizes,
    values: impl IntoIterator<Item = Vec<u8>>,
) -> Result<SerializedStrings, Errno> {
    let mut pointer_offsets = Vec::with_capacity(sizes.count as usize);
    let mut buffer = Vec::with_capacity(sizes.buffer_size as usize);
    for value in values {
        pointer_offsets.push(u32::try_from(buffer.len()).map_err(|_| Errno::Inval)?);
        buffer.extend_from_slice(&value);
    }
    if pointer_offsets.len() != sizes.count as usize || buffer.len() != sizes.buffer_size as usize {
        return Err(Errno::Inval);
    }
    Ok(SerializedStrings {
        pointer_offsets,
        buffer,
    })
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::MAX_RANDOM_GET;

    fn job() -> JobContext {
        JobContext::new(
            vec![b"rustc".to_vec(), b"--version".to_vec()],
            vec![
                (b"LANG".to_vec(), b"C".to_vec()),
                (b"EMPTY".to_vec(), Vec::new()),
            ],
            [0x5a; 32],
        )
        .unwrap()
    }

    #[test]
    fn args_and_environment_have_exact_preview1_layout() {
        let process = ProcessContext::new(job());
        assert_eq!(
            process.args_sizes_get(),
            Ok(StringListSizes {
                count: 2,
                buffer_size: 16,
            })
        );
        assert_eq!(
            process.args_get(),
            Ok(SerializedStrings {
                pointer_offsets: vec![0, 6],
                buffer: b"rustc\0--version\0".to_vec(),
            })
        );
        assert_eq!(
            process.environ_sizes_get(),
            Ok(StringListSizes {
                count: 2,
                buffer_size: 14,
            })
        );
        assert_eq!(
            process.environ_get(),
            Ok(SerializedStrings {
                pointer_offsets: vec![0, 7],
                buffer: b"LANG=C\0EMPTY=\0".to_vec(),
            })
        );
        assert_eq!(process.job.environment_value(b"LANG"), Ok(b"C".as_slice()));
        assert_eq!(
            process.job.environment_value(b"HOST_ONLY"),
            Err(Errno::Notcapable)
        );
    }

    #[test]
    fn empty_lists_and_checked_size_overflow_are_exact() {
        let empty = JobContext::new(Vec::new(), Vec::new(), [0; 32]).unwrap();
        let process = ProcessContext::new(empty);
        assert_eq!(
            process.args_sizes_get().unwrap(),
            StringListSizes {
                count: 0,
                buffer_size: 0
            }
        );
        assert_eq!(
            process.args_get().unwrap(),
            SerializedStrings {
                pointer_offsets: Vec::new(),
                buffer: Vec::new()
            }
        );
        assert_eq!(
            process.environ_sizes_get().unwrap(),
            StringListSizes {
                count: 0,
                buffer_size: 0
            }
        );
        assert_eq!(
            process.environ_get().unwrap(),
            SerializedStrings {
                pointer_offsets: Vec::new(),
                buffer: Vec::new()
            }
        );

        assert_eq!(
            checked_list_sizes(1, [u32::MAX as usize, 1]),
            Err(Errno::Inval)
        );
        assert_eq!(checked_list_sizes(2, [usize::MAX, 1]), Err(Errno::Inval));
    }

    #[test]
    fn embedded_terminators_and_invalid_environment_keys_are_rejected() {
        assert_eq!(
            JobContext::new(vec![b"a\0b".to_vec()], Vec::new(), [0; 32]),
            Err(Errno::Inval)
        );
        assert_eq!(
            JobContext::new(Vec::new(), vec![(b"A=B".to_vec(), b"x".to_vec())], [0; 32]),
            Err(Errno::Inval)
        );
    }

    #[test]
    fn identical_contexts_and_fuel_trace_byte_identically() {
        #[derive(Debug, Eq, PartialEq)]
        struct Trace {
            args: SerializedStrings,
            environment: SerializedStrings,
            clocks: [u64; 4],
            random: Vec<u8>,
        }

        fn trace(mut process: ProcessContext) -> Trace {
            Trace {
                args: process.args_get().unwrap(),
                environment: process.environ_get().unwrap(),
                clocks: [
                    process.clock_time_get(1, 5).unwrap(),
                    process.clock_time_get(0, 5).unwrap(),
                    process.clock_time_get(1, 987).unwrap(),
                    process.clock_time_get(0, 987).unwrap(),
                ],
                random: process.random_get(37).unwrap(),
            }
        }

        let first = ProcessContext::new(job());
        let second = ProcessContext::new(job());
        assert_eq!(trace(first), trace(second));
    }

    #[test]
    fn clock_and_random_errors_are_typed_and_random_is_transactional() {
        let mut process = ProcessContext::new(job());
        assert_eq!(
            process.clock_time_get(99, 0),
            Err(ProcessError::Wasi(Errno::Inval))
        );
        assert_eq!(
            process.clock_time_get(0, u64::MAX),
            Err(ProcessError::Wasi(Errno::Inval))
        );

        let mut expected = ProcessContext::new(job());
        assert_eq!(
            process.random_get(MAX_RANDOM_GET + 1),
            Err(ProcessError::Wasi(Errno::Inval))
        );
        assert_eq!(process.random_get(25), expected.random_get(25));
    }

    #[test]
    fn first_proc_exit_wins_and_every_later_guest_call_fails_closed() {
        let mut process = ProcessContext::new(job());
        assert_eq!(
            process.proc_exit(0xfeed_beef),
            Ok(HostEffect::Exit(0xfeed_beef))
        );
        assert_eq!(process.exit_code(), Some(0xfeed_beef));
        assert_eq!(process.proc_exit(7), Err(ProcessError::Exited(0xfeed_beef)));
        assert_eq!(process.args_get(), Err(ProcessError::Exited(0xfeed_beef)));
        assert_eq!(
            process.args_sizes_get(),
            Err(ProcessError::Exited(0xfeed_beef))
        );
        assert_eq!(
            process.environ_get(),
            Err(ProcessError::Exited(0xfeed_beef))
        );
        assert_eq!(
            process.environ_sizes_get(),
            Err(ProcessError::Exited(0xfeed_beef))
        );
        assert_eq!(
            process.clock_time_get(1, 0),
            Err(ProcessError::Exited(0xfeed_beef))
        );
        assert_eq!(
            process.random_get(1),
            Err(ProcessError::Exited(0xfeed_beef))
        );
        assert_eq!(
            process.sched_yield(),
            Err(ProcessError::Exited(0xfeed_beef))
        );
        assert_eq!(
            process.poll_oneoff(0, &[clock(1, 0, ClockTimeout::Relative)]),
            Err(ProcessError::Exited(0xfeed_beef))
        );
    }

    #[test]
    fn sched_yield_is_a_typed_effect_without_exiting() {
        let process = ProcessContext::new(job());
        assert_eq!(process.sched_yield(), Ok(HostEffect::Yield));
        assert_eq!(process.exit_code(), None);
        assert!(process.args_get().is_ok());
    }

    fn clock(userdata: u64, timeout_ns: u64, timeout: ClockTimeout) -> Subscription {
        Subscription::Clock(ClockSubscription {
            userdata,
            clock_id: ClockId::Monotonic as u32,
            timeout_ns,
            timeout,
        })
    }

    #[test]
    fn poll_oneoff_handles_relative_and_absolute_logical_timers() {
        let process = ProcessContext::new(job());
        let result = process
            .poll_oneoff(
                100,
                &[
                    clock(1, 0, ClockTimeout::Relative),
                    clock(2, 101, ClockTimeout::Absolute),
                    Subscription::Clock(ClockSubscription {
                        userdata: 3,
                        clock_id: ClockId::Realtime as u32,
                        timeout_ns: EPOCH_2000_NS + 99,
                        timeout: ClockTimeout::Absolute,
                    }),
                ],
            )
            .unwrap();
        assert_eq!(
            result.events,
            vec![PollEvent { userdata: 1 }, PollEvent { userdata: 3 }]
        );
        assert_eq!(result.next_wake_fuel, None);

        assert_eq!(
            process.poll_oneoff(100, &[clock(4, 7, ClockTimeout::Relative)]),
            Ok(PollResult {
                events: Vec::new(),
                next_wake_fuel: Some(107),
            })
        );
        assert_eq!(
            process.poll_oneoff(100, &[clock(5, 101, ClockTimeout::Absolute)]),
            Ok(PollResult {
                events: Vec::new(),
                next_wake_fuel: Some(101),
            })
        );
    }

    #[test]
    fn poll_oneoff_rejects_fd_unknown_clock_empty_and_overflow_without_partial_events() {
        let process = ProcessContext::new(job());
        assert_eq!(
            process.poll_oneoff(0, &[]),
            Err(ProcessError::Wasi(Errno::Inval))
        );
        assert_eq!(
            process.poll_oneoff(
                5,
                &[
                    clock(1, 0, ClockTimeout::Relative),
                    Subscription::FdRead { userdata: 2, fd: 0 }
                ],
            ),
            Err(ProcessError::Wasi(Errno::Inval))
        );
        assert_eq!(
            process.poll_oneoff(
                5,
                &[Subscription::Clock(ClockSubscription {
                    userdata: 1,
                    clock_id: 77,
                    timeout_ns: 0,
                    timeout: ClockTimeout::Relative,
                })],
            ),
            Err(ProcessError::Wasi(Errno::Inval))
        );
        assert_eq!(
            process.poll_oneoff(u64::MAX, &[clock(1, 1, ClockTimeout::Relative)]),
            Err(ProcessError::Wasi(Errno::Inval))
        );
    }
}
