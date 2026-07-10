use core::{slice, str};

use limine::response::{ExecutableFileResponse, ModuleResponse};
use raios_core::{
    core_policy::{
        verify_current_core_policy, CorePolicySlot, VerifiedCorePolicy,
        CORE_POLICY_MAX_EXECUTABLE_LEN, CORE_POLICY_MODULE_PATH, CORE_POLICY_RECORD_LEN,
    },
    sha256_hex,
};
use spin::Once;

use crate::{agent_protocol::boot_control, serial};

static VERIFIED_CORE_POLICY: Once<Result<VerifiedCorePolicy, &'static str>> = Once::new();

pub(crate) fn init(
    executable_response: Option<&ExecutableFileResponse>,
    module_response: Option<&ModuleResponse>,
) {
    let result = VERIFIED_CORE_POLICY.call_once(|| verify(executable_response, module_response));
    match result {
        Ok(policy) => log_verified(policy),
        Err(reason) => serial::write_fmt(format_args!("CORE_POLICY_DENIED reason={}\r\n", reason)),
    }
}

pub(crate) fn verified() -> Result<&'static VerifiedCorePolicy, &'static str> {
    match VERIFIED_CORE_POLICY.get() {
        Some(Ok(policy)) => Ok(policy),
        Some(Err(reason)) => Err(*reason),
        None => Err("core_policy_not_initialized"),
    }
}

fn verify(
    executable_response: Option<&ExecutableFileResponse>,
    module_response: Option<&ModuleResponse>,
) -> Result<VerifiedCorePolicy, &'static str> {
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
    verify_current_core_policy(
        policy_blob,
        executable,
        &decision,
        authoritative_record.as_ref(),
    )
    .map_err(|denied| denied.reason())
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
