use {
    crate::hooks::{asan_load, size_t},
    core::{
        ffi::{c_int, c_void},
        ptr::null_mut,
        slice::from_raw_parts,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_memrchr"]
pub unsafe extern "C" fn memrchr(cx: *const c_void, c: c_int, n: size_t) -> *mut c_void {
    trace!("memrchr - cx: {:p}, c: {:#x}, n: {:#x}", cx, c, n);

    if n == 0 {
        return null_mut();
    }

    if cx.is_null() {
        panic!("memrchr - cx is null");
    }

    asan_load(cx, n);
    let slice = from_raw_parts(cx as *const u8, n);
    let pos = slice.iter().rev().position(|&x| x as c_int == c);
    match pos {
        Some(pos) => cx.add(pos) as *mut c_void,
        None => null_mut(),
    }
}
