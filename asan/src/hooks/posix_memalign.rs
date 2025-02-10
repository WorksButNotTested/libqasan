use {
    crate::{
        hooks::{asan_alloc, size_t},
        GuestAddr,
    },
    core::{
        ffi::{c_int, c_void},
        mem::size_of,
    },
    log::trace,
};

const EINVAL: c_int = -22;

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut c_void,
    align: size_t,
    size: size_t,
) -> c_int {
    trace!(
        "posix_memalign - memptr: {:p}, align: {:#x}, size: {:#x}",
        memptr,
        align,
        size
    );

    fn is_power_of_two(n: size_t) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }

    if memptr.is_null() || size % size_of::<GuestAddr>() != 0 || !is_power_of_two(align) {
        EINVAL
    } else {
        let p = asan_alloc(size, align);
        *memptr = p;
        0
    }
}
