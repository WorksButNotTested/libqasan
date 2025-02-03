//! # symbols
//! This module provides implementations symbol lookups. The ability to
//! substitute this functionality may be helpful for targets where
//! conventional symbol lookup is not possible, e.g. if libc is statically
//! linked
use alloc::fmt::Debug;

#[cfg(feature = "libc")]
pub mod dlsym;

pub trait Symbols: Debug + Sized {
    type Error: Debug;
    fn lookup<F: Copy>(name: &str) -> Result<F, Self::Error>;
}
