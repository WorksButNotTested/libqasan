//! # nostd
//! This module is used to support `no_std` environments.
use {core::panic::PanicInfo, log::error};

#[cfg(feature = "libc")]
pub use crate::nostd::libc::die;

#[cfg(feature = "linux")]
pub use crate::nostd::linux::die;

#[cfg(feature = "libc")]
pub mod libc;

#[cfg(feature = "linux")]
pub mod linux;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic!");
    error!("INFO: {}", info);
    die();
}

#[no_mangle]
extern "C" fn rust_eh_personality() {
    error!("rust_eh_personality");
    die();
}
