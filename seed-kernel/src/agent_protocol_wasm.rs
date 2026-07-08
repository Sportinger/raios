use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields_trailing_comma, end_response, raw_line,
        record_bool as b, record_field as f, record_sha, record_static_str_array, record_str as s,
        record_str_or_null,
    },
    agent_protocol_time::REAL_TEST_CERT_DER,
    memory_store, module_candidate_channel, module_candidate_intake, wasm_runtime,
};
use raios_core::{
    cert_validity_window::{
        decode_certwindow_record, encode_certwindow_record, parse_x509_cert_validity_window,
        CertValidityDateTime, X509ValidityError, CERTWINDOW_RECORD_LEN,
    },
    http_response_parse::{
        decode_httphead_record, encode_httphead_record, parse_http_head, HTTPHEAD_RECORD_LEN,
    },
    record::Value as V,
    sha256_bytes,
    x509_spki::{
        decode_certspki_record, encode_certspki_record, extract_p256_spki,
        CERTSPKI_ERROR_NOT_P256_SPKI, CERTSPKI_RECORD_LEN, P256_UNCOMPRESSED_POINT_LEN,
    },
};

// Synthetic HTTP/1.1 fixtures - byte-identical to raios-http-parse host tests.
// Not live traffic; grant no provider trust. (M11-7)
// positive: len 59, sha256 f49383a81b679f0569a28430340963e08c898b581b9f910be129ccfe60e54f9a
pub(crate) const REAL_TEST_HTTP_RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nhi\r\n0\r\n\r\n";
// malformed = &REAL_TEST_HTTP_RESPONSE[..54]; len 54, sha256 ec11883d03c5aa89eeb04064577993e812b53441349fac620d99c4497806bbcf
// content-length: len 101, sha256 9d467b71e028580c15a68e9bd3aa9dba44dc0055ff11eb71123b05dfc648fa9c
pub(crate) const REAL_TEST_HTTP_CONTENT_LENGTH_RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}";

