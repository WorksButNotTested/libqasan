//! # mmap
//! This module provides implementations for creating memory mappings. This is
//! used by the guest shadow implementation and can also be used by allocator
//! backends.
use {crate::GuestAddr, alloc::fmt::Debug};

#[cfg(feature = "libc")]
pub mod libc;

#[cfg(feature = "linux")]
pub mod linux;

pub trait Mmap: Sized + Ord + Debug {
    type Error: Debug;
    fn map(len: usize) -> Result<Self, Self::Error>;
    fn map_at(addr: GuestAddr, len: usize) -> Result<Self, Self::Error>;
    fn as_slice(&self) -> &[u8];
    fn as_mut_slice(&mut self) -> &mut [u8];
}
