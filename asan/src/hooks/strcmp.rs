use {
    crate::hooks::asan_load,
    core::{
        cmp::Ordering,
        ffi::{c_char, c_int, c_void},
        slice::from_raw_parts,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strcmp"]
pub unsafe extern "C" fn strcmp(cs: *const c_char, ct: *const c_char) -> c_int {
    trace!("strcmp - cs: {:p}, ct: {:p}", cs, ct);

    if cs.is_null() {
        panic!("strcmp - cs is null");
    }

    if ct.is_null() {
        panic!("strcmp - ct is null");
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

    if cs_len != ct_len {
        return (cs_len - ct_len) as c_int;
    }

    let size = cs_len;

    let slice1 = from_raw_parts(cs as *const u8, size);
    let slice2 = from_raw_parts(ct as *const u8, size);

    for i in 0..size {
        match slice1[i].cmp(&slice2[i]) {
            Ordering::Equal => (),
            Ordering::Less => return -1,
            Ordering::Greater => return 1,
        }
    }

    0
}
