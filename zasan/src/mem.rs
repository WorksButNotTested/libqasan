use {
    asan::{
        allocator::backend::{dlmalloc::DlmallocBackend, GlobalAllocator},
        mmap::linux::LinuxMmap,
    },
    core::slice::{from_raw_parts, from_raw_parts_mut},
};

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator<DlmallocBackend<LinuxMmap>> =
    GlobalAllocator::new(DlmallocBackend::new());

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, count: usize) {
    let src_slice = from_raw_parts(src, count);
    let dest_slice = from_raw_parts_mut(dest, count);

    if src < dest {
        #[allow(clippy::manual_memcpy)]
        for i in 0..count {
            let idx = count - 1 - i;
            dest_slice[idx] = src_slice[idx];
        }
    } else {
        #[allow(clippy::manual_memcpy)]
        for i in 0..count {
            dest_slice[i] = src_slice[i];
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, count: usize) {
    let src_slice = from_raw_parts(src, count);
    let dest_slice = from_raw_parts_mut(dest, count);
    #[allow(clippy::manual_memcpy)]
    for i in 0..count {
        dest_slice[i] = src_slice[i];
    }
}

#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, value: u8, count: usize) {
    let dest_slice = from_raw_parts_mut(dest, count);
    #[allow(clippy::needless_range_loop)]
    for i in 0..count {
        dest_slice[i] = value;
    }
}
