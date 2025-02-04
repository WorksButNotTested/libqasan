#![no_std]
use core::{
    ffi::{c_char, c_void},
    panic::PanicInfo,
};

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn dlsym(_handle: *mut c_void, _symbol: *const c_char) -> *mut c_void {
    todo!();
}

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn dlerror() -> *mut c_char {
    todo!();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cfg(target_arch = "arm")]
#[no_mangle]
extern "C" fn __aeabi_unwind_cpp_pr0() {
    loop {}
}
