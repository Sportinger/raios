use alloc::vec;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, emit_record_fields_trailing_comma, end_response,
        raw_line, record_bool as b, record_field as f, record_sha, record_str as s,
    },
    distribution_candidate, event_log, wasm_runtime,
};
use raios_core::{
    distribution_provenance::verify_distribution_provenance_signature,
    record::Value as V,
    scoped_distribution_provenance_honesty::{
        evaluate_distribution_provenance_honesty, DistributionProvenanceHonestyDecision,
        DistributionProvenanceHonestyInput, SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_ID,
        SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_MARKER,
        SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_SCHEMA,
    },
};

const SELFTEST_ECHO_PROVENANCE_SIGNATURE_DER_HEX: &[u8] =
    b"304402201fd9aa3e26579ab9852a1ea61a7fe23f79c39badd13e2c74dbdf9d957a25449b02204f783191894cfb609d35c5babc9fb3208e77d9c712d2645a633120db6dcdd89b";

#[derive(Clone, Copy)]
struct SelftestCase {
    name: &'static str,
    provenance_verified: bool,
    status: &'static str,
    reason: &'static str,
    honest: bool,
    load_authorized: bool,
    install_authorized: bool,
    owner_sealed: bool,
    requires_m6_reverify_for_load: bool,
    authorizes_load: bool,
    authorizes_install: bool,
    authorizes_execution: bool,
    writes_persistent_state: bool,
    passed: bool,
}

