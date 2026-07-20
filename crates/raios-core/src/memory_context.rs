//! Read-only memory-context policy.
//!
//! The plan is a pure description for the kernel emitter.  It never reads
//! memory, event logs, provider state, or recovery state and never grants a
//! mutation or export capability.

use crate::{method_eq, method_head_eq};

pub const MEMORY_EVENT_DEFAULT_LIMIT: usize = 32;
pub const MEMORY_MUTATION_METHODS: &[&str] = &[
    "memory.record_observation",
    "memory.propose_policy",
    "memory.supersede_fact",
    "memory.redact",
    "memory.compact",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryContextPlan {
    pub profile: &'static str,
    pub target_tokens: u16,
    pub estimated_tokens: u16,
    pub event_limit: usize,
    pub provider_export: bool,
    pub mutation_allowed: bool,
}

pub fn memory_mutation_method(method: &str) -> bool {
    MEMORY_MUTATION_METHODS
        .iter()
        .any(|candidate| method_eq(method, candidate))
}

pub fn memory_context_profile(method: &str) -> &'static str {
    let arg = memory_method_arg(method, "memory.context");
    if method_eq(arg, "planning") {
        "planning"
    } else if method_eq(arg, "provider_minimal") {
        "provider_minimal"
    } else {
        "diagnostic"
    }
}

pub fn memory_context_target_tokens(profile: &str) -> u16 {
    if method_eq(profile, "planning") {
        8000
    } else if method_eq(profile, "provider_minimal") {
        2000
    } else {
        4000
    }
}

pub fn memory_context_estimated_tokens(profile: &str) -> u16 {
    if method_eq(profile, "planning") {
        1600
    } else if method_eq(profile, "provider_minimal") {
        900
    } else {
        1200
    }
}

pub fn memory_method_arg<'a>(method: &'a str, canonical: &str) -> &'a str {
    let method = method.trim();
    let head_len = if method_head_eq(method, canonical) {
        canonical.len()
    } else if method_head_eq(method, "memctx") {
        "memctx".len()
    } else if method_head_eq(method, "memtrace") {
        "memtrace".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

pub fn event_limit_arg(method: &str) -> usize {
    let method = method.trim();
    let head_len = if method_head_eq(method, "memory.recent_events") {
        "memory.recent_events".len()
    } else if method_head_eq(method, "audit.events") {
        "audit.events".len()
    } else if method_head_eq(method, "events") {
        "events".len()
    } else {
        return MEMORY_EVENT_DEFAULT_LIMIT;
    };
    parse_usize_arg(method[head_len..].trim())
}

pub fn parse_usize_arg(value: &str) -> usize {
    let mut parsed = 0usize;
    let mut saw_digit = false;
    for byte in value.bytes() {
        if byte.is_ascii_whitespace() {
            if saw_digit {
                break;
            }
            continue;
        }
        if !byte.is_ascii_digit() {
            break;
        }
        saw_digit = true;
        parsed = parsed
            .saturating_mul(10)
            .saturating_add((byte - b'0') as usize);
    }
    if saw_digit {
        parsed
    } else {
        MEMORY_EVENT_DEFAULT_LIMIT
    }
}

pub fn memory_context_plan(method: &str) -> MemoryContextPlan {
    let profile = memory_context_profile(method);
    MemoryContextPlan {
        profile,
        target_tokens: memory_context_target_tokens(profile),
        estimated_tokens: memory_context_estimated_tokens(profile),
        event_limit: event_limit_arg(method),
        provider_export: false,
        mutation_allowed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_vocabulary_remains_denied() {
        assert_eq!(MEMORY_MUTATION_METHODS.len(), 5);
        assert!(memory_mutation_method("Memory.Redact"));
        assert!(!memory_mutation_method("memory.context"));
    }

    #[test]
    fn profiles_and_budgets_match_guest_policy() {
        assert_eq!(
            memory_context_profile("memory.context planning"),
            "planning"
        );
        assert_eq!(
            memory_context_profile("memctx provider_minimal"),
            "provider_minimal"
        );
        assert_eq!(
            memory_context_profile("memory.context unknown"),
            "diagnostic"
        );
        assert_eq!(memory_context_target_tokens("planning"), 8000);
        assert_eq!(memory_context_estimated_tokens("provider_minimal"), 900);
    }

    #[test]
    fn event_limits_parse_without_panicking() {
        assert_eq!(event_limit_arg("memory.recent_events 7"), 7);
        assert_eq!(event_limit_arg("audit.events \t 12 trailing"), 12);
        assert_eq!(
            event_limit_arg("events malformed"),
            MEMORY_EVENT_DEFAULT_LIMIT
        );
        assert_eq!(parse_usize_arg("999999999999999999999999"), usize::MAX);
        assert_eq!(parse_usize_arg("   "), MEMORY_EVENT_DEFAULT_LIMIT);
    }

    #[test]
    fn plan_cannot_authorize_mutation_or_export() {
        let plan = memory_context_plan("memory.context planning");
        assert_eq!(plan.profile, "planning");
        assert!(!plan.provider_export);
        assert!(!plan.mutation_allowed);
    }
}
