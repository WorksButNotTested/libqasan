//! # dlmalloc
//! This allocator makes use of the `dlmalloc` crate to manage memory. It in
//! turn uses pages of memory allocated by one of the implementations of the
//! `Mmap` trait described in the `mmap` module.
use {
    crate::{allocator::backend::AllocatorBackend, mmap::Mmap, GuestAddr},
    alloc::{
        collections::BTreeMap,
        fmt::{self, Debug, Formatter},
    },
    core::ptr::null_mut,
    dlmalloc::{Allocator, Dlmalloc},
    log::debug,
    spin::Mutex,
    thiserror::Error,
};

const PAGE_SIZE: usize = 4096;

pub struct DlmallocBackendMap<M: Mmap> {
    maps: Mutex<BTreeMap<GuestAddr, M>>,
}

unsafe impl<M: Mmap + Send> Allocator for DlmallocBackendMap<M> {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        debug!("alloc - size: 0x{:x}", size);
        let map = M::map(size);
        match map {
            Ok(mut map) => {
                let slice = map.as_mut_slice();
                let addr = slice.as_mut_ptr() as GuestAddr;
                let result = (slice.as_mut_ptr(), slice.len(), 0);
                let mut maps = self.maps.lock();
                maps.insert(addr, map);
                result
            }
            Err(e) => {
                debug!("alloc failed: {:#?}", e);
                (null_mut(), 0, 0)
            }
        }
    }

    fn remap(&self, ptr: *mut u8, oldsize: usize, newsize: usize, can_move: bool) -> *mut u8 {
        debug!(
            "remap - ptr: 0x{:p}, oldsize: 0x{:x}, newsize: 0x{:x}, can_move: {}",
            ptr, oldsize, newsize, can_move
        );
        null_mut()
    }

    fn free_part(&self, ptr: *mut u8, oldsize: usize, newsize: usize) -> bool {
        debug!(
            "free_part - ptr: 0x{:p}, oldsize: 0x{:x}, newsize: 0x{:x}",
            ptr, oldsize, newsize
        );
        false
    }

    fn free(&self, ptr: *mut u8, size: usize) -> bool {
        debug!("free - ptr: 0x{:p}, size: 0x{:x}", ptr, size);
        let mut maps = self.maps.lock();
        let addr = ptr as GuestAddr;
        maps.remove(&addr).is_some()
    }

    fn can_release_part(&self, flags: u32) -> bool {
        debug!("can_release_part - flags: 0x{:x}", flags);
        false
    }

    fn allocates_zeros(&self) -> bool {
        debug!("allocates_zeros");
        true
    }

    fn page_size(&self) -> usize {
        debug!("page_size");
        PAGE_SIZE
    }
}

impl<M: Mmap> DlmallocBackendMap<M> {
    pub const fn new() -> DlmallocBackendMap<M> {
        DlmallocBackendMap {
            maps: Mutex::new(BTreeMap::new()),
        }
    }
}

impl<M: Mmap> Default for DlmallocBackendMap<M> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DlmallocBackend<M: Mmap> {
    dlmalloc: Dlmalloc<DlmallocBackendMap<M>>,
}

impl<M: Mmap + Send> Debug for DlmallocBackend<M> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "DlmallocBackend")
    }
}

impl<M: Mmap + Send> AllocatorBackend for DlmallocBackend<M> {
    type Error = DlmallocBackendError;

    fn alloc(&mut self, size: usize, align: usize) -> Result<GuestAddr, DlmallocBackendError> {
        debug!("alloc - size: 0x{:x}, align: 0x{:x}", size, align);
        let ptr = unsafe { self.dlmalloc.malloc(size, align) };
        if ptr.is_null() {
            Err(DlmallocBackendError::FailedToAllocate(size, align))?;
        }
        Ok(ptr as GuestAddr)
    }

    fn dealloc(&mut self, addr: GuestAddr, size: usize, align: usize) -> Result<(), Self::Error> {
        debug!(
            "dealloc - addr: 0x{:x}, size: 0x{:x}, align: 0x{:x}",
            addr, size, align
        );
        unsafe { self.dlmalloc.free(addr as *mut u8, size, align) }
        Ok(())
    }
}

impl<M: Mmap + Send> DlmallocBackend<M> {
    pub const fn new() -> DlmallocBackend<M> {
        let backend = DlmallocBackendMap::new();
        let dlmalloc = Dlmalloc::<DlmallocBackendMap<M>>::new_with_allocator(backend);
        Self { dlmalloc }
    }
}

impl<M: Mmap + Send> Default for DlmallocBackend<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum DlmallocBackendError {
    #[error("Failed to allocate - size: {0}, align: {1}")]
    FailedToAllocate(usize, usize),
}
