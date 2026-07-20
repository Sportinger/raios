use raios_core::wasi_preview1_import_abi::{
    WasiImportKind, WasiValueType, RUSTC_WASM_C6DCCF3E_IMPORTS,
};
use wasm_import_inventory::contract::{
    check_contract, validate_descriptor_json, validate_embedded_descriptor, Floor, DESCRIPTOR_JSON,
};
use wasm_import_inventory::inventory_from_bytes;

fn inventory(wat_source: &str) -> wasm_import_inventory::Inventory {
    let bytes = wat::parse_str(wat_source).expect("fixture WAT must compile");
    inventory_from_bytes(&bytes).expect("fixture must inventory")
}

fn general_floor_wat() -> &'static str {
    r#"(module
      (import "env" "log" (func (param i32 i32)))
      (import "env" "counter_get" (func (result i64)))
      (import "env" "input_len" (func (result i32)))
      (import "env" "input_read" (func (param i32 i32) (result i32)))
      (import "env" "output_write" (func (param i32 i32) (result i32))))"#
}

fn build_floor_wat() -> String {
    let mut source = String::from("(module\n");
    for declaration in RUSTC_WASM_C6DCCF3E_IMPORTS {
        source.push_str(&format!(
            "(import {:?} {:?} ",
            declaration.module, declaration.name
        ));
        match declaration.kind {
            WasiImportKind::Func { params, results } => {
                source.push_str("(func");
                if !params.is_empty() {
                    source.push_str(" (param");
                    for ty in params {
                        source.push(' ');
                        source.push_str(wasi_type(*ty));
                    }
                    source.push(')');
                }
                if !results.is_empty() {
                    source.push_str(" (result");
                    for ty in results {
                        source.push(' ');
                        source.push_str(wasi_type(*ty));
                    }
                    source.push(')');
                }
                source.push(')');
            }
            WasiImportKind::Memory {
                initial,
                maximum: Some(maximum),
                shared,
            } => {
                assert!(shared);
                source.push_str(&format!("(memory {initial} {maximum} shared)"));
            }
            other => panic!("unexpected canonical kind: {other:?}"),
        }
        source.push_str(")\n");
    }
    source.push(')');
    source
}

fn wasi_type(ty: WasiValueType) -> &'static str {
    match ty {
        WasiValueType::I32 => "i32",
        WasiValueType::I64 => "i64",
        WasiValueType::F32 => "f32",
        WasiValueType::F64 => "f64",
        WasiValueType::V128 => "v128",
        WasiValueType::FuncRef => "funcref",
        WasiValueType::ExternRef => "externref",
    }
}

#[test]
fn descriptor_is_exactly_bound_to_both_raios_core_surfaces() {
    validate_embedded_descriptor().expect("embedded descriptor must match raios-core");
}

#[test]
fn documented_general_and_build_floor_imports_are_accepted() {
    let general = check_contract(&inventory(general_floor_wat()), Floor::General);
    assert!(general.accepted());
    assert_eq!(general.reason, "documented_floor_imports_only");
    assert_eq!(
        (general.expected_import_count, general.observed_import_count),
        (5, 5)
    );

    let build = check_contract(&inventory(&build_floor_wat()), Floor::Build);
    assert!(build.accepted());
    assert_eq!(build.reason, "documented_floor_imports_only");
    assert_eq!(
        (build.expected_import_count, build.observed_import_count),
        (30, 30)
    );
}

#[test]
fn undeclared_env_import_is_not_mislabeled_as_kernel_internal() {
    let report = check_contract(
        &inventory(r#"(module (import "env" "not_documented" (func)))"#),
        Floor::General,
    );
    assert_eq!(report.reason, "undeclared_floor_import");
    assert_eq!(report.import_index, Some(0));
}

#[test]
fn reserved_namespace_is_classified_before_unknown_import_handling() {
    let report = check_contract(
        &inventory(r#"(module (import "raios_kernel" "scheduler_task" (global externref)))"#),
        Floor::General,
    );
    assert_eq!(report.reason, "kernel_internal_type_dependency");
}

#[test]
fn non_wire_forms_are_kernel_internal_type_dependencies() {
    for wat in [
        r#"(module (import "env" "log" (func (param f32))))"#,
        r#"(module (import "env" "log" (func (param f64))))"#,
        r#"(module (import "env" "log" (func (param v128))))"#,
        r#"(module (import "env" "log" (global i32)))"#,
        r#"(module (import "env" "log" (table 1 funcref)))"#,
        r#"(module (type $exception (func (param i32))) (import "env" "log" (tag (type $exception))))"#,
        r#"(module (import "env" "memory" (memory i64 1 2)))"#,
    ] {
        let report = check_contract(&inventory(wat), Floor::General);
        assert_eq!(report.reason, "kernel_internal_type_dependency", "{wat}");
    }
}

#[test]
fn descriptor_drift_duplicate_order_signature_kind_and_digest_fail_closed() {
    let original: serde_json::Value = serde_json::from_str(DESCRIPTOR_JSON).unwrap();
    let mutations: Vec<Box<dyn Fn(&mut serde_json::Value)>> = vec![
        Box::new(|value| {
            value["general_floor"]["imports"]
                .as_array_mut()
                .unwrap()
                .swap(0, 1)
        }),
        Box::new(|value| {
            let duplicate = value["general_floor"]["imports"][0].clone();
            value["general_floor"]["imports"].as_array_mut().unwrap()[1] = duplicate;
        }),
        Box::new(|value| value["general_floor"]["imports"][0]["params"][0] = "i64".into()),
        Box::new(|value| value["general_floor"]["imports"][0]["kind"] = "global".into()),
        Box::new(|value| value["build_floor"]["canonical_sha256"] = "00".repeat(32).into()),
    ];
    for mutate in mutations {
        let mut value = original.clone();
        mutate(&mut value);
        assert!(validate_descriptor_json(&value.to_string()).is_err());
    }
}

#[test]
fn structured_summary_exposes_counts_reason_and_verdict() {
    let report = check_contract(&inventory(general_floor_wat()), Floor::General);
    let json = report.to_json();
    println!("{json}");
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["expected_import_count"], 5);
    assert_eq!(value["observed_import_count"], 5);
    assert_eq!(value["verdict"], "accepted");
    assert_eq!(value["reason"], "documented_floor_imports_only");
}
