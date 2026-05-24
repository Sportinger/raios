    Send-AgentCommand -Command "agent provider.context_gate_selftest provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_gate_selftest"
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_schema" -Needle '"schema": "raios.provider_context_gate_negative_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_test_infrastructure" -Needle '"test_infrastructure": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_global_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_request_envelope" -Needle '"creates_provider_request_envelope": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_positive_bindings" -Needle '"creates_positive_binding_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_injection_disabled" -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_body_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_count" -Needle '"case_count": 16' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_stale_request" -Needle '"case": "stale_dropped_request_binding_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_stale_envelope" -Needle '"case": "stale_dropped_envelope_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_previous_boot" -Needle '"case": "previous_boot_or_unretained_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_denial_schema" -Needle '"case": "denial_schema_substitution"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_positive_substitution" -Needle '"case": "positive_record_substitution"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_body_hash" -Needle '"case": "request_body_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_envelope_hash" -Needle '"case": "request_envelope_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_binding_hash" -Needle '"case": "request_binding_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_packet_hash" -Needle '"case": "provider_minimal_packet_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_exported_hash" -Needle '"case": "exported_field_list_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_omitted_hash" -Needle '"case": "omitted_field_list_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_stale_reason" -Needle '"actual_reason": "binding_stale_or_dropped_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_denial_reason" -Needle '"actual_reason": "binding_denied_schema_or_wrong_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_context_hash_reason" -Needle '"actual_reason": "binding_provider_minimal_packet_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_gate_false" -Needle '"satisfies_current_boot_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_can_export_false" -Needle '"can_export": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent provider.context_injection_gate provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_injection_gate"
    Assert-LogContains -Name "protocol:provider_context_injection_gate_schema" -Needle '"schema": "raios.provider_context_injection_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_export_disabled" -Needle '"provider_export": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_injection_disabled" -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_no_body_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_final_schema" -Needle '"final_authorization_schema": "raios.provider_context_injection_authorization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_final_missing" -Needle '"final_authorization": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_body_check_not_attempted" -Needle '"final_prewrite_body_check": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_can_attach_false" -Needle '"can_attach_context": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_current_boot_gate_false" -Needle '"satisfies_current_boot_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_blocked_final" -Needle '"reason": "final_injection_authorization_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_blocked_disabled" -Needle '"reason": "automatic_context_injection_disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_required_authorization" -Needle '"raios.provider_context_injection_authorization.v0"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent provider.context_injection_gate_selftest provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_injection_gate_selftest"
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_schema" -Needle '"schema": "raios.provider_context_injection_gate_negative_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_test_infrastructure" -Needle '"test_infrastructure": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_global_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_request_envelope" -Needle '"creates_provider_request_envelope": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_positive_bindings" -Needle '"creates_positive_binding_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_final_records" -Needle '"creates_final_authorization_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_injection_disabled" -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_can_attach_false" -Needle '"can_attach_context": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_count" -Needle '"case_count": 7' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_missing" -Needle '"case": "missing_final_authorization"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_stale" -Needle '"case": "stale_dropped_final_authorization_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_schema_substitution" -Needle '"case": "final_authorization_schema_substitution"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_positive_substitution" -Needle '"case": "substituted_positive_final_authorization_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_body_hash" -Needle '"case": "final_authorization_body_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_trust_downgrade" -Needle '"case": "final_authorization_trust_downgrade"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_body_attachment_attempt" -Needle '"case": "body_attachment_without_final_authorization"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_missing_reason" -Needle '"actual_reason": "final_injection_authorization_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_stale_reason" -Needle '"actual_reason": "final_injection_authorization_stale_or_dropped_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_schema_reason" -Needle '"actual_reason": "final_injection_authorization_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_substitution_reason" -Needle '"actual_reason": "final_injection_authorization_substituted_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_body_hash_reason" -Needle '"actual_reason": "final_prewrite_body_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_trust_reason" -Needle '"actual_reason": "final_provider_trust_downgraded_before_write"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_attachment_reason" -Needle '"actual_reason": "body_attachment_without_final_authorization"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.query" -ExpectedMarker "RAIOS_AGENT_END memory.query"
    Assert-LogContains -Name "protocol:memory_query_schema" -Needle '"schema": "memory.query.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_query_snapshot_record" -Needle "snapshot.current" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_query_projection_record" -Needle "snapshot.current.provider_minimal" -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.trace snapshot.current" -ExpectedMarker "RAIOS_AGENT_END memory.trace"
    Assert-LogContains -Name "protocol:memory_trace_schema" -Needle '"schema": "memory.trace.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_trace_snapshot_source" -Needle '"source_method": "system.snapshot"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.recent_events" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
    Assert-LogContains -Name "protocol:memory_recent_events_schema" -Needle '"schema": "event.log.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_record_schema" -Needle '"record_schema": "audit.event.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_scope" -Needle '"scope": "current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_bounded" -Needle '"bounded": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_record_id" -Needle '"id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_sequence" -Needle '"sequence":' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_classification" -Needle '"classification": "public"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_read_outcome" -Needle '"outcome": "response"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_read_kind" -Needle '"kind": "agent_protocol.read_response"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_snapshot_source" -Needle '"source_method": "system.snapshot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_source" -Needle '"source_method": "provider.context_export"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_capability" -Needle '"requested_capability": "cap.provider.context_export"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_risk" -Needle '"risk": "export"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_request_binding_denied_kind" -Needle '"kind": "provider_context_export.request_binding_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_request_binding_denied_outcome" -Needle '"outcome": "denied_not_bound"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_audit_kind" -Needle '"kind": "provider_context_export.denial_audit"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_audit_outcome" -Needle '"outcome": "denied_no_provider_write"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_write_not_attempted" -Needle "provider_write_not_attempted" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_evidence" -Needle '"evidence":' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_request_denial_bindings" -Needle '"bindings": {"schema": "raios.provider_request_binding_denial.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_export_denial_bindings" -Needle '"bindings": {"schema": "raios.provider_context_export_denial_audit.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_binding_gate_false" -Needle '"satisfies_current_boot_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_binding_packet_hash" -Needle '"hashes": {"packet_canonicalization": "raios.provider_minimal.packet.canonical.v0", "projected_packet_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_binding_exported_fields_hash" -Needle '"exported_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_binding_omitted_fields_hash" -Needle '"omitted_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogDoesNotContain -Name "protocol:no_positive_request_binding_schema" -Needle '"schema": "raios.provider_request_binding.v0"'
    Assert-LogDoesNotContain -Name "protocol:no_positive_export_audit_binding_schema" -Needle '"schema": "raios.provider_context_export_audit_binding.v0"'
    Assert-LogDoesNotContain -Name "protocol:no_positive_current_boot_export_gate" -Needle '"satisfies_current_boot_export_gate": true'
    Assert-LogDoesNotContain -Name "protocol:no_positive_export_authorization" -Needle '"positive_export_authorization": true'
    Assert-LogContains -Name "protocol:memory_recent_events_ram_only" -Needle '"persistence": "none"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.record_observation" -ExpectedMarker "RAIOS_AGENT_END memory.record_observation"
    Assert-LogContains -Name "policy:memory_record_observation_method" -Needle '"method": "memory.record_observation"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_record_observation_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_record_observation_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_record_observation_audit_event_id" -Needle '"audit_event_id": "event.current_boot.' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent audit.events 8" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
    Assert-LogContains -Name "protocol:audit_events_alias_schema" -Needle '"schema": "event.log.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_limit" -Needle '"limit": 8' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_denied_outcome" -Needle '"outcome": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_denied_kind" -Needle '"kind": "agent_protocol.capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_denied_source" -Needle '"source_method": "memory.record_observation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_denied_capability" -Needle '"requested_capability": "cap.memory.mutate"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.propose_policy" -ExpectedMarker "RAIOS_AGENT_END memory.propose_policy"
    Assert-LogContains -Name "policy:memory_propose_policy_method" -Needle '"method": "memory.propose_policy"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.supersede_fact" -ExpectedMarker "RAIOS_AGENT_END memory.supersede_fact"
    Assert-LogContains -Name "policy:memory_supersede_fact_method" -Needle '"method": "memory.supersede_fact"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.redact" -ExpectedMarker "RAIOS_AGENT_END memory.redact"
    Assert-LogContains -Name "policy:memory_redact_method" -Needle '"method": "memory.redact"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.compact" -ExpectedMarker "RAIOS_AGENT_END memory.compact"
    Assert-LogContains -Name "policy:memory_compact_method" -Needle '"method": "memory.compact"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_audit_required" -Needle "raios.audit_record.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_persistence_required" -Needle "raios.memory_persistence.v0" -TimeoutSeconds 1