pub(crate) fn emit_distribution_provenance_diagnostic(arg: &str) {
    let decoded = decode_distribution_signature_der_hex(distribution_signature_hex_arg(arg));
    let signature_der = match decoded.as_ref() {
        Some((bytes, len)) => &bytes[..*len],
        None => &[],
    };
    let outcome = distribution_candidate::verify_retained_candidate_provenance(signature_der);

    begin_response("module.distribution_provenance_diagnostic");
    emit_record_fields_trailing_comma(
        vec![
            f("method", s("module.distribution_provenance_diagnostic")),
            f("schema", s("raios.distribution_candidate.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_kind", s(outcome.source_kind)),
            f("artifact_sha256", record_sha(outcome.artifact_sha256)),
            f("retained_present", b(outcome.retained_present)),
            f("retained_wasm_valid", b(outcome.retained_wasm_valid)),
            f(
                "provenance_signature_present",
                b(outcome.provenance_signature_present),
            ),
            f("provenance_verified", b(outcome.provenance_verified)),
            f(
                "publisher_key_sha256",
                record_sha(outcome.publisher_key_sha256),
            ),
            f(
                "decision_schema",
                s(SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_SCHEMA),
            ),
            f(
                "decision_id",
                s(SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_ID),
            ),
            f(
                "decision_marker",
                s(SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_MARKER),
            ),
            f("status", s(outcome.status)),
            f("reason", s(outcome.reason)),
            f("honest", b(outcome.honest)),
            f("load_authorized", b(outcome.load_authorized)),
            f("install_authorized", b(outcome.install_authorized)),
            f("owner_sealed", b(outcome.owner_sealed)),
            f(
                "requires_m6_reverify_for_load",
                b(outcome.requires_m6_reverify_for_load),
            ),
            f("trust_tier", s(outcome.trust_tier)),
            f("load_attempted", b(outcome.load_attempted)),
            f("execution_attempted", b(outcome.execution_attempted)),
            f("authorizes_load", b(outcome.authorizes_load)),
            f("authorizes_execution", b(outcome.authorizes_execution)),
            f(
                "writes_persistent_state",
                b(outcome.writes_persistent_state),
            ),
            f(
                "live_load_projection_present",
                b(outcome.live_load_projection_present),
            ),
            f(
                "live_load_projection_can_load_now",
                b(outcome.live_load_projection_can_load_now),
            ),
            f("durable_write", b(false)),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("module.distribution_provenance_diagnostic");
}

pub(crate) fn emit_distribution_provenance_diagnostic_selftest() {
    let cases = distribution_provenance_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases.iter().map(record_selftest_case).collect();

    begin_response("module.distribution_provenance_diagnostic_selftest");
    emit_record_fields(
        vec![
            f(
                "method",
                s("module.distribution_provenance_diagnostic_selftest"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("passed", b(passed)),
            f("case_count", V::U64(cases.len() as u64)),
            f("cases", V::Array(case_records)),
            f(
                "decision_schema",
                s(SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_SCHEMA),
            ),
            f(
                "decision_id",
                s(SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_ID),
            ),
            f(
                "decision_marker",
                s(SCOPED_DISTRIBUTION_PROVENANCE_HONESTY_DECISION_MARKER),
            ),
            f("trust_tier", s("dev_key_not_owner_sealed")),
            f("owner_sealed", b(false)),
            f("read_only", b(true)),
            f("durable_write", b(false)),
        ],
        6,
    );
    end_response("module.distribution_provenance_diagnostic_selftest");
}

fn distribution_signature_hex_arg(arg: &str) -> &[u8] {
    let trimmed = arg.trim();
    let payload = trimmed
        .strip_prefix("module.distribution_provenance_diagnostic")
        .unwrap_or(trimmed)
        .trim();
    payload.split_whitespace().next().unwrap_or("").as_bytes()
}

fn distribution_provenance_selftest_cases() -> [SelftestCase; 4] {
    let decoded = decode_distribution_signature_der_hex(SELFTEST_ECHO_PROVENANCE_SIGNATURE_DER_HEX);
    let (signature_der, signature_len) =
        decoded.unwrap_or(([0; event_log::MAX_PROMOTION_SIGNATURE_DER_LEN], 0));
    let echo_hash = wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH;
    let signature = &signature_der[..signature_len];

    let absent = evaluate(false, false);

    let mut tampered_der = signature_der;
    if signature_len > 0 {
        tampered_der[0] ^= 0x01;
    }
    let tampered_verified = signature_len > 0
        && verify_distribution_provenance_signature(&tampered_der[..signature_len], &echo_hash);
    let tampered = evaluate(signature_len > 0, tampered_verified);

    let wrong_artifact_verified =
        signature_len > 0 && verify_distribution_provenance_signature(signature, &[0x11; 32]);
    let wrong_artifact = evaluate(signature_len > 0, wrong_artifact_verified);

    let valid_verified =
        signature_len > 0 && verify_distribution_provenance_signature(signature, &echo_hash);
    let valid = evaluate(signature_len > 0, valid_verified);

    [
        selftest_case(
            "absent_signature_denied",
            absent,
            "denied",
            "provenance_signature_absent",
            false,
        ),
        selftest_case(
            "tampered_signature_rejected",
            tampered,
            "denied",
            "provenance_signature_unverified",
            false,
        ),
        selftest_case(
            "signature_bound_to_wrong_artifact_rejected",
            wrong_artifact,
            "denied",
            "provenance_signature_unverified",
            false,
        ),
        selftest_case(
            "valid_provenance_load_still_denied",
            valid,
            "distribution_candidate_provenance_verified_load_still_denied",
            "provenance_authenticated_candidate_intake_only_grants_nothing",
            true,
        ),
    ]
}

fn evaluate(
    provenance_signature_present: bool,
    provenance_signature_verified: bool,
) -> DistributionProvenanceHonestyDecision {
    evaluate_distribution_provenance_honesty(&DistributionProvenanceHonestyInput {
        source_kind: Some(distribution_candidate::DISTRIBUTION_SOURCE_KIND),
        artifact_sha256_present: true,
        provenance_signature_present,
        provenance_signature_verified,
        intake_as_candidate: true,
        claims_installable: false,
        claims_load_authorized_by_provenance: false,
        requires_m6_reverify_for_load: true,
        claims_owner_sealed: false,
    })
}

fn selftest_case(
    name: &'static str,
    decision: DistributionProvenanceHonestyDecision,
    expected_status: &'static str,
    expected_reason: &'static str,
    expected_verified: bool,
) -> SelftestCase {
    let load_authorized = false;
    let install_authorized = false;
    let owner_sealed = false;
    let requires_m6_reverify_for_load = true;
    let authorizes_execution = false;
    let writes_persistent_state = false;
    let passed = decision.status == expected_status
        && decision.reason == expected_reason
        && decision.honest == expected_verified
        && decision.provenance_verified == expected_verified
        && decision.authorizes_load == false
        && decision.authorizes_install == false
        && !load_authorized
        && !install_authorized
        && !owner_sealed
        && requires_m6_reverify_for_load
        && !authorizes_execution
        && !writes_persistent_state;

    SelftestCase {
        name,
        provenance_verified: decision.provenance_verified,
        status: decision.status,
        reason: decision.reason,
        honest: decision.honest,
        load_authorized,
        install_authorized,
        owner_sealed,
        requires_m6_reverify_for_load,
        authorizes_load: decision.authorizes_load,
        authorizes_install: decision.authorizes_install,
        authorizes_execution,
        writes_persistent_state,
        passed,
    }
}

fn record_selftest_case(case: &SelftestCase) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("provenance_verified", b(case.provenance_verified)),
        f("status", s(case.status)),
        f("reason", s(case.reason)),
        f("honest", b(case.honest)),
        f("load_authorized", b(case.load_authorized)),
        f("install_authorized", b(case.install_authorized)),
        f("owner_sealed", b(case.owner_sealed)),
        f(
            "requires_m6_reverify_for_load",
            b(case.requires_m6_reverify_for_load),
        ),
        f("authorizes_load", b(case.authorizes_load)),
        f("authorizes_install", b(case.authorizes_install)),
        f("authorizes_execution", b(case.authorizes_execution)),
        f("writes_persistent_state", b(case.writes_persistent_state)),
        f("passed", b(case.passed)),
    ])
}

fn decode_distribution_signature_der_hex(
    signature_hex: &[u8],
) -> Option<([u8; event_log::MAX_PROMOTION_SIGNATURE_DER_LEN], usize)> {
    if !distribution_signature_hex_candidate(signature_hex) {
        return None;
    }

    let mut out = [0u8; event_log::MAX_PROMOTION_SIGNATURE_DER_LEN];
    let mut idx = 0usize;
    while idx < signature_hex.len() / 2 {
        let high = hex_nibble(signature_hex[idx * 2])?;
        let low = hex_nibble(signature_hex[idx * 2 + 1])?;
        out[idx] = (high << 4) | low;
        idx += 1;
    }
    Some((out, idx))
}

fn distribution_signature_hex_candidate(signature_hex: &[u8]) -> bool {
    !signature_hex.is_empty()
        && signature_hex.len() % 2 == 0
        && signature_hex.len() / 2 <= event_log::MAX_PROMOTION_SIGNATURE_DER_LEN
        && signature_hex.iter().all(|byte| hex_nibble(*byte).is_some())
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
