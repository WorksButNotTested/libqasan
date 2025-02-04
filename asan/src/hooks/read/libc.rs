use {
    crate::{
        asan_store, asan_sym,
        hooks::{size_t, ssize_t},
        symbols::{Function, FunctionPointer},
    },
    core::ffi::c_long,
    libc::{c_int, c_void, SYS_read},
    log::trace,
};

#[derive(Debug)]
struct FunctionSyscall;

impl Function for FunctionSyscall {
    type Func = unsafe extern "C" fn(num: c_long, ...) -> c_long;
    const NAME: &'static str = "syscall\0";
}

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    trace!("read - fd: {:#x}, buf: {:p}, count: {:#x}", fd, buf, count);
    asan_store(buf, count);
    let symbol = asan_sym("syscall");
    let syscall = FunctionSyscall::as_ptr(symbol).unwrap();
    let ret = syscall(SYS_read, fd, buf, count);
    ret as ssize_t
}
