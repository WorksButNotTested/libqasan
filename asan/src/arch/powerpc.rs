use core::ffi::{c_int, c_void};

#[allow(non_camel_case_types)]
type pid_t = i32;

// Rustix does not currently implement these necessary symbols for powerpc.
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

#[no_mangle]
pub unsafe extern "C" fn kill(_pid: pid_t, _sig: c_int) -> c_int {
    unimplemented!();
}