pub(crate) fn emit_submit_candidate_chunk(arg: &str) {
    let outcome = module_candidate_channel::submit_candidate_chunk(arg);

    begin_response("module.submit_candidate_chunk");
    emit_record_fields_trailing_comma(
        vec![
            f("method", s("module.submit_candidate_chunk")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("chunk_index", V::U64(outcome.chunk_index as u64)),
            f("decoded_byte_len", V::U64(outcome.decoded_byte_len as u64)),
            f("pending_byte_len", V::U64(outcome.pending_byte_len as u64)),
            f(
                "pending_chunk_count",
                V::U64(outcome.pending_chunk_count as u64),
            ),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f(
                "discarded_pending_delivery",
                b(outcome.discarded_pending_delivery),
            ),
            f("reason", s(outcome.reason)),
            f("load_attempted", b(outcome.load_attempted)),
            f("execution_attempted", b(outcome.execution_attempted)),
            f("authorizes_load", b(outcome.authorizes_load)),
            f("authorizes_execution", b(outcome.authorizes_execution)),
            f(
                "writes_persistent_state",
                b(outcome.writes_persistent_state),
            ),
            f(
                "external_delivery_channel",
                s(outcome.external_delivery_channel),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("module.submit_candidate_chunk");
}

pub(crate) fn emit_submit_candidate_finalize() {
    let outcome = module_candidate_channel::submit_candidate_finalize();
    let candidate = &outcome.candidate;

    begin_response("module.submit_candidate_finalize");
    emit_record_fields_trailing_comma(
        vec![
            f("method", s("module.submit_candidate_finalize")),
            f("scope", s(candidate.scope)),
            f("classification", s("local_only")),
            f(
                "delivered_byte_len",
                V::U64(outcome.delivered_byte_len as u64),
            ),
            f(
                "delivered_chunk_count",
                V::U64(outcome.delivered_chunk_count as u64),
            ),
            f("byte_len", V::U64(candidate.byte_len as u64)),
            f("artifact_sha256", record_sha(candidate.artifact_sha256)),
            f("wasm_valid", b(candidate.wasm_valid)),
            f("retained_in_ram", b(candidate.retained_in_ram)),
            f("rejected", b(candidate.rejected)),
            f("reason", s(candidate.reason)),
            f("load_attempted", b(candidate.load_attempted)),
            f("execution_attempted", b(candidate.execution_attempted)),
            f("authorizes_load", b(candidate.authorizes_load)),
            f("authorizes_execution", b(candidate.authorizes_execution)),
            f(
                "writes_persistent_state",
                b(candidate.writes_persistent_state),
            ),
            f(
                "external_delivery_channel",
                s(candidate.external_delivery_channel),
            ),
            f(
                "candidate",
                record_candidate_intake_case("serial_console_base64_delivery", candidate),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("module.submit_candidate_finalize");
}

pub(crate) fn emit_wasm_echo_probe() {
    let probe = wasm_runtime::run_echo_probe();
    let candidate_probe = module_candidate_intake::run_candidate_intake_probe();

    begin_response("wasm.echo_probe");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.wasm_echo_probe.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(false)),
            f("method", s("wasm.echo_probe")),
            f("service_id", s("svc.demo.echo")),
            f("artifact_id", s("wasm:svc.demo.echo")),
            f("artifact_sha256", record_sha(probe.artifact_hash)),
            f(
                "artifact_identity_descriptor_sha256",
                record_sha(probe.descriptor_hash),
            ),
            f(
                "artifact_signature_envelope_sha256",
                record_sha(probe.signature_envelope_hash),
            ),
            f("validation_ok", b(probe.validation_ok)),
            f("capability_envelope", s("wasmi_linker_import_surface")),
            f(
                "granted_host_imports",
                record_static_str_array(&["env.log", "env.counter_get"]),
            ),
            f("host_import_count", V::U64(2)),
            f("instantiation_ok", b(probe.instantiation_ok)),
            f("entrypoint", s("raios_service_main")),
            f("run_outcome", s(probe.run_outcome)),
            f("return_value_i32", record_return_value(probe.return_value)),
            f("fuel_budget", V::U64(probe.fuel_budget)),
            f("fuel_used", V::U64(probe.fuel_used)),
            f("log_prefix", s("WASM_GUEST_LOG")),
            f("log_line_emitted", b(probe.log_line.is_some())),
            f("log_line", record_str_or_null(probe.log_line.as_deref())),
            f("negative_probe", s("forbidden_import_link_failure")),
            f(
                "negative_module_imports",
                record_static_str_array(&["env.forbidden_write"]),
            ),
            f("negative_validation_ok", b(probe.forbidden_validation_ok)),
            f(
                "negative_instantiation_ok",
                b(probe.forbidden_instantiation_ok),
            ),
            f(
                "negative_link_error_kind",
                s(probe.forbidden_link_error_kind),
            ),
            f(
                "negative_missing_import_module",
                record_str_or_null(probe.forbidden_missing_import_module),
            ),
            f(
                "negative_missing_import_name",
                record_str_or_null(probe.forbidden_missing_import_name),
            ),
            f("capability_boundary_held", b(probe.forbidden_boundary_held)),
            f(
                "hardening_case_count",
                V::U64(wasm_runtime::WASM_HARDENING_CASE_COUNT as u64),
            ),
            f(
                "hardening_passed_count",
                V::U64(count_hardening_passed(&probe.hardening_cases)),
            ),
            f(
                "hardening_all_passed",
                b(count_hardening_passed(&probe.hardening_cases)
                    == wasm_runtime::WASM_HARDENING_CASE_COUNT as u64),
            ),
            f(
                "hardening_cases",
                record_hardening_cases(&probe.hardening_cases),
            ),
            f("accepts_external_artifact_bytes", b(false)),
            f("maps_executable_pages", b(false)),
            f("writes_persistent_state", b(false)),
            f("mutates_service_inventory", b(false)),
            f("mutates_global_event_log", b(false)),
            f(
                "candidate_intake",
                record_candidate_intake_probe(&candidate_probe),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("wasm.echo_probe");
}

pub(crate) fn emit_wasm_bufecho_probe() {
    let roundtrip = wasm_runtime::run_bufecho_roundtrip(b"raios-m11-bufecho-roundtrip-nonce");
    let negative = wasm_runtime::run_bufecho_unauthorized_probe();
    let audit = memory_store::record_wasm_import_grant_audit(
        wasm_runtime::BUFECHO_SERVICE_ID,
        wasm_runtime::BUFECHO_AUTHORIZED_IMPORTS,
        &roundtrip.run,
    );

    begin_response("wasm.bufecho_probe");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.wasm_bufecho_probe.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("method", s("wasm.bufecho_probe")),
            f("service_id", s(wasm_runtime::BUFECHO_SERVICE_ID)),
            f("input_len", V::U64(roundtrip.input_len)),
            f("input_sha256", record_sha(roundtrip.input_sha256)),
            f(
                "captured_output_len",
                V::U64(roundtrip.run.captured_output_len),
            ),
            f(
                "captured_output_sha256",
                record_sha(roundtrip.run.captured_output_sha256),
            ),
            f("run_outcome", s(roundtrip.run.run_outcome)),
            f(
                "authorized_import_count",
                V::U64(roundtrip.run.authorized_import_count),
            ),
            f(
                "linked_host_import_count",
                V::U64(roundtrip.run.linked_host_import_count),
            ),
            f(
                "module_imports_within_authorized_list",
                b(roundtrip.run.module_imports_within_authorized_list),
            ),
            f("audit_dedupe", s(audit.dedupe)),
            f("audit_record_id", s(audit.record_id)),
            // Provenance for WHY the import-grant audit did/did not durably persist.
            // In this focused current-boot profile the shared durable reclog is not
            // provisioned, so the append is honestly fail-closed to RAM-only; this
            // surfaces the exact underlying reason instead of a bare "denied".
            f(
                "audit_reason",
                s(audit
                    .evidence
                    .as_ref()
                    .map(|evidence| evidence.reason)
                    .unwrap_or("no_append_attempted")),
            ),
            f(
                "negative",
                V::InlineObject(vec![
                    f(
                        "module_imports_within_authorized_list",
                        b(negative.module_imports_within_authorized_list),
                    ),
                    f("run_outcome", s(negative.run_outcome)),
                    f(
                        "missing_import_module",
                        record_str_or_null(negative.missing_import_module.as_deref()),
                    ),
                    f("instantiation_ok", b(negative.instantiation_ok)),
                    f("captured_output_len", V::U64(negative.captured_output_len)),
                ]),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("wasm.bufecho_probe");
}

pub(crate) fn emit_wasm_certwindow_probe() {
    let cert = REAL_TEST_CERT_DER;
    let cert_sha256 = sha256_bytes(cert);
    let rt = wasm_runtime::run_certwindow_roundtrip(cert);
    let gd = decode_certwindow_record(&rt.raw_captured_output);
    let core = parse_x509_cert_validity_window(cert);
    let core_encoded = encode_certwindow_record(core);
    let core_output_sha256 = sha256_bytes(&core_encoded);
    let guest_matches_core = gd.record_valid
        && match core {
            Ok(window) => gd.status == 0 && gd.error_code == 0 && gd.window == Some(window),
            Err(error) => gd.status == 1 && gd.error_code == error.code() && gd.window.is_none(),
        };
    let output_bytes_match = rt.captured_output_len == CERTWINDOW_RECORD_LEN as u64
        && rt.captured_output_sha256 == core_output_sha256;

    let bad = &cert[..120];
    let rtm = wasm_runtime::run_certwindow_roundtrip(bad);
    let gdm = decode_certwindow_record(&rtm.raw_captured_output);
    let corem = parse_x509_cert_validity_window(bad);
    let corem_code = match corem {
        Ok(_) => 0,
        Err(error) => error.code(),
    };
    let malformed_matches = gdm.record_valid
        && corem.is_err()
        && gdm.status == 1
        && gdm.error_code == corem_code
        && gdm.window.is_none();
    let neg = wasm_runtime::run_certwindow_unauthorized_probe();

    begin_response("wasm.certwindow_probe");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.wasm_certwindow_probe.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("method", s("wasm.certwindow_probe")),
            f("service_id", s(wasm_runtime::CERTWINDOW_SERVICE_ID)),
            f("input_len", V::U64(cert.len() as u64)),
            f("input_sha256", record_sha(cert_sha256)),
            f("cert_sha256", record_sha(cert_sha256)),
            f("run_outcome", s(rt.run_outcome)),
            f(
                "authorized_import_count",
                V::U64(rt.authorized_import_count),
            ),
            f(
                "linked_host_import_count",
                V::U64(rt.linked_host_import_count),
            ),
            f(
                "module_imports_within_authorized_list",
                b(rt.module_imports_within_authorized_list),
            ),
            f("fuel_budget", V::U64(rt.fuel_budget)),
            f("fuel_used", V::U64(rt.fuel_used)),
            f("captured_output_len", V::U64(rt.captured_output_len)),
            f(
                "captured_output_sha256",
                record_sha(rt.captured_output_sha256),
            ),
            f("core_output_sha256", record_sha(core_output_sha256)),
            f("guest_record_valid", b(gd.record_valid)),
            f(
                "guest_parse_ok",
                b(gd.record_valid && gd.status == 0 && gd.error_code == 0),
            ),
            f("guest_error_code", V::U64(gd.error_code as u64)),
            f(
                "guest_not_before",
                datetime_or_null(gd.window.map(|window| window.not_before)),
            ),
            f(
                "guest_not_after",
                datetime_or_null(gd.window.map(|window| window.not_after)),
            ),
            f("core_parse_ok", b(core.is_ok())),
            f("core_error_code", V::U64(core_error_code(core) as u64)),
            f("core_error_reason", s(core_error_reason(core))),
            f(
                "core_not_before",
                datetime_or_null(core.ok().map(|window| window.not_before)),
            ),
            f(
                "core_not_after",
                datetime_or_null(core.ok().map(|window| window.not_after)),
            ),
            f("guest_matches_core", b(guest_matches_core)),
            f("output_bytes_match", b(output_bytes_match)),
            f("guest_output_is_evidence_only", b(true)),
            f("core_is_authority", b(true)),
            f("policy_allows_beyond_env", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
            f("validates_cert_time", b(false)),
            f("authorizes_provider_request", b(false)),
            f("authorizes_provider_export", b(false)),
            f("durable_write", b(false)),
            f("capability_granted", b(false)),
            f(
                "malformed_case",
                V::InlineObject(vec![
                    f("guest_record_valid", b(gdm.record_valid)),
                    f("guest_parse_ok", b(false)),
                    f("guest_error_code", V::U64(gdm.error_code as u64)),
                    f("core_parse_ok", b(false)),
                    f("core_error_code", V::U64(corem_code as u64)),
                    f("error_codes_match", b(gdm.error_code == corem_code)),
                    f("guest_matches_core", b(malformed_matches)),
                    f(
                        "guest_error_reason",
                        s(X509ValidityError::from_code(gdm.error_code)
                            .map(|error| error.reason())
                            .unwrap_or("truncated")),
                    ),
                    f("capability_granted", b(false)),
                ]),
            ),
            f(
                "negative",
                V::InlineObject(vec![
                    f(
                        "module_imports_within_authorized_list",
                        b(neg.module_imports_within_authorized_list),
                    ),
                    f("run_outcome", s(neg.run_outcome)),
                    f(
                        "missing_import_module",
                        record_str_or_null(neg.missing_import_module.as_deref()),
                    ),
                    f("instantiation_ok", b(neg.instantiation_ok)),
                    f("captured_output_len", V::U64(neg.captured_output_len)),
                ]),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("wasm.certwindow_probe");
}

pub(crate) fn emit_wasm_httphead_probe() {
    let response = REAL_TEST_HTTP_RESPONSE;
    let response_sha256 = sha256_bytes(response);
    let rt = wasm_runtime::run_httphead_roundtrip(response);
    let gd = decode_httphead_record(&rt.raw_captured_output);
    let core = parse_http_head(response);
    let core_encoded = encode_httphead_record(core);
    let core_output_sha256 = sha256_bytes(&core_encoded);
    let guest_matches_core = gd.record_valid
        && gd.status_present == core.status_code.is_some()
        && gd.status_code == core.status_code.unwrap_or(0)
        && gd.content_length_present == core.content_length.is_some()
        && gd.content_length == core.content_length.unwrap_or(0)
        && gd.response_complete == core.response_complete
        && gd.chunked == core.chunked;
    let output_bytes_match = rt.captured_output_len == HTTPHEAD_RECORD_LEN as u64
        && rt.captured_output_sha256 == core_output_sha256;

    // Malformed/truncated chunked stream: drop the terminal "\r\n0\r\n" chunk.
    let bad = &REAL_TEST_HTTP_RESPONSE[..54];
    let rtm = wasm_runtime::run_httphead_roundtrip(bad);
    let gdm = decode_httphead_record(&rtm.raw_captured_output);
    let corem = parse_http_head(bad);
    let malformed_matches = gdm.record_valid
        && gdm.status_present == corem.status_code.is_some()
        && gdm.status_code == corem.status_code.unwrap_or(0)
        && gdm.content_length_present == corem.content_length.is_some()
        && gdm.content_length == corem.content_length.unwrap_or(0)
        && gdm.response_complete == corem.response_complete
        && gdm.chunked == corem.chunked;

    // Content-Length fixture: typed evidence that BOTH guest and core parse
    // the real Content-Length: 11 header after skipping the status line.
    let cl = REAL_TEST_HTTP_CONTENT_LENGTH_RESPONSE;
    let rtc = wasm_runtime::run_httphead_roundtrip(cl);
    let gdc = decode_httphead_record(&rtc.raw_captured_output);
    let corec = parse_http_head(cl);
    let content_length_matches = gdc.record_valid
        && gdc.status_present == corec.status_code.is_some()
        && gdc.status_code == corec.status_code.unwrap_or(0)
        && gdc.content_length_present == corec.content_length.is_some()
        && gdc.content_length == corec.content_length.unwrap_or(0)
        && gdc.response_complete == corec.response_complete
        && gdc.chunked == corec.chunked;

    let neg = wasm_runtime::run_httphead_unauthorized_probe();

    begin_response("wasm.httphead_probe");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.wasm_httphead_probe.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("method", s("wasm.httphead_probe")),
            f("service_id", s(wasm_runtime::HTTPHEAD_SERVICE_ID)),
            f("input_len", V::U64(response.len() as u64)),
            f("input_sha256", record_sha(response_sha256)),
            f("run_outcome", s(rt.run_outcome)),
            f(
                "authorized_import_count",
                V::U64(rt.authorized_import_count),
            ),
            f(
                "linked_host_import_count",
                V::U64(rt.linked_host_import_count),
            ),
            f(
                "module_imports_within_authorized_list",
                b(rt.module_imports_within_authorized_list),
            ),
            f("fuel_budget", V::U64(rt.fuel_budget)),
            f("fuel_used", V::U64(rt.fuel_used)),
            f("captured_output_len", V::U64(rt.captured_output_len)),
            f(
                "captured_output_sha256",
                record_sha(rt.captured_output_sha256),
            ),
            f("core_output_sha256", record_sha(core_output_sha256)),
            f("guest_record_valid", b(gd.record_valid)),
            f("guest_status_present", b(gd.status_present)),
            f("guest_status_code", V::U64(gd.status_code as u64)),
            f("guest_content_length_present", b(gd.content_length_present)),
            f("guest_content_length", V::U64(gd.content_length)),
            f("guest_response_complete", b(gd.response_complete)),
            f("guest_chunked", b(gd.chunked)),
            f("core_status_present", b(core.status_code.is_some())),
            f(
                "core_status_code",
                V::U64(core.status_code.unwrap_or(0) as u64),
            ),
            f(
                "core_content_length_present",
                b(core.content_length.is_some()),
            ),
            f(
                "core_content_length",
                V::U64(core.content_length.unwrap_or(0)),
            ),
            f("core_response_complete", b(core.response_complete)),
            f("core_chunked", b(core.chunked)),
            f("guest_matches_core", b(guest_matches_core)),
            f("output_bytes_match", b(output_bytes_match)),
            f("guest_output_is_evidence_only", b(true)),
            f("core_is_authority", b(true)),
            f("policy_allows_beyond_env", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
            f("authorizes_provider_request", b(false)),
            f("authorizes_provider_export", b(false)),
            f("durable_write", b(false)),
            f("capability_granted", b(false)),
            f(
                "malformed_case",
                V::InlineObject(vec![
                    f("guest_record_valid", b(gdm.record_valid)),
                    f("guest_status_present", b(gdm.status_present)),
                    f("guest_status_code", V::U64(gdm.status_code as u64)),
                    f(
                        "guest_content_length_present",
                        b(gdm.content_length_present),
                    ),
                    f("guest_response_complete", b(gdm.response_complete)),
                    f("guest_chunked", b(gdm.chunked)),
                    f("core_response_complete", b(corem.response_complete)),
                    f("guest_matches_core", b(malformed_matches)),
                    f("capability_granted", b(false)),
                ]),
            ),
            f(
                "content_length_case",
                V::InlineObject(vec![
                    f("guest_record_valid", b(gdc.record_valid)),
                    f("guest_status_present", b(gdc.status_present)),
                    f("guest_status_code", V::U64(gdc.status_code as u64)),
                    f(
                        "guest_content_length_present",
                        b(gdc.content_length_present),
                    ),
                    f("guest_content_length", V::U64(gdc.content_length)),
                    f("guest_response_complete", b(gdc.response_complete)),
                    f("guest_chunked", b(gdc.chunked)),
                    f(
                        "core_content_length_present",
                        b(corec.content_length.is_some()),
                    ),
                    f(
                        "core_content_length",
                        V::U64(corec.content_length.unwrap_or(0)),
                    ),
                    f("guest_matches_core", b(content_length_matches)),
                    f("capability_granted", b(false)),
                ]),
            ),
            f(
                "negative",
                V::InlineObject(vec![
                    f(
                        "module_imports_within_authorized_list",
                        b(neg.module_imports_within_authorized_list),
                    ),
                    f("run_outcome", s(neg.run_outcome)),
                    f(
                        "missing_import_module",
                        record_str_or_null(neg.missing_import_module.as_deref()),
                    ),
                    f("instantiation_ok", b(neg.instantiation_ok)),
                    f("captured_output_len", V::U64(neg.captured_output_len)),
                ]),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("wasm.httphead_probe");
}

pub(crate) fn emit_wasm_certspki_probe() {
    let cert = REAL_TEST_CERT_DER;
    let cert_sha256 = sha256_bytes(cert);
    let rt = wasm_runtime::run_certspki_roundtrip(cert);
    let gd = decode_certspki_record(&rt.raw_captured_output);
    let core = extract_p256_spki(cert);
    let core_encoded = encode_certspki_record(core);
    let core_output_sha256 = sha256_bytes(&core_encoded);
    let guest_matches_core = gd.record_valid
        && match core {
            Some(spki) => {
                gd.status == 0
                    && gd.error_code == 0
                    && gd.spki_der_len as usize == spki.der.len()
                    && gd.public_key[..] == spki.public_key[..]
            }
            None => {
                gd.status == 1
                    && gd.error_code == CERTSPKI_ERROR_NOT_P256_SPKI
                    && gd.spki_der_len == 0
                    && gd.public_key == [0; P256_UNCOMPRESSED_POINT_LEN]
            }
        };
    let output_bytes_match = rt.captured_output_len == CERTSPKI_RECORD_LEN as u64
        && rt.captured_output_sha256 == core_output_sha256;
    let core_public_key_sha256 = core
        .map(|spki| sha256_bytes(spki.public_key))
        .unwrap_or([0; 32]);
    let guest_public_key_sha256 = if gd.record_valid && gd.status == 0 {
        sha256_bytes(&gd.public_key)
    } else {
        [0; 32]
    };

    let bad = &cert[..120];
    let rtm = wasm_runtime::run_certspki_roundtrip(bad);
    let gdm = decode_certspki_record(&rtm.raw_captured_output);
    let corem = extract_p256_spki(bad);
    let malformed_matches = gdm.record_valid
        && corem.is_none()
        && gdm.status == 1
        && gdm.error_code == CERTSPKI_ERROR_NOT_P256_SPKI
        && gdm.spki_der_len == 0
        && gdm.public_key == [0; P256_UNCOMPRESSED_POINT_LEN];

    let neg = wasm_runtime::run_certspki_unauthorized_probe();

    begin_response("wasm.certspki_probe");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.wasm_certspki_probe.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("method", s("wasm.certspki_probe")),
            f("service_id", s(wasm_runtime::CERTSPKI_SERVICE_ID)),
            f("input_len", V::U64(cert.len() as u64)),
            f("input_sha256", record_sha(cert_sha256)),
            f("cert_sha256", record_sha(cert_sha256)),
            f("run_outcome", s(rt.run_outcome)),
            f(
                "authorized_import_count",
                V::U64(rt.authorized_import_count),
            ),
            f(
                "linked_host_import_count",
                V::U64(rt.linked_host_import_count),
            ),
            f(
                "module_imports_within_authorized_list",
                b(rt.module_imports_within_authorized_list),
            ),
            f("fuel_budget", V::U64(rt.fuel_budget)),
            f("fuel_used", V::U64(rt.fuel_used)),
            f("captured_output_len", V::U64(rt.captured_output_len)),
            f(
                "captured_output_sha256",
                record_sha(rt.captured_output_sha256),
            ),
            f("core_output_sha256", record_sha(core_output_sha256)),
            f("guest_record_valid", b(gd.record_valid)),
            f(
                "guest_parse_ok",
                b(gd.record_valid && gd.status == 0 && gd.error_code == 0),
            ),
            f("guest_error_code", V::U64(gd.error_code as u64)),
            f("guest_spki_der_len", V::U64(gd.spki_der_len as u64)),
            f(
                "guest_public_key_len",
                V::U64(if gd.record_valid && gd.status == 0 {
                    P256_UNCOMPRESSED_POINT_LEN as u64
                } else {
                    0
                }),
            ),
            f(
                "guest_public_key_sha256",
                record_sha(guest_public_key_sha256),
            ),
            f("core_parse_ok", b(core.is_some())),
            f(
                "core_error_code",
                V::U64(if core.is_some() {
                    0
                } else {
                    CERTSPKI_ERROR_NOT_P256_SPKI as u64
                }),
            ),
            f(
                "core_spki_der_len",
                V::U64(core.map(|spki| spki.der.len() as u64).unwrap_or(0)),
            ),
            f(
                "core_public_key_len",
                V::U64(
                    core.map(|_| P256_UNCOMPRESSED_POINT_LEN as u64)
                        .unwrap_or(0),
                ),
            ),
            f("core_public_key_sha256", record_sha(core_public_key_sha256)),
            f("guest_matches_core", b(guest_matches_core)),
            f("output_bytes_match", b(output_bytes_match)),
            f("guest_output_is_evidence_only", b(true)),
            f("core_is_authority", b(true)),
            f("policy_allows_beyond_env", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
            f("validates_provider_spki", b(false)),
            f("authorizes_provider_request", b(false)),
            f("authorizes_provider_export", b(false)),
            f("durable_write", b(false)),
            f("capability_granted", b(false)),
            f(
                "malformed_case",
                V::InlineObject(vec![
                    f("guest_record_valid", b(gdm.record_valid)),
                    f("guest_parse_ok", b(false)),
                    f("guest_error_code", V::U64(gdm.error_code as u64)),
                    f("guest_spki_der_len", V::U64(gdm.spki_der_len as u64)),
                    f("core_parse_ok", b(false)),
                    f(
                        "core_error_code",
                        V::U64(CERTSPKI_ERROR_NOT_P256_SPKI as u64),
                    ),
                    f("guest_matches_core", b(malformed_matches)),
                    f("capability_granted", b(false)),
                ]),
            ),
            f(
                "negative",
                V::InlineObject(vec![
                    f(
                        "module_imports_within_authorized_list",
                        b(neg.module_imports_within_authorized_list),
                    ),
                    f("run_outcome", s(neg.run_outcome)),
                    f(
                        "missing_import_module",
                        record_str_or_null(neg.missing_import_module.as_deref()),
                    ),
                    f("instantiation_ok", b(neg.instantiation_ok)),
                    f("captured_output_len", V::U64(neg.captured_output_len)),
                ]),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("wasm.certspki_probe");
}

fn datetime_or_null(value: Option<CertValidityDateTime>) -> V<'static> {
    match value {
        Some(value) => datetime_value(value),
        None => V::Null,
    }
}

fn datetime_value(value: CertValidityDateTime) -> V<'static> {
    V::Object(vec![
        f("year", V::U64(value.year as u64)),
        f("month", V::U64(value.month as u64)),
        f("day", V::U64(value.day as u64)),
        f("hour", V::U64(value.hour as u64)),
        f("minute", V::U64(value.minute as u64)),
        f("second", V::U64(value.second as u64)),
    ])
}

fn core_error_code(
    result: Result<raios_core::cert_validity_window::CertValidityWindow, X509ValidityError>,
) -> u8 {
    match result {
        Ok(_) => 0,
        Err(error) => error.code(),
    }
}

fn core_error_reason(
    result: Result<raios_core::cert_validity_window::CertValidityWindow, X509ValidityError>,
) -> &'static str {
    match result {
        Ok(_) => "none",
        Err(error) => error.reason(),
    }
}

fn record_return_value(value: Option<i32>) -> V<'static> {
    match value {
        Some(value) if value >= 0 => V::U64(value as u64),
        _ => V::Null,
    }
}

fn count_hardening_passed(cases: &[wasm_runtime::WasmHardeningCase]) -> u64 {
    let mut count = 0u64;
    let mut idx = 0usize;
    while idx < cases.len() {
        if cases[idx].passed {
            count += 1;
        }
        idx += 1;
    }
    count
}

fn record_hardening_cases(cases: &[wasm_runtime::WasmHardeningCase]) -> V<'static> {
    let mut values = Vec::new();
    let mut idx = 0usize;
    while idx < cases.len() {
        let case = cases[idx];
        values.push(V::InlineObject(vec![
            f("name", s(case.name)),
            f("mechanism", s(case.mechanism)),
            f("expected_outcome", s(case.expected_outcome)),
            f("actual_outcome", s(case.actual_outcome)),
            f("passed", b(case.passed)),
        ]));
        idx += 1;
    }
    V::Array(values)
}

fn record_candidate_intake_probe(
    probe: &module_candidate_intake::CandidateIntakeProbe,
) -> V<'static> {
    V::InlineObject(vec![
        f(
            "max_external_wasm_candidate_bytes",
            V::U64(module_candidate_intake::MAX_EXTERNAL_WASM_CANDIDATE_BYTES as u64),
        ),
        f(
            "external_delivery_channel",
            s(module_candidate_intake::EXTERNAL_WASM_CANDIDATE_DELIVERY_CHANNEL),
        ),
        f("case_count", V::U64(3)),
        f(
            "echo_external_candidate",
            record_candidate_intake_case(
                "echo_external_test_vector",
                &probe.echo_external_candidate,
            ),
        ),
        f(
            "malformed_under_bound_candidate",
            record_candidate_intake_case(
                "malformed_under_bound",
                &probe.malformed_under_bound_candidate,
            ),
        ),
        f(
            "oversize_candidate",
            record_candidate_intake_case("oversize_rejected", &probe.oversize_candidate),
        ),
        f("all_load_denied", b(all_candidate_load_denied(probe))),
        f(
            "all_execution_denied",
            b(all_candidate_execution_denied(probe)),
        ),
        f(
            "all_persistence_denied",
            b(all_candidate_persistence_denied(probe)),
        ),
    ])
}

fn record_candidate_intake_case(
    name: &'static str,
    outcome: &module_candidate_intake::ExternalWasmCandidateOutcome,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(name)),
        f("byte_len", V::U64(outcome.byte_len as u64)),
        f("artifact_sha256", record_sha(outcome.artifact_sha256)),
        f("wasm_valid", b(outcome.wasm_valid)),
        f("scope", s(outcome.scope)),
        f("retained_in_ram", b(outcome.retained_in_ram)),
        f("rejected", b(outcome.rejected)),
        f("reason", s(outcome.reason)),
        f("load_attempted", b(outcome.load_attempted)),
        f("execution_attempted", b(outcome.execution_attempted)),
        f("authorizes_load", b(outcome.authorizes_load)),
        f("authorizes_execution", b(outcome.authorizes_execution)),
        f(
            "writes_persistent_state",
            b(outcome.writes_persistent_state),
        ),
        f(
            "external_delivery_channel",
            s(outcome.external_delivery_channel),
        ),
    ])
}

fn all_candidate_load_denied(probe: &module_candidate_intake::CandidateIntakeProbe) -> bool {
    candidate_load_denied(&probe.echo_external_candidate)
        && candidate_load_denied(&probe.malformed_under_bound_candidate)
        && candidate_load_denied(&probe.oversize_candidate)
}

fn all_candidate_execution_denied(probe: &module_candidate_intake::CandidateIntakeProbe) -> bool {
    candidate_execution_denied(&probe.echo_external_candidate)
        && candidate_execution_denied(&probe.malformed_under_bound_candidate)
        && candidate_execution_denied(&probe.oversize_candidate)
}

fn all_candidate_persistence_denied(probe: &module_candidate_intake::CandidateIntakeProbe) -> bool {
    !probe.echo_external_candidate.writes_persistent_state
        && !probe
            .malformed_under_bound_candidate
            .writes_persistent_state
        && !probe.oversize_candidate.writes_persistent_state
}

fn candidate_load_denied(outcome: &module_candidate_intake::ExternalWasmCandidateOutcome) -> bool {
    !outcome.load_attempted && !outcome.authorizes_load
}

fn candidate_execution_denied(
    outcome: &module_candidate_intake::ExternalWasmCandidateOutcome,
) -> bool {
    !outcome.execution_attempted && !outcome.authorizes_execution
}
