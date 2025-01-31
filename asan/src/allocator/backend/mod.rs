//! # backend
//! The backend is responsible for allocating the underlying memory used by the
//! application. At present there is only one implemented allocator:
//!
//! - `dlmalloc` - A pure rust allocator based on the `dlmalloc` crate.
//!
//! A number other of possible implementations could be considered:
//! - A simple bump allocator allocating from a fixed memory buffer
//! - An allocator which calls down into the original `libc` implementation of `malloc`

use {crate::GuestAddr, alloc::fmt::Debug, core::alloc::GlobalAlloc, log::debug, spin::Mutex};

#[cfg(feature = "dlmalloc")]
pub mod dlmalloc;

#[cfg(feature = "mimalloc")]
pub mod mimalloc;

pub trait AllocatorBackend: Sized + Debug {
    type Error: Debug;
    fn alloc(&mut self, len: usize, align: usize) -> Result<GuestAddr, Self::Error>;
    fn dealloc(&mut self, addr: GuestAddr, len: usize, align: usize) -> Result<(), Self::Error>;
}

pub struct GlobalAllocator<A: AllocatorBackend> {
    allocator: Mutex<A>,
}

unsafe impl<A: AllocatorBackend> GlobalAlloc for GlobalAllocator<A> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        debug!("alloc - size: {}, align: {}", layout.size(), layout.align());
        let mut allocator = self.allocator.lock();
        let addr = allocator
            .alloc(layout.size(), layout.align())
            .unwrap_or_default();
        addr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        debug!(
            "dealloc - ptr: {:p}, size: {}, align: {}",
            ptr,
            layout.size(),
            layout.align()
        );
        let mut allocator = self.allocator.lock();
        allocator
            .dealloc(ptr as GuestAddr, layout.size(), layout.align())
            .unwrap();
    }
}

impl<A: AllocatorBackend> GlobalAllocator<A> {
    pub const fn new(allocator: A) -> GlobalAllocator<A> {
        GlobalAllocator {
            allocator: Mutex::new(allocator),
        }
    }
}
