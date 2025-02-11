//! # linux
//! This implementation of `Mmap` uses the `rustix` crate to make direct
//! `syscalls` to allocate pages and therefore whilst Linux specific, does not
//! introduce a dependency on the `libc` library and is therefore suited for
//! targets where `libc` is statically linked.
use {
    crate::{
        mmap::{Mmap, MmapProt},
        GuestAddr,
    },
    core::{
        ffi::c_void,
        ptr::null_mut,
        slice::{from_raw_parts, from_raw_parts_mut},
    },
    log::trace,
    rustix::{
        io::Errno,
        mm::{
            madvise, mmap_anonymous, mprotect, munmap, Advice, MapFlags, MprotectFlags, ProtFlags,
        },
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

    fn protect(addr: GuestAddr, len: usize, prot: MmapProt) -> Result<(), Self::Error> {
        trace!(
            "protect - addr: {:#x}, len: {:#x}, prot: {:#x}",
            addr,
            len,
            prot
        );
        unsafe {
            mprotect(addr as *mut c_void, len, MprotectFlags::from(&prot))
                .map_err(|errno| LinuxMapError::FailedToMprotect(addr, len, prot, errno))
        }
    }

    fn huge_pages(addr: GuestAddr, len: usize) -> Result<(), Self::Error> {
        trace!("huge_pages - addr: {:#x}, len: {:#x}", addr, len);
        unsafe {
            madvise(addr as *mut c_void, len, Advice::LinuxHugepage)
                .map_err(|errno| LinuxMapError::FailedToMadviseHugePage(addr, len, errno))
        }
    }

    fn dont_dump(addr: GuestAddr, len: usize) -> Result<(), Self::Error> {
        trace!("dont_dump - addr: {:#x}, len: {:#x}", addr, len);
        unsafe {
            madvise(addr as *mut c_void, len, Advice::LinuxDontDump)
                .map_err(|errno| LinuxMapError::FailedToMadviseDontDump(addr, len, errno))
        }
    }
}

impl From<&MmapProt> for MprotectFlags {
    fn from(prot: &MmapProt) -> Self {
        let mut ret = MprotectFlags::empty();
        if prot.contains(MmapProt::READ) {
            ret |= MprotectFlags::READ;
        }
        if prot.contains(MmapProt::WRITE) {
            ret |= MprotectFlags::WRITE;
        }
        if prot.contains(MmapProt::EXEC) {
            ret |= MprotectFlags::EXEC;
        }
        ret
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
    #[error("Failed to mprotect - addr: {0}, len: {1}, prot: {2:?}, errno: {3}")]
    FailedToMprotect(GuestAddr, usize, MmapProt, Errno),
    #[error("Failed to madvise HUGEPAGE - addr: {0}, len: {1}, errno: {2}")]
    FailedToMadviseHugePage(GuestAddr, usize, Errno),
    #[error("Failed to madvise DONTDUMP - addr: {0}, len: {1}, errno: {2}")]
    FailedToMadviseDontDump(GuestAddr, usize, Errno),
}
