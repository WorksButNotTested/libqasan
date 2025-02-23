use {crate::exit::abort, log::error};

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "arm")]
mod arm;

#[cfg(target_arch = "powerpc")]
mod powerpc;

#[no_mangle]
extern "C" fn _Unwind_Resume() {
    error!("_Unwind_Resume");
    abort();
}
