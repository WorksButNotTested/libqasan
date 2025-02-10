use {
    crate::{
        hooks::{asan_alloc, size_t},
        GuestAddr,
    },
    core::{ffi::c_void, mem::size_of, ptr::null_mut},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn memalign(align: size_t, size: size_t) -> *mut c_void {
    trace!("memalign - align: {:#x}, size: {:#x}", align, size);
    fn is_power_of_two(n: size_t) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }

    if size % size_of::<GuestAddr>() != 0 || !is_power_of_two(align) {
        null_mut()
    } else {
        asan_alloc(size, align)
    }
}
