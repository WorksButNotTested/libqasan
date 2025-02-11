//! # nostd
//! This module is used to support `no_std` environments.
use {crate::exit::abort, core::panic::PanicInfo, log::error};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic!");
    error!("INFO: {}", info);
    abort();
}

#[no_mangle]
extern "C" fn rust_eh_personality() {
    error!("rust_eh_personality");
    abort();
}
