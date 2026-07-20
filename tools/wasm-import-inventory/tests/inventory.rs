use sha2::{Digest, Sha256};
use wasm_import_inventory::{inventory_from_bytes, ImportKind, ValueType};

fn mixed_import_fixture() -> Vec<u8> {
    wat::parse_str(
        r#"
        (module
            (type $unary (func (param i32) (result i64)))
            (type $pair (func (param f32 f64) (result i32 i64)))
            (import "wasi_snapshot_preview1" "same" (func (type $unary)))
            (import "wasi_snapshot_preview1" "same" (func (type $pair)))
            (import "env" "shared_memory" (memory 2 5 shared))
            (import "env" "counter" (global (mut i64)))
            (import "env" "objects" (table 3 7 externref))
        )
        "#,
    )
    .expect("fixture WAT must compile")
}

#[test]
fn inventories_every_import_in_binary_order_and_preserves_duplicates() {
    let wasm = mixed_import_fixture();
    let inventory = inventory_from_bytes(&wasm).expect("fixture must be inventoried");

    assert_eq!(inventory.file_length(), wasm.len() as u64);
    assert_eq!(inventory.file_sha256().len(), 64);
    assert_eq!(inventory.import_count(), 5);
    assert_eq!(inventory.imports_sha256().len(), 64);

    let imports = inventory.imports();
    assert_eq!(
        (&imports[0].module[..], &imports[0].name[..]),
        ("wasi_snapshot_preview1", "same")
    );
    assert_eq!(
        imports[0].kind,
        ImportKind::Func {
            type_index: 0,
            params: vec![ValueType::I32],
            results: vec![ValueType::I64],
        }
    );
    assert_eq!(
        (&imports[1].module[..], &imports[1].name[..]),
        ("wasi_snapshot_preview1", "same")
    );
    assert_eq!(
        imports[1].kind,
        ImportKind::Func {
            type_index: 1,
            params: vec![ValueType::F32, ValueType::F64],
            results: vec![ValueType::I32, ValueType::I64],
        }
    );
    assert_eq!(
        imports[2].kind,
        ImportKind::Memory {
            initial: 2,
            maximum: Some(5),
            shared: true,
            memory64: false,
        }
    );
    assert_eq!(
        imports[3].kind,
        ImportKind::Global {
            value_type: ValueType::I64,
            mutable: true,
        }
    );
    assert_eq!(
        imports[4].kind,
        ImportKind::Table {
            element_type: ValueType::ExternRef,
            initial: 3,
            maximum: Some(7),
        }
    );
}

#[test]
fn canonical_json_is_byte_identical_across_runs() {
    let wasm = mixed_import_fixture();
    let first_inventory = inventory_from_bytes(&wasm).expect("first inventory must succeed");
    let first = first_inventory.to_canonical_json();
    let second = inventory_from_bytes(&wasm)
        .expect("second inventory must succeed")
        .to_canonical_json();

    assert_eq!(first, second);
    let text = std::str::from_utf8(&first).expect("JSON must be UTF-8");
    assert!(text.contains(
        r#"{"module":"wasi_snapshot_preview1","name":"same","kind":"func","type_index":0,"params":["i32"],"results":["i64"]}"#
    ));
    assert!(!text.contains("unknown"));

    let marker = b"\"imports\":";
    let imports_start = first
        .windows(marker.len())
        .position(|window| window == marker)
        .expect("canonical JSON must contain imports")
        + marker.len();
    let imports_end = first.len() - 2; // Exclude the top-level `}` and final newline.
    let imports_hash = Sha256::digest(&first[imports_start..imports_end]);
    let imports_hash_hex = imports_hash
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(first_inventory.imports_sha256(), imports_hash_hex);
}

#[test]
fn unresolved_function_type_index_is_an_error() {
    let wasm = wat::parse_str(r#"(module (import "m" "f" (func (type 9))))"#)
        .expect("wat emits this intentionally invalid type reference");
    let error = inventory_from_bytes(&wasm).expect_err("missing type must fail closed");
    assert!(error
        .to_string()
        .contains("unresolvable function type index 9"));
}
