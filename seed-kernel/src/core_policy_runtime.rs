use core::{slice, str};

use limine::response::{ExecutableFileResponse, ModuleResponse};
use raios_core::{
    boot_control::BootPosture,
    core_policy::{
        verify_current_core_policy, verify_safe_last_good_core_policy, CorePolicySlot,
        VerifiedCorePolicy, VerifiedSafeLastGoodCorePolicy, CORE_POLICY_MAX_EXECUTABLE_LEN,
        CORE_POLICY_MODULE_PATH, CORE_POLICY_RECORD_LEN,
    },
    sha256_hex,
};
use spin::Once;

use crate::{agent_protocol::boot_control, serial};

enum RuntimeCorePolicy {
    Current(VerifiedCorePolicy),
    SafeLastGood(VerifiedSafeLastGoodCorePolicy),
}

static VERIFIED_CORE_POLICY: Once<Result<RuntimeCorePolicy, &'static str>> = Once::new();

pub(crate) fn init(
    executable_response: Option<&ExecutableFileResponse>,
    module_response: Option<&ModuleResponse>,
) {
    let result = VERIFIED_CORE_POLICY.call_once(|| verify(executable_response, module_response));
    match result {
        Ok(RuntimeCorePolicy::Current(policy)) => log_verified(policy),
        Ok(RuntimeCorePolicy::SafeLastGood(policy)) => log_safe_last_good_verified(policy),
        Err(reason) => serial::write_fmt(format_args!("CORE_POLICY_DENIED reason={}\r\n", reason)),
    }
}

pub(crate) fn verified() -> Result<&'static VerifiedCorePolicy, &'static str> {
    match VERIFIED_CORE_POLICY.get() {
        Some(Ok(RuntimeCorePolicy::Current(policy))) => Ok(policy),
        Some(Ok(RuntimeCorePolicy::SafeLastGood(_))) => Err("core_policy_safe_last_good_only"),
        Some(Err(reason)) => Err(*reason),
        None => Err("core_policy_not_initialized"),
    }
}

pub(crate) fn safe_last_good_verified(
) -> Result<&'static VerifiedSafeLastGoodCorePolicy, &'static str> {
    match VERIFIED_CORE_POLICY.get() {
        Some(Ok(RuntimeCorePolicy::SafeLastGood(policy))) => Ok(policy),
        Some(Ok(RuntimeCorePolicy::Current(_))) => Err("core_policy_not_safe_last_good"),
        Some(Err(reason)) => Err(*reason),
        None => Err("core_policy_not_initialized"),
    }
}

fn verify(
    executable_response: Option<&ExecutableFileResponse>,
    module_response: Option<&ModuleResponse>,
) -> Result<RuntimeCorePolicy, &'static str> {
    let executable_file = executable_response
        .ok_or("executable_file_response_missing")?
        .file();
    let executable_len = executable_file.size();
    if executable_len == 0 || executable_len > CORE_POLICY_MAX_EXECUTABLE_LEN {
        return Err("executable_file_size_invalid");
    }
    let executable_len =
        usize::try_from(executable_len).map_err(|_| "executable_file_size_invalid")?;
    let executable_addr = executable_file.addr();
    if executable_addr.is_null() {
        return Err("executable_file_address_missing");
    }

    let modules = module_response
        .ok_or("core_policy_module_response_missing")?
        .modules();
    let mut policy_file = None;
    for module in modules {
        if module.path().to_bytes() != CORE_POLICY_MODULE_PATH {
            continue;
        }
        if policy_file.is_some() {
            return Err("core_policy_module_duplicate");
        }
        policy_file = Some(*module);
    }
    let policy_file = policy_file.ok_or("core_policy_module_missing")?;
    if policy_file.size() != CORE_POLICY_RECORD_LEN as u64 {
        return Err("core_policy_module_size_mismatch");
    }
    let policy_addr = policy_file.addr();
    if policy_addr.is_null() {
        return Err("core_policy_module_address_missing");
    }

    // Limine owns both raw-file buffers for the duration of boot. This adapter
    // never mutates them, and sizes are checked before either slice is formed.
    let executable = unsafe { slice::from_raw_parts(executable_addr, executable_len) };
    let policy_blob = unsafe { slice::from_raw_parts(policy_addr, CORE_POLICY_RECORD_LEN) };
    let (_, decision, authoritative_record) = boot_control::current_boot_last_good_view();
    if decision.posture == BootPosture::Safe {
        verify_safe_last_good_core_policy(
            policy_blob,
            executable,
            &decision,
            authoritative_record.as_ref(),
        )
        .map(RuntimeCorePolicy::SafeLastGood)
        .map_err(|denied| denied.reason())
    } else {
        verify_current_core_policy(
            policy_blob,
            executable,
            &decision,
            authoritative_record.as_ref(),
        )
        .map(RuntimeCorePolicy::Current)
        .map_err(|denied| denied.reason())
    }
}

fn log_verified(policy: &VerifiedCorePolicy) {
    let policy_id_hex = sha256_hex(&policy.policy_id_sha256());
    let executable_hex = sha256_hex(&policy.executable_sha256());
    let policy_id_hex = str::from_utf8(&policy_id_hex).unwrap_or("invalid");
    let executable_hex = str::from_utf8(&executable_hex).unwrap_or("invalid");
    let slot = match policy.slot() {
        CorePolicySlot::A => "A",
        CorePolicySlot::B => "B",
    };
    serial::write_fmt(format_args!(
        "CORE_POLICY_OWNER_VERIFIED slot={} generation={} policy_id_sha256=sha256:{} executable_sha256=sha256:{}\r\n",
        slot,
        policy.core_generation(),
        policy_id_hex,
        executable_hex,
    ));
}

fn log_safe_last_good_verified(policy: &VerifiedSafeLastGoodCorePolicy) {
    let policy_id_hex = sha256_hex(&policy.policy_id_sha256());
    let executable_hex = sha256_hex(&policy.executable_sha256());
    let policy_id_hex = str::from_utf8(&policy_id_hex).unwrap_or("invalid");
    let executable_hex = str::from_utf8(&executable_hex).unwrap_or("invalid");
    let slot = match policy.slot() {
        CorePolicySlot::A => "A",
        CorePolicySlot::B => "B",
    };
    serial::write_fmt(format_args!(
        "CORE_POLICY_SAFE_LAST_GOOD_OWNER_VERIFIED slot={} generation={} policy_id_sha256=sha256:{} executable_sha256=sha256:{}\r\n",
        slot,
        policy.core_generation(),
        policy_id_hex,
        executable_hex,
    ));
}
