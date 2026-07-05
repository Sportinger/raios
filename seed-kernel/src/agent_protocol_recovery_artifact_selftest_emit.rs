use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_recovery_artifact_types::{
        RecoveryIdentitySelfTestCase, RecoveryLifelineRequestSelfTestCase,
        RecoveryLoaderSelfTestCase, RecoveryLocalApprovalSelfTestCase,
        RecoveryRollbackEvidenceSelfTestCase, RecoveryTrustSelfTestCase,
        RecoveryVmTestSelfTestCase,
    },
    agent_protocol_support::SerialSink,
};
use raios_core::{
    record::{write_json, Field, Value},
    ByteSink,
};

fn emit_case(fields: Vec<Field<'_>>, comma: bool) {
    let value = Value::Object(fields);
    let Value::Object(fields) = &value else {
        return;
    };
    let mut sink = SerialSink;

    sink.write_bytes(b"        {");
    let mut idx = 0usize;
    while idx < fields.len() {
        if idx != 0 {
            sink.write_bytes(b", ");
        }
        write_json(&Value::Str(fields[idx].key), &mut sink, 8);
        sink.write_bytes(b": ");
        write_json(&fields[idx].value, &mut sink, 8);
        idx += 1;
    }
    sink.write_bytes(b"}");
    if comma {
        sink.write_bytes(b",");
    }
    sink.write_bytes(b"\r\n");
}

macro_rules! emit_case_fn {
    ($fn_name:ident, $case_ty:ty, [$($extra:expr),+ $(,)?]) => {
        pub(crate) fn $fn_name(case: &$case_ty, comma: bool) {
            emit_case(
                vec![
                    Field::new("case", Value::Str(case.name)),
                    Field::new("expected_status", Value::Str(case.expected_status)),
                    Field::new("expected_reason", Value::Str(case.expected_reason)),
                    Field::new("actual_status", Value::Str(case.actual_status)),
                    Field::new("actual_reason", Value::Str(case.actual_reason)),
                    Field::new("passed", Value::Bool(case.passed)),
                    $($extra,)+
                ],
                comma,
            );
        }
    };
}

emit_case_fn!(
    emit_recovery_identity_selftest_case,
    RecoveryIdentitySelfTestCase,
    [
        Field::new("authorizes_recovery_load", Value::Bool(false)),
        Field::new("can_move_beyond_denial", Value::Bool(false)),
        Field::new("loads_recovery_artifact", Value::Bool(false)),
        Field::new("load_attempted", Value::Bool(false)),
    ]
);

emit_case_fn!(
    emit_recovery_trust_selftest_case,
    RecoveryTrustSelfTestCase,
    [
        Field::new("authorizes_recovery_load", Value::Bool(false)),
        Field::new("can_move_beyond_denial", Value::Bool(false)),
        Field::new("loads_recovery_artifact", Value::Bool(false)),
        Field::new("load_attempted", Value::Bool(false)),
    ]
);

emit_case_fn!(
    emit_recovery_vm_test_selftest_case,
    RecoveryVmTestSelfTestCase,
    [
        Field::new("authorizes_recovery_load", Value::Bool(false)),
        Field::new("can_move_beyond_denial", Value::Bool(false)),
        Field::new("loads_recovery_artifact", Value::Bool(false)),
        Field::new("load_attempted", Value::Bool(false)),
    ]
);

emit_case_fn!(
    emit_recovery_local_approval_selftest_case,
    RecoveryLocalApprovalSelfTestCase,
    [
        Field::new("authorizes_recovery_load", Value::Bool(false)),
        Field::new("can_move_beyond_denial", Value::Bool(false)),
        Field::new("loads_recovery_artifact", Value::Bool(false)),
        Field::new("load_attempted", Value::Bool(false)),
    ]
);

emit_case_fn!(
    emit_recovery_loader_selftest_case,
    RecoveryLoaderSelfTestCase,
    [
        Field::new("authorizes_recovery_load", Value::Bool(false)),
        Field::new("can_move_beyond_denial", Value::Bool(false)),
        Field::new("loads_recovery_loader", Value::Bool(false)),
        Field::new("loads_recovery_artifact", Value::Bool(false)),
        Field::new("load_attempted", Value::Bool(false)),
    ]
);

emit_case_fn!(
    emit_recovery_rollback_evidence_selftest_case,
    RecoveryRollbackEvidenceSelfTestCase,
    [
        Field::new("authorizes_recovery_load", Value::Bool(false)),
        Field::new("can_move_beyond_denial", Value::Bool(false)),
        Field::new("loads_recovery_artifact", Value::Bool(false)),
        Field::new("creates_durable_records", Value::Bool(false)),
        Field::new("installs_rollback_plan", Value::Bool(false)),
        Field::new("load_attempted", Value::Bool(false)),
    ]
);

emit_case_fn!(
    emit_recovery_lifeline_request_selftest_case,
    RecoveryLifelineRequestSelfTestCase,
    [
        Field::new("authorizes_recovery_load", Value::Bool(false)),
        Field::new("can_move_beyond_denial", Value::Bool(false)),
        Field::new("loads_recovery_loader", Value::Bool(false)),
        Field::new("loads_recovery_artifact", Value::Bool(false)),
        Field::new("creates_durable_records", Value::Bool(false)),
        Field::new("installs_rollback_plan", Value::Bool(false)),
        Field::new("allocates_service_slot", Value::Bool(false)),
        Field::new("service_inventory_change", Value::Str("none")),
        Field::new("load_attempted", Value::Bool(false)),
    ]
);
