use {
    crate::{
        hooks::{asan_track, asan_unpoison, off_t, size_t},
        GuestAddr,
    },
    core::ffi::{c_int, c_void},
    log::trace,
    rustix::{
        fd::BorrowedFd,
        mm::{mmap as rmmap, MapFlags, ProtFlags},
    },
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_mmap")]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    len: size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off_t,
) -> *mut c_void {
    trace!(
        "mmap - addr: {:p}, len: {:#x}, prot: {:#x}, flags: {:#x}, fd: {:#x}, offset: {:#x}",
        addr,
        len,
        prot,
        flags,
        fd,
        offset
    );
    let file = BorrowedFd::borrow_raw(fd);
    let mmap_prot = ProtFlags::from_bits_retain(prot as u32);
    let mmap_flags = MapFlags::from_bits_retain(flags as u32);
    if let Ok(map) = rmmap(addr, len, mmap_prot, mmap_flags, file, offset as u64) {
        asan_unpoison(map, len);
        asan_track(map, len);
        map
    } else {
        GuestAddr::MAX as *mut c_void
    }
}
