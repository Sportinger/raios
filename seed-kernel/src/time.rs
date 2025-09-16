use core::arch::asm;

#[inline]
pub fn rdtsc() -> u64 {
    let hi: u32;
    let lo: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("edx") hi,
            out("eax") lo,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((hi as u64) << 32) | lo as u64
}
