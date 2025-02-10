use {
    crate::hooks::{asan_store, size_t, ssize_t},
    core::{
        ffi::{c_int, c_void},
        slice::from_raw_parts_mut,
    },
    log::trace,
    rustix::{fd::BorrowedFd, io},
};

#[no_mangle]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    trace!("read - fd: {:#x}, buf: {:p}, count: {:#x}", fd, buf, count);
    asan_store(buf, count);
    let file = BorrowedFd::borrow_raw(fd);
    let data = from_raw_parts_mut(buf as *mut u8, count as usize);
    if let Ok(ret) = io::read(file, data) {
        return ret as ssize_t;
    } else {
        return -1;
    }
}
