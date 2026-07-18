//! Pure ownership for the current-boot singleton TCP transport.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransportOwner {
    domain: u64,
    id: u64,
}

impl TransportOwner {
    pub const fn new(domain: u64, id: u64) -> Self {
        Self { domain, id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransportLeaseToken {
    owner: TransportOwner,
    generation: u64,
}

impl TransportLeaseToken {
    pub const fn owner(self) -> TransportOwner {
        self.owner
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportLeaseError {
    NetworkTransportBusy,
    ForeignOwner,
    StaleGeneration,
    LeaseTimedOut,
    LeaseRevoked,
    GenerationExhausted,
}

impl TransportLeaseError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NetworkTransportBusy => "network_transport_busy",
            Self::ForeignOwner => "transport_lease_foreign_owner",
            Self::StaleGeneration => "transport_lease_stale_generation",
            Self::LeaseTimedOut => "transport_lease_timed_out",
            Self::LeaseRevoked => "transport_lease_revoked",
            Self::GenerationExhausted => "transport_lease_generation_exhausted",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportLeaseRelease {
    Released,
    AlreadyReleased,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndReason {
    Released,
    TimedOut,
    Revoked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveLease {
    token: TransportLeaseToken,
    deadline_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EndedLease {
    token: TransportLeaseToken,
    reason: EndReason,
}

#[derive(Debug)]
pub struct TransportLease {
    active: Option<ActiveLease>,
    last_ended: Option<EndedLease>,
    last_generation: u64,
}

impl TransportLease {
    pub const fn new() -> Self {
        Self {
            active: None,
            last_ended: None,
            last_generation: 0,
        }
    }

    pub fn claim(
        &mut self,
        owner: TransportOwner,
        now_ms: u64,
        timeout_ms: u64,
    ) -> Result<TransportLeaseToken, TransportLeaseError> {
        if self.active.is_some() {
            return Err(TransportLeaseError::NetworkTransportBusy);
        }
        let generation = self
            .last_generation
            .checked_add(1)
            .ok_or(TransportLeaseError::GenerationExhausted)?;
        let token = TransportLeaseToken { owner, generation };
        self.last_generation = generation;
        self.active = Some(ActiveLease {
            token,
            deadline_ms: now_ms.saturating_add(timeout_ms.max(1)),
        });
        Ok(token)
    }

    pub fn progress(
        &mut self,
        token: TransportLeaseToken,
        now_ms: u64,
    ) -> Result<(), TransportLeaseError> {
        if self
            .active
            .map(|active| now_ms >= active.deadline_ms)
            .unwrap_or(false)
        {
            self.end_active(EndReason::TimedOut);
        }
        let Some(active) = self.active else {
            return Err(self.ended_error(token));
        };
        if active.token.owner != token.owner {
            return Err(TransportLeaseError::ForeignOwner);
        }
        if active.token.generation != token.generation {
            return Err(TransportLeaseError::StaleGeneration);
        }
        Ok(())
    }

    pub fn release(
        &mut self,
        token: TransportLeaseToken,
        now_ms: u64,
    ) -> Result<TransportLeaseRelease, TransportLeaseError> {
        if self.active.is_none()
            && self.last_ended
                == Some(EndedLease {
                    token,
                    reason: EndReason::Released,
                })
        {
            return Ok(TransportLeaseRelease::AlreadyReleased);
        }
        self.progress(token, now_ms)?;
        self.end_active(EndReason::Released);
        Ok(TransportLeaseRelease::Released)
    }

    pub fn expire(&mut self, now_ms: u64) -> Option<TransportLeaseToken> {
        let active = self.active?;
        if now_ms < active.deadline_ms {
            return None;
        }
        self.end_active(EndReason::TimedOut);
        Some(active.token)
    }

    pub fn revoke(&mut self) -> Option<TransportLeaseToken> {
        let active = self.active?;
        self.end_active(EndReason::Revoked);
        Some(active.token)
    }

    pub const fn is_held(&self) -> bool {
        self.active.is_some()
    }

    fn end_active(&mut self, reason: EndReason) {
        if let Some(active) = self.active.take() {
            self.last_ended = Some(EndedLease {
                token: active.token,
                reason,
            });
        }
    }

    fn ended_error(&self, token: TransportLeaseToken) -> TransportLeaseError {
        match self.last_ended {
            Some(EndedLease {
                token: ended,
                reason: EndReason::TimedOut,
            }) if ended == token => TransportLeaseError::LeaseTimedOut,
            Some(EndedLease {
                token: ended,
                reason: EndReason::Revoked,
            }) if ended == token => TransportLeaseError::LeaseRevoked,
            _ => TransportLeaseError::StaleGeneration,
        }
    }
}

impl Default for TransportLease {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NATIVE: TransportOwner = TransportOwner::new(1, 0);
    const TEST: TransportOwner = TransportOwner::new(2, 7);

    #[test]
    fn exclusive_claim_and_same_owner_progress() {
        let mut lease = TransportLease::new();
        let token = lease.claim(NATIVE, 10, 100).unwrap();
        assert_eq!(lease.progress(token, 20), Ok(()));
        assert_eq!(
            lease.claim(TEST, 20, 100),
            Err(TransportLeaseError::NetworkTransportBusy)
        );
        assert_eq!(
            lease.claim(NATIVE, 20, 100),
            Err(TransportLeaseError::NetworkTransportBusy)
        );
    }

    #[test]
    fn foreign_and_stale_send_receive_close_abort_are_denied() {
        let mut lease = TransportLease::new();
        let foreign = lease.claim(TEST, 0, 100).unwrap();
        assert_eq!(
            lease.release(foreign, 1),
            Ok(TransportLeaseRelease::Released)
        );
        let first = lease.claim(NATIVE, 2, 100).unwrap();
        for _operation in ["send", "receive"] {
            assert_eq!(
                lease.progress(foreign, 3),
                Err(TransportLeaseError::ForeignOwner)
            );
        }
        for _operation in ["close", "abort"] {
            assert_eq!(
                lease.release(foreign, 3),
                Err(TransportLeaseError::ForeignOwner)
            );
        }
        assert_eq!(lease.progress(first, 3), Ok(()));
        assert_eq!(lease.release(first, 3), Ok(TransportLeaseRelease::Released));
        let second = lease.claim(NATIVE, 4, 100).unwrap();
        assert_eq!(
            lease.progress(first, 5),
            Err(TransportLeaseError::StaleGeneration)
        );
        assert_eq!(
            lease.release(first, 5),
            Err(TransportLeaseError::StaleGeneration)
        );
        assert_eq!(lease.progress(second, 5), Ok(()));
    }

    #[test]
    fn timeout_and_revocation_release_for_retry() {
        let mut lease = TransportLease::new();
        let timed = lease.claim(TEST, 10, 5).unwrap();
        assert_eq!(lease.expire(14), None);
        assert_eq!(lease.expire(15), Some(timed));
        assert_eq!(
            lease.progress(timed, 15),
            Err(TransportLeaseError::LeaseTimedOut)
        );
        let revoked = lease.claim(NATIVE, 15, 100).unwrap();
        assert_eq!(lease.revoke(), Some(revoked));
        assert_eq!(
            lease.progress(revoked, 16),
            Err(TransportLeaseError::LeaseRevoked)
        );
        assert!(lease.claim(TEST, 16, 100).is_ok());
    }

    #[test]
    fn generations_advance_and_owner_teardown_is_idempotent() {
        let mut lease = TransportLease::new();
        let first = lease.claim(NATIVE, 0, 100).unwrap();
        assert_eq!(lease.release(first, 1), Ok(TransportLeaseRelease::Released));
        assert_eq!(
            lease.release(first, 2),
            Ok(TransportLeaseRelease::AlreadyReleased)
        );
        let second = lease.claim(NATIVE, 3, 100).unwrap();
        assert!(second.generation() > first.generation());
        assert_eq!(
            lease.release(first, 4),
            Err(TransportLeaseError::StaleGeneration)
        );
        assert_eq!(lease.progress(second, 4), Ok(()));
    }
}
