//! Pure ownership and budget decisions for one current-boot beyond-env invocation.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvocationAuthority {
    pub service_id: &'static str,
    pub invocation_id: u64,
    pub service_generation: u64,
    pub instance_generation: u64,
    pub captured_kill_generation: u64,
    pub policy_allows_beyond_env: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingHostOperation {
    pub invocation_id: u64,
    pub operation_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalOutcome {
    Finished,
    GuestTrap,
    HostError,
    OutOfFuel,
    Killed,
    GenerationInvalidated,
    WallBudgetExceeded,
    StepBudgetExceeded,
    Abandoned,
}

impl TerminalOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::GuestTrap => "guest_trap",
            Self::HostError => "host_error",
            Self::OutOfFuel => "out_of_fuel",
            Self::Killed => "killed",
            Self::GenerationInvalidated => "generation_invalidated",
            Self::WallBudgetExceeded => "wall_budget_exceeded",
            Self::StepBudgetExceeded => "step_budget_exceeded",
            Self::Abandoned => "abandoned",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundaryState {
    pub now_ms: u64,
    pub kill_generation: u64,
    pub service_generation: u64,
    pub instance_generation: u64,
    pub posture_allows_execution: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandleGeneration {
    slot: u8,
    generation: u32,
    live: bool,
}

impl HandleGeneration {
    pub const fn new(slot: u8, generation: u32) -> Self {
        Self {
            slot,
            generation: if generation == 0 { 1 } else { generation },
            live: true,
        }
    }

    pub fn handle(self) -> i32 {
        (((self.generation & 0x007f_ffff) << 8) | u32::from(self.slot)) as i32
    }

    pub fn accepts(self, handle: i32) -> bool {
        self.live && handle > 0 && handle == self.handle()
    }

    pub fn invalidate(&mut self) {
        self.live = false;
        self.generation = self.generation.wrapping_add(1).max(1);
    }

    pub const fn is_live(self) -> bool {
        self.live
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeardownReceipt {
    pub teardown_count: u32,
    pub handles_invalid: bool,
    pub pending_operation_absent: bool,
    pub owned_socket_aborted: bool,
    pub tls_slots_zeroized: bool,
    pub incomplete_acquisition_discarded: bool,
    pub prior_candidate_unchanged: bool,
    pub singleton_lease_released: bool,
    pub pending_buffers_cleared: bool,
    pub terminal_outcome_recorded: bool,
}

impl TeardownReceipt {
    const fn new() -> Self {
        Self {
            teardown_count: 0,
            handles_invalid: false,
            pending_operation_absent: false,
            owned_socket_aborted: false,
            tls_slots_zeroized: false,
            incomplete_acquisition_discarded: false,
            prior_candidate_unchanged: true,
            singleton_lease_released: false,
            pending_buffers_cleared: false,
            terminal_outcome_recorded: false,
        }
    }
}

#[derive(Debug)]
pub struct InvocationLifecycle {
    authority: InvocationAuthority,
    handle: HandleGeneration,
    pending: Option<PendingHostOperation>,
    started_ms: u64,
    wall_budget_ms: u64,
    max_steps: u32,
    steps_used: u32,
    suspension_count: u32,
    resume_count: u32,
    terminal: Option<TerminalOutcome>,
    receipt: TeardownReceipt,
}

impl InvocationLifecycle {
    pub const fn new(
        authority: InvocationAuthority,
        started_ms: u64,
        wall_budget_ms: u64,
        max_steps: u32,
        handle_generation: u32,
    ) -> Self {
        Self {
            authority,
            handle: HandleGeneration::new(1, handle_generation),
            pending: None,
            started_ms,
            wall_budget_ms,
            max_steps,
            steps_used: 0,
            suspension_count: 0,
            resume_count: 0,
            terminal: None,
            receipt: TeardownReceipt::new(),
        }
    }

    pub const fn authority(&self) -> InvocationAuthority {
        self.authority
    }

    pub fn handle(&self) -> i32 {
        self.handle.handle()
    }

    pub fn accepts_handle(&self, handle: i32) -> bool {
        self.handle.accepts(handle)
    }

    pub const fn pending(&self) -> Option<PendingHostOperation> {
        self.pending
    }

    pub fn record_pending(&mut self, operation_id: u32) -> Option<PendingHostOperation> {
        if self.terminal.is_some() || self.pending.is_some() {
            return None;
        }
        let pending = PendingHostOperation {
            invocation_id: self.authority.invocation_id,
            operation_id,
        };
        self.pending = Some(pending);
        self.suspension_count = self.suspension_count.saturating_add(1);
        Some(pending)
    }

    pub fn suspend_marker_matches(&self, invocation_id: u64, operation_id: u32) -> bool {
        self.pending
            == Some(PendingHostOperation {
                invocation_id,
                operation_id,
            })
    }

    pub fn take_pending(&mut self) -> Option<PendingHostOperation> {
        self.pending.take()
    }

    pub fn check_boundary(&mut self, state: BoundaryState) -> Result<(), TerminalOutcome> {
        if state.kill_generation != self.authority.captured_kill_generation {
            return Err(TerminalOutcome::Killed);
        }
        if state.service_generation != self.authority.service_generation
            || state.instance_generation != self.authority.instance_generation
            || !state.posture_allows_execution
        {
            return Err(TerminalOutcome::GenerationInvalidated);
        }
        if state.now_ms.saturating_sub(self.started_ms) >= self.wall_budget_ms {
            return Err(TerminalOutcome::WallBudgetExceeded);
        }
        if self.steps_used >= self.max_steps {
            return Err(TerminalOutcome::StepBudgetExceeded);
        }
        self.steps_used += 1;
        Ok(())
    }

    pub fn note_resume(&mut self) {
        self.resume_count = self.resume_count.saturating_add(1);
    }

    /// Runs lifecycle teardown steps 1-8. The kernel owner drops wasmi state as step 9.
    pub fn teardown(&mut self, outcome: TerminalOutcome) -> bool {
        if self.receipt.teardown_count != 0 {
            return false;
        }
        // 1. Terminalize the owner and invalidate every issued generation.
        self.terminal = Some(outcome);
        self.handle.invalidate();
        // 2. Remove the sole pending operation before any resource cleanup.
        self.pending = None;
        // 3-7. NET-2 owns no socket, TLS slot, acquisition, lease, or payload
        // buffer; their receipt fields therefore stay false rather than
        // pretending an effect occurred. The prior candidate remains intact.
        // 8. Publish the single cleanup receipt. The kernel owner performs
        // step 9 by dropping continuation, handles, and Store without a lock.
        self.receipt = TeardownReceipt {
            teardown_count: 1,
            handles_invalid: !self.handle.is_live(),
            pending_operation_absent: true,
            owned_socket_aborted: false,
            tls_slots_zeroized: false,
            incomplete_acquisition_discarded: false,
            prior_candidate_unchanged: true,
            singleton_lease_released: false,
            pending_buffers_cleared: true,
            terminal_outcome_recorded: true,
        };
        true
    }

    pub const fn terminal(&self) -> Option<TerminalOutcome> {
        self.terminal
    }

    pub const fn teardown_receipt(&self) -> TeardownReceipt {
        self.receipt
    }

    pub const fn steps_used(&self) -> u32 {
        self.steps_used
    }

    pub const fn max_steps(&self) -> u32 {
        self.max_steps
    }

    pub const fn wall_budget_ms(&self) -> u64 {
        self.wall_budget_ms
    }

    pub const fn suspension_count(&self) -> u32 {
        self.suspension_count
    }

    pub const fn resume_count(&self) -> u32 {
        self.resume_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lifecycle() -> InvocationLifecycle {
        InvocationLifecycle::new(
            InvocationAuthority {
                service_id: "test.fixture.beyond_env_lifecycle",
                invocation_id: 7,
                service_generation: 2,
                instance_generation: 3,
                captured_kill_generation: 11,
                policy_allows_beyond_env: false,
            },
            100,
            90_000,
            3,
            9,
        )
    }

    fn boundary(now_ms: u64) -> BoundaryState {
        BoundaryState {
            now_ms,
            kill_generation: 11,
            service_generation: 2,
            instance_generation: 3,
            posture_allows_execution: true,
        }
    }

    #[test]
    fn start_suspend_resume_finish_and_stale_handle_denial() {
        let mut invocation = lifecycle();
        let handle = invocation.handle();
        assert!(invocation.accepts_handle(handle));
        assert!(invocation.record_pending(1).is_some());
        assert!(invocation.suspend_marker_matches(7, 1));
        assert_eq!(invocation.take_pending().unwrap().operation_id, 1);
        invocation.note_resume();
        assert!(invocation.teardown(TerminalOutcome::Finished));
        assert!(!invocation.accepts_handle(handle));
        assert_eq!(invocation.resume_count(), 1);
        assert_eq!(invocation.teardown_receipt().teardown_count, 1);
    }

    #[test]
    fn mismatched_suspend_marker_is_rejected() {
        let mut invocation = lifecycle();
        invocation.record_pending(4).unwrap();
        assert!(!invocation.suspend_marker_matches(8, 4));
        assert!(!invocation.suspend_marker_matches(7, 5));
    }

    #[test]
    fn kill_and_generation_changes_fail_before_resume() {
        let mut invocation = lifecycle();
        let mut killed = boundary(101);
        killed.kill_generation += 1;
        assert_eq!(
            invocation.check_boundary(killed),
            Err(TerminalOutcome::Killed)
        );

        let mut invocation = lifecycle();
        let mut stale = boundary(101);
        stale.instance_generation += 1;
        assert_eq!(
            invocation.check_boundary(stale),
            Err(TerminalOutcome::GenerationInvalidated)
        );
    }

    #[test]
    fn wall_and_step_budgets_are_cumulative() {
        let mut invocation = lifecycle();
        assert_eq!(
            invocation.check_boundary(boundary(90_100)),
            Err(TerminalOutcome::WallBudgetExceeded)
        );

        let mut invocation = lifecycle();
        assert!(invocation.check_boundary(boundary(101)).is_ok());
        assert!(invocation.check_boundary(boundary(102)).is_ok());
        assert!(invocation.check_boundary(boundary(103)).is_ok());
        assert_eq!(
            invocation.check_boundary(boundary(104)),
            Err(TerminalOutcome::StepBudgetExceeded)
        );
    }

    #[test]
    fn every_terminal_route_uses_exactly_once_teardown() {
        for outcome in [
            TerminalOutcome::Finished,
            TerminalOutcome::GuestTrap,
            TerminalOutcome::HostError,
            TerminalOutcome::OutOfFuel,
            TerminalOutcome::Killed,
            TerminalOutcome::GenerationInvalidated,
            TerminalOutcome::WallBudgetExceeded,
            TerminalOutcome::StepBudgetExceeded,
            TerminalOutcome::Abandoned,
        ] {
            let mut invocation = lifecycle();
            invocation.record_pending(1).unwrap();
            assert!(invocation.teardown(outcome));
            assert!(!invocation.teardown(outcome));
            let receipt = invocation.teardown_receipt();
            assert_eq!(receipt.teardown_count, 1);
            assert!(receipt.handles_invalid && receipt.pending_operation_absent);
            assert!(!receipt.singleton_lease_released && receipt.prior_candidate_unchanged);
        }
    }
}
