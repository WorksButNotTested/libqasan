use {
    crate::{
        asan_load,
        hooks::{size_t, ssize_t},
    },
    core::{
        ffi::{c_int, c_void},
        slice::from_raw_parts,
    },
    log::trace,
    rustix::{fd::BorrowedFd, io},
};

#[no_mangle]
unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    trace!("write - fd: {:#x}, buf: {:p}, count: {:#x}", fd, buf, count);
    asan_load(buf, count);
    let file = BorrowedFd::borrow_raw(fd);
    let data = from_raw_parts(buf as *const u8, count as usize);
    if let Ok(ret) = io::write(file, data) {
        return ret as ssize_t;
    } else {
        return -1;
    }
}
