use {
    crate::{
        symbols::{Function, FunctionPointer},
        GuestAddr,
    },
    core::ffi::{c_char, c_int},
    libc::{pid_t, SIGABRT},
};

#[derive(Debug)]
struct FunctionGetpid;

impl Function for FunctionGetpid {
    type Func = unsafe extern "C" fn() -> pid_t;
    const NAME: &'static str = "getpid\0";
}

#[derive(Debug)]
struct FunctionKill;

impl Function for FunctionKill {
    type Func = unsafe extern "C" fn(pid_t, c_int) -> c_int;
    const NAME: &'static str = "kill\0";
}

extern "C" {
    fn asan_sym(name: *const c_char) -> GuestAddr;
}

pub fn die() -> ! {
    let getpid_addr = unsafe { asan_sym(FunctionGetpid::NAME.as_ptr() as *const c_char) };
    let fn_getpid = FunctionGetpid::as_ptr(getpid_addr).unwrap();

    let kill_addr = unsafe { asan_sym(FunctionKill::NAME.as_ptr() as *const c_char) };
    let fn_kill = FunctionKill::as_ptr(kill_addr).unwrap();

    let pid = unsafe { fn_getpid() };
    unsafe { fn_kill(pid, SIGABRT) };
    unreachable!();
}
