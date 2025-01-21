//! # linux
//! This implementation of `Mmap` uses the `rustix` crate to make direct
//! `syscalls` to allocate pages and therefore whilst Linux specific, does not
//! introduce a dependency on the `libc` library and is therefore suited for
//! targets where `libc` is statically linked.
use {
    crate::{mmap::Mmap, GuestAddr},
    core::{
        ffi::c_void,
        ptr::null_mut,
        slice::{from_raw_parts, from_raw_parts_mut},
    },
    log::trace,
    rustix::{
        io::Errno,
        mm::{mmap_anonymous, munmap, MapFlags, ProtFlags},
    },
    thiserror::Error,
};

#[derive(Ord, PartialOrd, PartialEq, Eq, Debug)]
pub struct LinuxMmap {
    addr: GuestAddr,
    len: usize,
}

impl Mmap for LinuxMmap {
    type Error = LinuxMapError;
    fn map(len: usize) -> Result<Self, Self::Error> {
        unsafe {
            let addr = mmap_anonymous(
                null_mut(),
                len,
                ProtFlags::READ | ProtFlags::WRITE,
                MapFlags::PRIVATE | MapFlags::NORESERVE,
            )
            .map_err(|errno| LinuxMapError::FailedToMap(len, errno))?
                as GuestAddr;
            trace!("Mapped: 0x{:x}-0x{:x}", addr, addr + len);
            Ok(Self { addr, len })
        }
    }

    fn map_at(addr: GuestAddr, len: usize) -> Result<LinuxMmap, LinuxMapError> {
        unsafe {
            mmap_anonymous(
                addr as *mut c_void,
                len,
                ProtFlags::READ | ProtFlags::WRITE,
                MapFlags::PRIVATE
                    | MapFlags::FIXED
                    | MapFlags::FIXED_NOREPLACE
                    | MapFlags::NORESERVE,
            )
            .map_err(|errno| LinuxMapError::FailedToMapAt(addr, len, errno))?;
            trace!("Mapped: 0x{:x}-0x{:x}", addr, addr + len);
        };
        Ok(Self { addr, len })
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts(self.addr as *const u8, self.len) }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { from_raw_parts_mut(self.addr as *mut u8, self.len) }
    }
}

impl Drop for LinuxMmap {
    fn drop(&mut self) {
        unsafe {
            munmap(self.addr as *mut c_void, self.len).unwrap();
        }
        trace!("Unmapped: 0x{:x}-0x{:x}", self.addr, self.addr + self.len);
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum LinuxMapError {
    #[error("Failed to map - len: {0}, errno: {1}")]
    FailedToMap(usize, Errno),
    #[error("Failed to map: {0}, len: {1}, errno: {2}")]
    FailedToMapAt(GuestAddr, usize, Errno),
}
