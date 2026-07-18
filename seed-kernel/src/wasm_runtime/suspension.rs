use super::invocation::BeyondEnvState;
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct HostSuspend {
    pub(super) invocation_id: u64,
    pub(super) operation_id: u32,
}

impl fmt::Display for HostSuspend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("raios host operation suspended")
    }
}

impl wasmi::core::HostError for HostSuspend {}

#[derive(Clone, Copy, Debug)]
pub(super) struct FixtureHostError;

impl fmt::Display for FixtureHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("raios labeled fixture host error")
    }
}

impl wasmi::core::HostError for FixtureHostError {}

pub(super) fn classify_resumable(
    invocation: &ResumableInvocation,
    store: &Store<BeyondEnvState>,
) -> Result<(), TerminalOutcome> {
    match invocation.suspension() {
        wasmi::Suspension::Host { host_error, .. } => {
            if matches!(host_error.trap_code(), Some(TrapCode::OutOfFuel)) {
                return Err(TerminalOutcome::OutOfFuel);
            }
            let marker = host_error
                .downcast_ref::<HostSuspend>()
                .ok_or(TerminalOutcome::HostError)?;
            if !store
                .data()
                .lifecycle
                .suspend_marker_matches(marker.invocation_id, marker.operation_id)
            {
                return Err(TerminalOutcome::HostError);
            }
            Ok(())
        }
        // Default-closed until T2 provides the scheduler (ADR 0016).
        wasmi::Suspension::Atomic(_) => Err(TerminalOutcome::HostError),
        // Unreachable while the kernel keeps resumable_fuel off; T2 wires it.
        wasmi::Suspension::FuelQuantum => Err(TerminalOutcome::OutOfFuel),
    }
}

pub(super) fn terminal_from_error(error: &wasmi::Error) -> TerminalOutcome {
    match error {
        wasmi::Error::Trap(trap) if matches!(trap.trap_code(), Some(TrapCode::OutOfFuel)) => {
            TerminalOutcome::OutOfFuel
        }
        wasmi::Error::Trap(trap)
            if matches!(trap.trap_code(), Some(TrapCode::UnreachableCodeReached)) =>
        {
            TerminalOutcome::GuestTrap
        }
        wasmi::Error::Trap(_) => TerminalOutcome::HostError,
        _ => TerminalOutcome::HostError,
    }
}
