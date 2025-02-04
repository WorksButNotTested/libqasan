use {
    crate::{
        asan_load, asan_sym,
        hooks::{size_t, ssize_t},
        symbols::{Function, FunctionPointer},
    },
    core::ffi::c_long,
    libc::{c_int, c_void, SYS_write},
    log::trace,
};

#[derive(Debug)]
struct FunctionSyscall;

impl Function for FunctionSyscall {
    type Func = unsafe extern "C" fn(num: c_long, ...) -> c_long;
    const NAME: &'static str = "syscall\0";
}

#[no_mangle]
unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    trace!("write - fd: {:#x}, buf: {:p}, count: {:#x}", fd, buf, count);
    asan_load(buf, count);
    let symbol = asan_sym("syscall");
    let syscall = FunctionSyscall::as_ptr(symbol).unwrap();
    let ret = syscall(SYS_write, fd, buf, count);
    ret as ssize_t
}
