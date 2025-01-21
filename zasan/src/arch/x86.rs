use core::{cmp::Ordering, slice::from_raw_parts};

#[no_mangle]
pub unsafe extern "C" fn memcmp(ptr1: *const u8, ptr2: *const u8, count: usize) -> i32 {
    let slice1 = from_raw_parts(ptr1, count);
    let slice2 = from_raw_parts(ptr2, count);

    for i in 0..count {
        match slice1[i].cmp(&slice2[i]) {
            Ordering::Equal => (),
            Ordering::Less => return -1,
            Ordering::Greater => return 1,
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut i = 0;
    let mut cursor = s;

    while *cursor != 0 {
        cursor = cursor.offset(1);
        i += 1;
    }

    i
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(ptr1: *const u8, ptr2: *const u8, count: usize) -> i32 {
    let slice1 = from_raw_parts(ptr1, count);
    let slice2 = from_raw_parts(ptr2, count);

    for i in 0..count {
        if slice1[i] != slice2[i] {
            return 1;
        }
    }

    0
}
