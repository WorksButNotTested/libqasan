use {
    core::ffi::{c_int, c_void},
    log::error,
};

#[no_mangle]
extern "C" fn _Unwind_Resume() {
    error!("_Unwind_Resume");
    loop {}
}

// Rustix does not currently implement these necessary symbols for powerpc.
#[no_mangle]
pub unsafe extern "C" fn munmap(_ptr: *mut c_void, _len: usize) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn mmap64(
    _addr: *mut c_void,
    _length: usize,
    _prot: c_int,
    _flags: c_int,
    _fd: c_int,
    _offset: u64,
) -> *mut c_void {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn __errno_location() -> *mut c_int {
    unimplemented!();
}
