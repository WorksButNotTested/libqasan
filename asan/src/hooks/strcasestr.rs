use {
    crate::hooks::asan_load,
    alloc::vec::Vec,
    core::{
        ffi::{c_char, c_void},
        ptr::null_mut,
        slice::from_raw_parts,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strcasestr"]
pub unsafe extern "C" fn strcasestr(cs: *const c_char, ct: *const c_char) -> *mut c_char {
    trace!("strcasestr - cs: {:p}, ct: {:p}", cs, ct);

    if cs.is_null() {
        panic!("strcasestr - cs is null");
    }

    if ct.is_null() {
        panic!("strcasestr - ct is null");
    }

    let mut cs_len = 0;
    while *cs.add(cs_len) != 0 {
        cs_len += 1;
    }
    let mut ct_len = 0;
    while *ct.add(ct_len) != 0 {
        ct_len += 1;
    }
    asan_load(cs as *const c_void, cs_len + 1);
    asan_load(ct as *const c_void, ct_len + 1);

    if ct_len == 0 {
        return cs as *mut c_char;
    }

    if ct_len > cs_len {
        return null_mut();
    }

    let to_upper = |c: c_char| -> c_char {
        if ('a' as c_char..='z' as c_char).contains(&c) {
            c - 'a' as c_char + 'A' as c_char
        } else {
            c
        }
    };

    let cs_slice = from_raw_parts(cs, cs_len)
        .iter()
        .cloned()
        .map(to_upper)
        .collect::<Vec<c_char>>();
    let ct_slice = from_raw_parts(ct, ct_len)
        .iter()
        .cloned()
        .map(to_upper)
        .collect::<Vec<c_char>>();
    for i in 0..(cs_len - ct_len + 1) {
        if cs_slice[i..i + ct_len] == ct_slice {
            return cs.add(i) as *mut c_char;
        }
    }

    null_mut()
}
