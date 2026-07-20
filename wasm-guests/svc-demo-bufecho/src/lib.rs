#![no_std]

use core::{
    panic::PanicInfo,
    ptr::{addr_of, addr_of_mut},
};

static mut BUF: [u8; 4096] = [0; 4096];

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
    let n = unsafe { input_len() };
    unsafe {
        let in_ptr = addr_of_mut!(BUF).cast::<u8>() as i32;
        let out_ptr = addr_of!(BUF).cast::<u8>() as i32;
        input_read(in_ptr, n);
        output_write(out_ptr, n);
    }
    0
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
