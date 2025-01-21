//! # host
//! Like `HostShadow` this implementation makes use of a `Host` implementation
//! in order to relay the requests for memory tracking to the host emulator. In
//! the case of QEMU on Linux, this will typically be by means of a bespoke
//! `syscall`.
use {
    crate::{
        host::{linux::LinuxHost, Host},
        tracking::Tracking,
        GuestAddr,
    },
    log::debug,
    syscalls::Errno,
};

#[derive(Debug)]
pub struct HostTracking;

impl Tracking for HostTracking {
    type Error = Errno;

    fn alloc(&mut self, start: GuestAddr, len: usize) -> Result<(), Self::Error> {
        debug!("alloc - start: 0x{:x}, len: 0x{:x}", start, len);
        /* Here QEMU expects a start and end, rather than start and length */
        LinuxHost::alloc(start, start + len)
    }

    fn dealloc(&mut self, start: GuestAddr) -> Result<(), Self::Error> {
        debug!("free - start: 0x{:x}", start);
        LinuxHost::dealloc(start)
    }
}

impl HostTracking {
    pub fn new() -> Result<Self, Errno> {
        Ok(HostTracking)
    }
}
