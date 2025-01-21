//! # frontend
//! The frontend of the allocator is responsible for applying the value-added
//! asan features on behalf of incoming user requests for allocations including
//! red-zones, poisoning and memory tracking.
use {crate::GuestAddr, alloc::fmt::Debug};

pub mod default;

pub trait Allocator: Sized {
    type Error: Debug;
    fn alloc(&mut self, len: usize, align: usize) -> Result<GuestAddr, Self::Error>;
    fn dealloc(&mut self, addr: GuestAddr) -> Result<(), Self::Error>;
}
