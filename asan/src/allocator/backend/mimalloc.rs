//! # dlmalloc
//! This allocator makes use of the `mimalloc` crate to manage memory.

use {
    crate::{allocator::backend::AllocatorBackend, GuestAddr},
    alloc::fmt::{self, Debug, Formatter},
    core::alloc::{GlobalAlloc, Layout, LayoutError},
    log::debug,
    mimalloc::MiMalloc,
    thiserror::Error,
};

pub struct MimallocBackend {
    mimalloc: MiMalloc,
}

impl Debug for MimallocBackend {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "MimallocBackend")
    }
}

impl AllocatorBackend for MimallocBackend {
    type Error = MimallocBackendError;

    fn alloc(&mut self, len: usize, align: usize) -> Result<crate::GuestAddr, Self::Error> {
        debug!("alloc - size: {}, align: {}", len, align);
        let layout = Layout::from_size_align(len, align)
            .map_err(|e| MimallocBackendError::InvalidLayout(e))?;
        Ok(unsafe { self.mimalloc.alloc(layout) } as GuestAddr)
    }

    fn dealloc(
        &mut self,
        addr: crate::GuestAddr,
        len: usize,
        align: usize,
    ) -> Result<(), Self::Error> {
        debug!(
            "dealloc - addr: 0x{:x}, size: 0x{:x}, align: 0x{:x}",
            addr, len, align
        );
        let layout = Layout::from_size_align(len, align)
            .map_err(|e| MimallocBackendError::InvalidLayout(e))?;
        unsafe { self.mimalloc.dealloc(addr as *mut u8, layout) }
        Ok(())
    }
}

impl MimallocBackend {
    pub const fn new() -> Self {
        MimallocBackend { mimalloc: MiMalloc }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum MimallocBackendError {
    #[error("Invalid layout: {0}")]
    InvalidLayout(LayoutError),
}
