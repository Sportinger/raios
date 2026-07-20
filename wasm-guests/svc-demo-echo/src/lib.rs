#![no_std]

use core::panic::PanicInfo;

#[link(wasm_import_module = "env")]
extern "C" {
    #[link_name = "log"]
    fn env_log(ptr: i32, len: i32);
    #[link_name = "counter_get"]
    fn env_counter_get() -> i64;
}

#[no_mangle]
pub extern "C" fn raios_service_main() -> i32 {
    let counter = unsafe { env_counter_get() };
    let mut line = [0u8; 40];
    let mut pos = 0usize;

    push_bytes(&mut line, &mut pos, b"echo counter=");
    push_i64(&mut line, &mut pos, counter);
    push_byte(&mut line, &mut pos, b'\n');

    unsafe { env_log(line.as_ptr() as i32, pos as i32) };
    0
}

fn push_i64(out: &mut [u8], pos: &mut usize, value: i64) {
    if value < 0 {
        push_byte(out, pos, b'-');
    }
    let mut n = value.unsigned_abs();
    let mut digits = [0u8; 20];
    let mut len = 0usize;

    loop {
        digits[len] = b'0' + (n % 10) as u8;
        len += 1;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    while len > 0 {
        len -= 1;
        push_byte(out, pos, digits[len]);
    }
}

fn push_bytes(out: &mut [u8], pos: &mut usize, bytes: &[u8]) {
    for byte in bytes {
        push_byte(out, pos, *byte);
    }
}

fn push_byte(out: &mut [u8], pos: &mut usize, byte: u8) {
    if *pos < out.len() {
        out[*pos] = byte;
        *pos += 1;
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
