use {
    crate::{
        hooks::{asan_panic, asan_store, asan_sym, size_t, ssize_t},
        symbols::{AtomicGuestAddr, Function, FunctionPointer},
    },
    core::ffi::{c_char, c_long},
    libc::{c_int, c_void, SYS_read},
    log::trace,
};

#[derive(Debug)]
struct FunctionSyscall;

impl Function for FunctionSyscall {
    type Func = unsafe extern "C" fn(num: c_long, ...) -> c_long;
    const NAME: &'static str = "syscall\0";
}

static SYSCALL_ADDR: AtomicGuestAddr = AtomicGuestAddr::new();

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_read")]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    trace!("read - fd: {:#x}, buf: {:p}, count: {:#x}", fd, buf, count);

    if buf.is_null() && count != 0 {
        asan_panic(c"read - buf is null".as_ptr() as *const c_char);
    }

    asan_store(buf, count);
    let addr = SYSCALL_ADDR
        .get_or_insert_with(|| asan_sym(FunctionSyscall::NAME.as_ptr() as *const c_char));
    let syscall = FunctionSyscall::as_ptr(addr).unwrap();
    let ret = syscall(SYS_read, fd, buf, count);
    ret as ssize_t
}
