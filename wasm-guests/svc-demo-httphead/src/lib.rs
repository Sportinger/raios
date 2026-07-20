#![no_std]

use core::{
    panic::PanicInfo,
    ptr::{addr_of, addr_of_mut},
};

use raios_http_parse as hp;

static mut IN: [u8; 4096] = [0; 4096];
static mut OUT: [u8; hp::HTTPHEAD_RECORD_LEN] = [0; hp::HTTPHEAD_RECORD_LEN];

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
    let n = if raw < 0 { 0 } else { (raw as usize).min(4096) };
    unsafe {
        let in_ptr = addr_of_mut!(IN).cast::<u8>() as i32;
        input_read(in_ptr, n as i32);
        let response: &[u8] = &(*addr_of!(IN))[..n];
        let facts = hp::parse_http_head(response);
        let encoded = hp::encode_httphead_record(facts);
        (*addr_of_mut!(OUT)).copy_from_slice(&encoded);
        let out_ptr = addr_of!(OUT).cast::<u8>() as i32;
        output_write(out_ptr, hp::HTTPHEAD_RECORD_LEN as i32);
    }
    0
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
