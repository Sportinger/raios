#![no_std]

use core::{
    panic::PanicInfo,
    ptr::{addr_of, addr_of_mut},
};

use raios_dns_parse as dns;

static mut IN: [u8; 4096] = [0; 4096];
static mut OUT: [u8; dns::DNS_RESPONSE_RECORD_LEN] = [0; dns::DNS_RESPONSE_RECORD_LEN];

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
    let mut encoded = dns::encode_dns_response_record(None);
    let raw = unsafe { input_len() };
    if (0..=4096).contains(&raw) {
        let n = raw as usize;
        unsafe {
            let in_ptr = addr_of_mut!(IN).cast::<u8>() as i32;
            if input_read(in_ptr, raw) == raw {
                let input = &(*addr_of!(IN))[..n];
                if let Some(query) = dns::decode_dns_query_record(input) {
                    encoded = dns::encode_dns_response_record(dns::parse_dns_response(
                        query.payload,
                        query.expected_id,
                        query.hostname,
                    ));
                }
            }
        }
    }
    unsafe {
        (*addr_of_mut!(OUT)).copy_from_slice(&encoded);
        let out_ptr = addr_of!(OUT).cast::<u8>() as i32;
        output_write(out_ptr, dns::DNS_RESPONSE_RECORD_LEN as i32);
    }
    0
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
