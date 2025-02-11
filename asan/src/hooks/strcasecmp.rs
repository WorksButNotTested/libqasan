use {
    crate::hooks::asan_load,
    core::{
        ffi::{c_char, c_int, c_void},
        slice::from_raw_parts,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strcasecmp"]
pub unsafe extern "C" fn strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int {
    trace!("strcasecmp - s1: {:p}, s2: {:p}", s1, s2);

    if s1.is_null() {
        panic!("strcasecmp - s1 is null");
    }

    if s2.is_null() {
        panic!("strcasecmp - s2 is null");
    }

    let mut s1_len = 0;
    while *s1.add(s1_len) != 0 {
        s1_len += 1;
    }
    let mut s2_len = 0;
    while *s2.add(s2_len) != 0 {
        s2_len += 1;
    }
    asan_load(s1 as *const c_void, s1_len + 1);
    asan_load(s2 as *const c_void, s2_len + 1);

    if s1_len != s2_len {
        return (s1_len - s2_len) as c_int;
    }

    let len = s1_len;

    let to_upper = |c: c_char| -> c_char {
        if ('a' as c_char..='z' as c_char).contains(&c) {
            c - 'a' as c_char + 'A' as c_char
        } else {
            c
        }
    };

    let s1_slice = from_raw_parts(s1, len);
    let s2_slice = from_raw_parts(s2, len);
    for (lc1, lc2) in s1_slice
        .iter()
        .cloned()
        .map(to_upper)
        .zip(s2_slice.iter().cloned().map(to_upper))
    {
        if lc1 != lc2 {
            return (lc1 - lc2) as c_int;
        }
    }

    0
}
