use {
    crate::hooks::{asan_load, wchar_t},
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
#[export_name = "patch_wcscmp"]
pub unsafe extern "C" fn wcscmp(cs: *const wchar_t, ct: *const wchar_t) -> c_int {
    trace!("wcscmp - cs: {:p}, ct: {:p}", cs, ct);

    if cs.is_null() {
        panic!("wcscmp - cs is null");
    }

    if ct.is_null() {
        panic!("wcscmp - ct is null");
    }

    let mut cs_len = 0;
    while *cs.add(cs_len) != 0 {
        cs_len += 1;
    }
    let mut ct_len = 0;
    while *ct.add(ct_len) != 0 {
        ct_len += 1;
    }
    asan_load(cs as *const c_void, size_of::<wchar_t>() * (cs_len + 1));
    asan_load(ct as *const c_void, size_of::<wchar_t>() * (ct_len + 1));

    if cs_len != ct_len {
        return (cs_len - ct_len) as c_int;
    }

    let size = cs_len;

    let slice1 = from_raw_parts(cs, size);
    let slice2 = from_raw_parts(ct, size);

    for i in 0..size {
        match slice1[i].cmp(&slice2[i]) {
            Ordering::Equal => (),
            Ordering::Less => return -1,
            Ordering::Greater => return 1,
        }
    }

    0
}
