#![no_std]

use core::{
    panic::PanicInfo,
    ptr::{addr_of, addr_of_mut},
};

use raios_wasm_ir::{assemble, ERR_OVERSIZE_SOURCE, MAX_IR_SOURCE_BYTES, MAX_WASM_OUTPUT_BYTES};

const ERROR_PREFIX: &[u8] = b"error:";
const ERROR_INPUT_ABI: &str = "input_abi";

static mut IN: [u8; MAX_IR_SOURCE_BYTES] = [0; MAX_IR_SOURCE_BYTES];
static mut OUT: [u8; MAX_WASM_OUTPUT_BYTES] = [0; MAX_WASM_OUTPUT_BYTES];

#[link(wasm_import_module = "env")]
extern "C" {
    #[link_name = "input_len"]
    fn input_len() -> i32;
    #[link_name = "input_read"]
    fn input_read(ptr: i32, len: i32) -> i32;
    #[link_name = "output_write"]
    fn output_write(ptr: i32, len: i32) -> i32;
}

#[no_mangle]
pub extern "C" fn raios_service_main() -> i32 {
    let raw = unsafe { input_len() };
    if raw > MAX_IR_SOURCE_BYTES as i32 {
        write_error(ERR_OVERSIZE_SOURCE);
        return 0;
    }
    if raw < 0 {
        write_error(ERROR_INPUT_ABI);
        return 0;
    }

    let n = raw as usize;
    unsafe {
        let in_ptr = addr_of_mut!(IN).cast::<u8>() as i32;
        if input_read(in_ptr, raw) != raw {
            write_error(ERROR_INPUT_ABI);
            return 0;
        }
        let input = &(*addr_of!(IN))[..n];
        match assemble(input) {
            Ok(wasm) => write_output(wasm.as_slice()),
            Err(reason) => write_error(reason),
        }
    }
    0
}

fn write_error(reason: &str) {
    unsafe {
        let out = &mut *addr_of_mut!(OUT);
        out[..ERROR_PREFIX.len()].copy_from_slice(ERROR_PREFIX);
        out[ERROR_PREFIX.len()..ERROR_PREFIX.len() + reason.len()]
            .copy_from_slice(reason.as_bytes());
        let out_ptr = addr_of!(OUT).cast::<u8>() as i32;
        output_write(out_ptr, (ERROR_PREFIX.len() + reason.len()) as i32);
    }
}

fn write_output(bytes: &[u8]) {
    unsafe {
        let out = &mut *addr_of_mut!(OUT);
        out[..bytes.len()].copy_from_slice(bytes);
        let out_ptr = addr_of!(OUT).cast::<u8>() as i32;
        output_write(out_ptr, bytes.len() as i32);
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
