//! # asan
//!
//! `asan` is a library intended to be used by a guest running in QEMU to
//! support address sanitizer.
//!
//! It has a modular design intended to support different use cases and
//! environments. The following initial variants are proposed:
//!
//! - `qasan` - Intended as a drop in replacement for the original libqasan,
//!   this will have dependency on `libc` and will interact with QEMU using the
//!   bespoke syscall interface to perform memory tracking and shadowing.
//! - `gasan` - This is similar to `qasan`, but rather than having QEMU perform
//!   the management of the shadow memory and memory tracking, this work will be
//!   carried out purely in the guest (and hence should be more performant).
//! - `zasan` - This variant is intended to have no dependencies on libc, nor
//!   any other libraries. It is intended to be used in bare-metal targets or
//!   targets which have statically linked `libc`.
//!
//! The componentized nature of the design is intended to permit the user to
//! adapt `asan` to their needs with minimal modification by selecting and
//! combining alternative implementations of the various key components.
//!
//! ## Features
//! - `dlmalloc` - Enable support for the dlmalloc allocator backend.
//! - `guest` - Enable support for shadow memory and tracking in the guest
//! - `host` - Enable support for shadow memory and tracking in the host
//! - `libc` - Enable use of `LibcMmap` to support creation of mappings using
//!   `libc`
//! - `linux` - Enable use of `LinuxMmap` to support creation of mappings and
//!   host interaction using `rustix`.
//! - `std` - Disable the magic used to support `no_std` environments
//!
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(target_arch = "powerpc", feature(asm_experimental_arch))]

pub mod host_info;

pub mod allocator;

#[cfg(feature = "host")]
pub mod host;

pub mod mmap;

#[cfg(not(feature = "std"))]
mod nostd;

pub mod shadow;

pub mod tracking;

extern crate alloc;

pub type GuestAddr = usize;
