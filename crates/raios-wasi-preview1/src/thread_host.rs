//! Capability boundary for the non-preview1 `wasi.thread-spawn` import.

/// Host-side thread reservation for a single WASI build instance.
///
/// Non-negative returns are guest thread identifiers. Negative returns are a
/// fail-closed denial and do not create a guest thread.
pub trait ThreadHost {
    fn spawn(&mut self, start_arg: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::ThreadHost;

    struct DenyAll;

    impl ThreadHost for DenyAll {
        fn spawn(&mut self, _start_arg: i32) -> i32 {
            -1
        }
    }

    #[test]
    fn negative_return_is_available_as_the_denial_boundary() {
        let mut host = DenyAll;
        assert!(host.spawn(7) < 0);
    }
}
