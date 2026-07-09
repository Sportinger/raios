pub const TPM2_ACPI_HEADER_LEN: usize = 36;
const TPM2_PLATFORM_CLASS_OFFSET: usize = TPM2_ACPI_HEADER_LEN;
const TPM2_CONTROL_AREA_OFFSET: usize = TPM2_ACPI_HEADER_LEN + 4;
const TPM2_START_METHOD_OFFSET: usize = TPM2_ACPI_HEADER_LEN + 12;
const TPM2_COMMON_INTERFACE_LEN: usize = TPM2_ACPI_HEADER_LEN + 16;

pub const TPM2_INTERFACE_KIND_NONE: &str = "none";
pub const TPM2_INTERFACE_KIND_NOT_SET: &str = "not_set";
pub const TPM2_INTERFACE_KIND_ACPI_START_METHOD: &str = "acpi_start_method";
pub const TPM2_INTERFACE_KIND_TIS_MMIO_CANCEL: &str = "tis_mmio_cancel";
pub const TPM2_INTERFACE_KIND_CRB: &str = "crb";
pub const TPM2_INTERFACE_KIND_CRB_ACPI_START: &str = "crb_acpi_start";
pub const TPM2_INTERFACE_KIND_CRB_ARM_SMC_HVC: &str = "crb_arm_smc_hvc";
pub const TPM2_INTERFACE_KIND_FIFO_I2C: &str = "fifo_i2c";
pub const TPM2_INTERFACE_KIND_CRB_AMD_MAILBOX: &str = "crb_amd_mailbox";
pub const TPM2_INTERFACE_KIND_FUTURE_MMIO: &str = "future_mmio_reserved";
pub const TPM2_INTERFACE_KIND_CRB_ARM_FFA: &str = "crb_arm_ffa";
pub const TPM2_INTERFACE_KIND_VENDOR_RESERVED: &str = "vendor_or_legacy_reserved";
pub const TPM2_INTERFACE_KIND_FUTURE_RESERVED: &str = "future_reserved";

pub const TPM2_INTERFACE_STATUS_NOT_PROBED: &str = "not_probed";
pub const TPM2_INTERFACE_STATUS_ABSENT: &str = "tpm2_acpi_absent";
pub const TPM2_INTERFACE_STATUS_TABLE_TOO_SHORT: &str = "tpm2_acpi_table_too_short";
pub const TPM2_INTERFACE_STATUS_CRB_DETAILS_PARSED: &str =
    "tpm2_crb_interface_details_parsed_status_read_not_attempted";
pub const TPM2_INTERFACE_STATUS_TIS_DETAILS_PARSED: &str =
    "tpm2_tis_interface_details_parsed_status_read_not_attempted";
pub const TPM2_INTERFACE_STATUS_TIS_FIXED_BASE_NOT_MAPPED: &str = "tpm2_tis_fixed_base_not_mapped";
pub const TPM2_INTERFACE_STATUS_CONTROL_AREA_MISSING: &str = "tpm2_control_area_missing";
pub const TPM2_INTERFACE_STATUS_START_METHOD_UNSUPPORTED: &str =
    "tpm2_start_method_not_supported_yet";

pub const TPM2_INTERFACE_REASON_TABLE_TOO_SHORT: &str =
    "TPM2 ACPI table shorter than common interface fields";
pub const TPM2_INTERFACE_REASON_CRB_PARSED: &str =
    "TPM2 CRB control area discovered; MMIO status read awaits read-only register slice";
pub const TPM2_INTERFACE_REASON_TIS_PARSED: &str =
    "TPM2 TIS/FIFO base discovered; MMIO status read awaits read-only register slice";
pub const TPM2_INTERFACE_REASON_TIS_FIXED_BASE: &str =
    "TPM2 TIS/FIFO fixed base not mapped by this slice";
pub const TPM2_INTERFACE_REASON_CONTROL_AREA_MISSING: &str =
    "TPM2 start method requires a nonzero control area for this probe";
pub const TPM2_INTERFACE_REASON_START_METHOD_UNSUPPORTED: &str =
    "TPM2 start method parsed but not supported by this read-only probe";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tpm2AcpiDetails {
    pub valid: bool,
    pub platform_class: u16,
    pub control_area: u64,
    pub start_method: u32,
    pub interface_kind: &'static str,
    pub status_probe_performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
}

