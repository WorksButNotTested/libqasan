use {
    crate::hooks::asan_load,
    core::{
        ffi::{c_char, c_int, c_void},
        ptr::null_mut,
        slice::from_raw_parts,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strrchr"]
pub unsafe extern "C" fn strrchr(cs: *const c_char, c: c_int) -> *mut c_char {
    trace!("strrchr - cs: {:p}, c: {:#x}", cs, c);

    if cs.is_null() {
        panic!("strrchr - cs is null");
    }

    let mut len = 0;
    while *cs.add(len) != 0 {
        len += 1;
    }
    asan_load(cs as *const c_void, len + 1);
    let cs_slice = from_raw_parts(cs, len);
    let pos = cs_slice.iter().rev().position(|&x| x as c_int == c);
    match pos {
        Some(pos) => cs.add(pos) as *mut c_char,
        None => null_mut(),
    }
}
