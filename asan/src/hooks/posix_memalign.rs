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

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_posix_memalign")]
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

    if memptr.is_null() {
        panic!("posix_memalign - memptr is null");
    }

    fn is_power_of_two(n: size_t) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }

    if align % size_of::<GuestAddr>() != 0 {
        panic!(
            "posix_memalign - align is not a multiple of {}",
            size_of::<GuestAddr>()
        );
    } else if !is_power_of_two(align) {
        panic!("posix_memalign - align is not a power of two");
    } else {
        let p = asan_alloc(size, align);
        *memptr = p;
        0
    }
}
