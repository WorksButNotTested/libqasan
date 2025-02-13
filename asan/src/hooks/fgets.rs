use {
    crate::{
        hooks::{asan_load, asan_store, asan_sym},
        symbols::{AtomicGuestAddr, Function, FunctionPointer},
    },
    core::ffi::{c_char, c_int, c_void},
    libc::FILE,
    log::trace,
};

#[derive(Debug)]
struct FunctionFgets;

impl Function for FunctionFgets {
    type Func = unsafe extern "C" fn(buf: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char;
    const NAME: &'static str = "fgets\0";
}

static FGETS_ADDR: AtomicGuestAddr = AtomicGuestAddr::new();

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_fgets")]
pub unsafe extern "C" fn fgets(buf: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char {
    trace!("fgets - buf: {:p}, n: {:#x}, stream: {:p}", buf, n, stream);

    if buf.is_null() && n != 0 {
        panic!("fgets - buf is null");
    }

    asan_store(buf as *const c_void, n as usize);
    asan_load(stream as *const c_void, size_of::<FILE>());
    let addr =
        FGETS_ADDR.get_or_insert_with(|| asan_sym(FunctionFgets::NAME.as_ptr() as *const c_char));
    let fgets = FunctionFgets::as_ptr(addr).unwrap();
    fgets(buf, n, stream)
}
