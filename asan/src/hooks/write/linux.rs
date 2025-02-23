use {
    crate::{asan_load, asan_panic, size_t, ssize_t},
    core::{
        ffi::{c_char, c_int, c_void},
        slice::from_raw_parts,
    },
    log::trace,
    rustix::{fd::BorrowedFd, io},
};

/// # Safety
/// See man pages
#[cfg_attr(not(feature = "test"), no_mangle)]
#[cfg_attr(feature = "test", export_name = "patch_write")]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    trace!("write - fd: {:#x}, buf: {:p}, count: {:#x}", fd, buf, count);

    if buf.is_null() && count != 0 {
        asan_panic(c"write - buf is null".as_ptr() as *const c_char);
    }

    asan_load(buf, count);
    let file = BorrowedFd::borrow_raw(fd);
    let data = from_raw_parts(buf as *const u8, count as usize);
    if let Ok(ret) = io::write(file, data) {
        return ret as ssize_t;
    } else {
        return -1;
    }
}
