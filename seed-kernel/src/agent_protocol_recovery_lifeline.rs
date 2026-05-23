use crate::agent_protocol_support::method_eq;

pub(crate) const RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_dispatch_denial.current_boot";

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineCommandSpec {
    pub(crate) command_id: &'static str,
    pub(crate) argument_schema: &'static str,
    pub(crate) required_capability: &'static str,
}

pub(crate) fn recovery_lifeline_status_command_spec() -> RecoveryLifelineCommandSpec {
    RecoveryLifelineCommandSpec {
        command_id: "recovery.lifeline.status",
        argument_schema: "raios.recovery_lifeline_command.status_args.v0",
        required_capability: "cap.recovery.load_artifact.read",
    }
}

pub(crate) fn recovery_lifeline_rollback_preview_command_spec() -> RecoveryLifelineCommandSpec {
    RecoveryLifelineCommandSpec {
        command_id: "recovery.lifeline.rollback_preview",
        argument_schema: "raios.recovery_lifeline_command.rollback_preview_args.v0",
        required_capability: "cap.recovery.rollback.read",
    }
}

pub(crate) fn recovery_lifeline_rollback_apply_command_spec() -> RecoveryLifelineCommandSpec {
    RecoveryLifelineCommandSpec {
        command_id: "recovery.lifeline.rollback_apply",
        argument_schema: "raios.recovery_lifeline_command.rollback_apply_args.v0",
        required_capability: "cap.recovery.rollback",
    }
}

pub(crate) fn recovery_lifeline_disable_module_command_spec() -> RecoveryLifelineCommandSpec {
    RecoveryLifelineCommandSpec {
        command_id: "recovery.lifeline.disable_module",
        argument_schema: "raios.recovery_lifeline_command.disable_module_args.v0",
        required_capability: "cap.recovery.module.disable",
    }
}

pub(crate) fn recovery_lifeline_restart_last_good_command_spec() -> RecoveryLifelineCommandSpec {
    RecoveryLifelineCommandSpec {
        command_id: "recovery.lifeline.restart_last_good",
        argument_schema: "raios.recovery_lifeline_command.restart_last_good_args.v0",
        required_capability: "cap.recovery.service.restart",
    }
}

pub(crate) fn recovery_lifeline_load_artifact_by_hash_command_spec() -> RecoveryLifelineCommandSpec
{
    RecoveryLifelineCommandSpec {
        command_id: "recovery.lifeline.load_artifact_by_hash",
        argument_schema: "raios.recovery_lifeline_command.load_artifact_by_hash_args.v0",
        required_capability: "cap.recovery.load_artifact",
    }
}

pub(crate) fn recovery_lifeline_command_spec(
    command_id: &str,
) -> Option<RecoveryLifelineCommandSpec> {
    if method_eq(command_id, "recovery.lifeline.status") {
        Some(recovery_lifeline_status_command_spec())
    } else if method_eq(command_id, "recovery.lifeline.rollback_preview") {
        Some(recovery_lifeline_rollback_preview_command_spec())
    } else if method_eq(command_id, "recovery.lifeline.rollback_apply") {
        Some(recovery_lifeline_rollback_apply_command_spec())
    } else if method_eq(command_id, "recovery.lifeline.disable_module") {
        Some(recovery_lifeline_disable_module_command_spec())
    } else if method_eq(command_id, "recovery.lifeline.restart_last_good") {
        Some(recovery_lifeline_restart_last_good_command_spec())
    } else if method_eq(command_id, "recovery.lifeline.load_artifact_by_hash") {
        Some(recovery_lifeline_load_artifact_by_hash_command_spec())
    } else {
        None
    }
}
