use {
    crate::hooks::{asan_load, size_t},
    core::{
        cmp::Ordering,
        ffi::{c_int, c_void},
        slice::from_raw_parts,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_memcmp"]
pub unsafe extern "C" fn memcmp(cx: *const c_void, ct: *const c_void, n: size_t) -> c_int {
    trace!("memcmp - cx: {:p}, ct: {:p}, n: {:#x}", cx, ct, n);

    if n == 0 {
        return 0;
    }

    if cx.is_null() {
        panic!("memcmp - cx is null");
    }

    if ct.is_null() {
        panic!("memcmp - ct is null");
    }

    asan_load(cx, n);
    asan_load(ct, n);

    let slice1 = from_raw_parts(cx as *const u8, n);
    let slice2 = from_raw_parts(ct as *const u8, n);

    for i in 0..n {
        match slice1[i].cmp(&slice2[i]) {
            Ordering::Equal => (),
            Ordering::Less => return -1,
            Ordering::Greater => return 1,
        }
    }

    0
}