pub const fn absent_tpm2_acpi_details(
    reason: &'static str,
    status_probe_performed: bool,
) -> Tpm2AcpiDetails {
    Tpm2AcpiDetails {
        valid: false,
        platform_class: 0,
        control_area: 0,
        start_method: 0,
        interface_kind: TPM2_INTERFACE_KIND_NONE,
        status_probe_performed,
        status: TPM2_INTERFACE_STATUS_ABSENT,
        reason,
    }
}

pub const fn tpm2_acpi_details_too_short() -> Tpm2AcpiDetails {
    Tpm2AcpiDetails {
        valid: false,
        platform_class: 0,
        control_area: 0,
        start_method: 0,
        interface_kind: TPM2_INTERFACE_KIND_NONE,
        status_probe_performed: true,
        status: TPM2_INTERFACE_STATUS_TABLE_TOO_SHORT,
        reason: TPM2_INTERFACE_REASON_TABLE_TOO_SHORT,
    }
}

pub fn parse_tpm2_acpi_details(table: &[u8]) -> Tpm2AcpiDetails {
    if table.len() < TPM2_COMMON_INTERFACE_LEN {
        return tpm2_acpi_details_too_short();
    }

    let platform_class = read_le_u16(table, TPM2_PLATFORM_CLASS_OFFSET).unwrap_or(0);
    let control_area = read_le_u64(table, TPM2_CONTROL_AREA_OFFSET).unwrap_or(0);
    let start_method = read_le_u32(table, TPM2_START_METHOD_OFFSET).unwrap_or(0);
    let interface_kind = tpm2_start_method_kind(start_method);
    let (status, reason) = tpm2_interface_status(start_method, control_area);

    Tpm2AcpiDetails {
        valid: true,
        platform_class,
        control_area,
        start_method,
        interface_kind,
        status_probe_performed: true,
        status,
        reason,
    }
}

pub const fn tpm2_start_method_kind(start_method: u32) -> &'static str {
    match start_method {
        0 => TPM2_INTERFACE_KIND_NOT_SET,
        1 | 3..=5 | 9..=10 => TPM2_INTERFACE_KIND_VENDOR_RESERVED,
        2 => TPM2_INTERFACE_KIND_ACPI_START_METHOD,
        6 => TPM2_INTERFACE_KIND_TIS_MMIO_CANCEL,
        7 => TPM2_INTERFACE_KIND_CRB,
        8 => TPM2_INTERFACE_KIND_CRB_ACPI_START,
        11 => TPM2_INTERFACE_KIND_CRB_ARM_SMC_HVC,
        12 => TPM2_INTERFACE_KIND_FIFO_I2C,
        13 => TPM2_INTERFACE_KIND_CRB_AMD_MAILBOX,
        14 => TPM2_INTERFACE_KIND_FUTURE_MMIO,
        15 => TPM2_INTERFACE_KIND_CRB_ARM_FFA,
        _ => TPM2_INTERFACE_KIND_FUTURE_RESERVED,
    }
}

pub const fn tpm2_interface_status(
    start_method: u32,
    control_area: u64,
) -> (&'static str, &'static str) {
    match start_method {
        7 | 8 => {
            if control_area == 0 {
                (
                    TPM2_INTERFACE_STATUS_CONTROL_AREA_MISSING,
                    TPM2_INTERFACE_REASON_CONTROL_AREA_MISSING,
                )
            } else {
                (
                    TPM2_INTERFACE_STATUS_CRB_DETAILS_PARSED,
                    TPM2_INTERFACE_REASON_CRB_PARSED,
                )
            }
        }
        6 => {
            if control_area == 0 {
                (
                    TPM2_INTERFACE_STATUS_TIS_FIXED_BASE_NOT_MAPPED,
                    TPM2_INTERFACE_REASON_TIS_FIXED_BASE,
                )
            } else {
                (
                    TPM2_INTERFACE_STATUS_TIS_DETAILS_PARSED,
                    TPM2_INTERFACE_REASON_TIS_PARSED,
                )
            }
        }
        _ => (
            TPM2_INTERFACE_STATUS_START_METHOD_UNSUPPORTED,
            TPM2_INTERFACE_REASON_START_METHOD_UNSUPPORTED,
        ),
    }
}

fn read_le_u16(data: &[u8], offset: usize) -> Option<u16> {
    read_le(data, offset, 2).map(|value| value as u16)
}

fn read_le_u32(data: &[u8], offset: usize) -> Option<u32> {
    read_le(data, offset, 4).map(|value| value as u32)
}

