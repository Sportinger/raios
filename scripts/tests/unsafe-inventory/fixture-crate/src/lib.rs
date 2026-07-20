// SAFETY[unsafe.fixture.first-function] Reason: caller supplies the required proof. Invariant: the documented precondition holds.
pub unsafe fn tagged_function() {
    // SAFETY[unsafe.fixture.block] Reason: fixture controls the pointer. Invariant: pointer remains aligned and live.
    unsafe { core::ptr::read(1usize as *const usize); }
}

// SAFETY[unsafe.fixture.function] Reason: caller supplies the required proof. Invariant: the documented precondition holds.
pub unsafe fn second_function() {}
