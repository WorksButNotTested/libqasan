use {
    crate::{
        hooks::{asan_sym, asan_untrack, size_t},
        symbols::{AtomicGuestAddr, Function, FunctionPointer},
    },
    core::ffi::{c_char, CStr},
    libc::{c_int, c_void},
    log::trace,
};

#[derive(Debug)]
struct FunctionMunmap;

impl Function for FunctionMunmap {
    type Func = unsafe extern "C" fn(addr: *mut c_void, len: size_t) -> c_int;
    const NAME: &'static CStr = c"munmap";
}

static MUNMAP_ADDR: AtomicGuestAddr = AtomicGuestAddr::new();

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_munmap")]
pub unsafe extern "C" fn munmap(addr: *mut c_void, len: size_t) -> c_int {
    trace!("munmap - addr: {:p}, len: {:#x}", addr, len);
    let mmap_addr =
        MUNMAP_ADDR.get_or_insert_with(|| asan_sym(FunctionMunmap::NAME.as_ptr() as *const c_char));
    let munmap = FunctionMunmap::as_ptr(mmap_addr).unwrap();
    let ret = munmap(addr, len);
    if ret < 0 {
        return ret;
    }

    asan_untrack(addr);
    ret
}