fn read_le_u64(data: &[u8], offset: usize) -> Option<u64> {
    read_le(data, offset, 8)
}

fn read_le(data: &[u8], offset: usize, width: usize) -> Option<u64> {
    let mut value = 0u64;
    let mut idx = 0usize;
    while idx < width {
        let byte = *data.get(offset.checked_add(idx)?)?;
        value |= (byte as u64) << ((idx * 8) as u32);
        idx += 1;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTROL_AREA: u64 = 0xfed4_0000;

    #[test]
    fn parses_crb_common_fields() {
        let table = table_with(0, CONTROL_AREA, 7);
        let details = parse_tpm2_acpi_details(&table);

        assert!(details.valid);
        assert_eq!(details.platform_class, 0);
        assert_eq!(details.control_area, CONTROL_AREA);
        assert_eq!(details.start_method, 7);
        assert_eq!(details.interface_kind, TPM2_INTERFACE_KIND_CRB);
        assert_eq!(details.status, TPM2_INTERFACE_STATUS_CRB_DETAILS_PARSED);
        assert!(details.status_probe_performed);
    }

    #[test]
    fn parses_crb_acpi_start_common_fields() {
        let table = table_with(1, CONTROL_AREA, 8);
        let details = parse_tpm2_acpi_details(&table);

        assert_eq!(details.platform_class, 1);
        assert_eq!(details.interface_kind, TPM2_INTERFACE_KIND_CRB_ACPI_START);
        assert_eq!(details.status, TPM2_INTERFACE_STATUS_CRB_DETAILS_PARSED);
    }

    #[test]
    fn reports_missing_crb_control_area() {
        let details = parse_tpm2_acpi_details(&table_with(0, 0, 7));

        assert!(details.valid);
        assert_eq!(details.status, TPM2_INTERFACE_STATUS_CONTROL_AREA_MISSING);
        assert_eq!(details.reason, TPM2_INTERFACE_REASON_CONTROL_AREA_MISSING);
    }

    #[test]
    fn reports_tis_fixed_base_when_fifo_base_is_zero() {
        let details = parse_tpm2_acpi_details(&table_with(0, 0, 6));

        assert_eq!(details.interface_kind, TPM2_INTERFACE_KIND_TIS_MMIO_CANCEL);
        assert_eq!(
            details.status,
            TPM2_INTERFACE_STATUS_TIS_FIXED_BASE_NOT_MAPPED
        );
    }

    #[test]
    fn reports_unsupported_start_method_without_authority() {
        let details = parse_tpm2_acpi_details(&table_with(0, CONTROL_AREA, 13));

        assert_eq!(details.interface_kind, TPM2_INTERFACE_KIND_CRB_AMD_MAILBOX);
        assert_eq!(
            details.status,
            TPM2_INTERFACE_STATUS_START_METHOD_UNSUPPORTED
        );
    }

    #[test]
    fn short_table_fails_closed() {
        let details = parse_tpm2_acpi_details(&[0; TPM2_COMMON_INTERFACE_LEN - 1]);

        assert!(!details.valid);
        assert_eq!(details.interface_kind, TPM2_INTERFACE_KIND_NONE);
        assert_eq!(details.status, TPM2_INTERFACE_STATUS_TABLE_TOO_SHORT);
    }

    #[test]
    fn absent_table_preserves_probe_reason() {
        let details = absent_tpm2_acpi_details("TPM2 ACPI table missing", true);

        assert!(!details.valid);
        assert!(details.status_probe_performed);
        assert_eq!(details.status, TPM2_INTERFACE_STATUS_ABSENT);
        assert_eq!(details.reason, "TPM2 ACPI table missing");
    }

    fn table_with(platform_class: u16, control_area: u64, start_method: u32) -> [u8; 52] {
        let mut table = [0u8; 52];
        table[TPM2_PLATFORM_CLASS_OFFSET..TPM2_PLATFORM_CLASS_OFFSET + 2]
            .copy_from_slice(&platform_class.to_le_bytes());
        table[TPM2_CONTROL_AREA_OFFSET..TPM2_CONTROL_AREA_OFFSET + 8]
            .copy_from_slice(&control_area.to_le_bytes());
        table[TPM2_START_METHOD_OFFSET..TPM2_START_METHOD_OFFSET + 4]
            .copy_from_slice(&start_method.to_le_bytes());
        table
    }
}
