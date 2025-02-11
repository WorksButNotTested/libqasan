use {
    crate::{
        hooks::{asan_alloc, size_t},
        GuestAddr,
    },
    core::{ffi::c_void, mem::size_of},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_memalign")]
pub unsafe extern "C" fn memalign(align: size_t, size: size_t) -> *mut c_void {
    trace!("memalign - align: {:#x}, size: {:#x}", align, size);
    fn is_power_of_two(n: size_t) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }

    if align % size_of::<GuestAddr>() != 0 {
        panic!("memalign - align is not a multiple of GuestAddr");
    } else if !is_power_of_two(align) {
        panic!("memalign - align is not a power of two");
    } else {
        asan_alloc(size, align)
    }
}
